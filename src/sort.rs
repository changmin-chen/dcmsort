use crate::types::{Layout, SortBy};
use crate::dicom::{read_meta, DicomMeta};
use crate::sanitize::sanitize_component;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Plan {
    pub src: PathBuf,
    pub dst: PathBuf,
    pub meta: DicomMeta,
}

pub fn scan(paths: &[PathBuf]) -> Vec<DicomMeta> {
    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::*;
        paths.par_iter().filter_map(|p| read_meta(p).ok()).collect()
    }
    #[cfg(not(feature = "parallel"))]
    {
        paths.iter().filter_map(|p| read_meta(p).ok()).collect()
    }
}

pub fn plan_operations(
    metas: &[DicomMeta],
    out_dir: &Path,
    layout: Layout,
    sort_by: SortBy,
    include_phi: bool,
) -> Vec<Plan> {
    // Group by (StudyUID, SeriesUID) with fallbacks
    let mut groups: HashMap<(String, String), Vec<&DicomMeta>> = HashMap::new();
    for m in metas {
        let study = m.study_uid.clone().unwrap_or_else(|| "UNKNOWN_STUDY".into());
        let series = m.series_uid.clone().unwrap_or_else(|| "UNKNOWN_SERIES".into());
        groups.entry((study, series)).or_default().push(m);
    }

    let mut plans = Vec::new();

    for ((_study, _series), mut items) in groups {
        let use_geom = match sort_by {
            SortBy::Geometry => true,
            SortBy::Instance => false,
            SortBy::Auto => items.iter().all(|m| m.geom_order().is_some()),
        };

        items.sort_by(|a, b| compare(a, b, use_geom));

        for (idx, m) in items.into_iter().enumerate() {
            let dst = build_dst(out_dir, layout, include_phi, m, idx as u32);
            plans.push(Plan {
                src: m.path.clone(),
                dst,
                meta: m.clone(),
            });
        }
    }

    plans
}

fn compare(a: &DicomMeta, b: &DicomMeta, use_geom: bool) -> Ordering {
    if use_geom {
        match (a.geom_order(), b.geom_order()) {
            (Some(x), Some(y)) => {
                let ord = x.partial_cmp(&y).unwrap_or(Ordering::Equal);
                if ord != Ordering::Equal { return ord; }
            }
            _ => {}
        }
    } else {
        match (a.instance_number, b.instance_number) {
            (Some(x), Some(y)) => {
                let ord = x.cmp(&y);
                if ord != Ordering::Equal { return ord; }
            }
            (Some(_), None) => return Ordering::Less,
            (None, Some(_)) => return Ordering::Greater,
            _ => {}
        }
    }

    // Tie-breaker: SOPInstanceUID or filename
    a.stable_id().cmp(&b.stable_id())
}

fn build_dst(
    out_dir: &Path,
    layout: Layout,
    include_phi: bool,
    m: &DicomMeta,
    order_index: u32,
) -> PathBuf {
    let patient_id = m.patient_id.clone().unwrap_or_else(|| "UNKNOWN_PATIENT".into());
    let study_uid = m.study_uid.clone().unwrap_or_else(|| "UNKNOWN_STUDY".into());
    let series_uid = m.series_uid.clone().unwrap_or_else(|| "UNKNOWN_SERIES".into());
    let sop_uid = m.sop_uid.clone().unwrap_or_else(|| "UNKNOWN_SOP".into());

    let patient = if include_phi {
        let name = m.patient_name.clone().unwrap_or_default();
        sanitize_component(&format!("{}_{}", patient_id, name))
    } else {
        sanitize_component(&patient_id)
    };

    let study = if include_phi {
        let date = m.study_date.clone().unwrap_or_default();
        let desc = m.study_description.clone().unwrap_or_default();
        sanitize_component(&format!("{}_{}_{}", date, desc, study_uid))
    } else {
        sanitize_component(&study_uid)
    };

    let series = if include_phi {
        let modl = m.modality.clone().unwrap_or_default();
        let sn = m.series_number.map(|x| x.to_string()).unwrap_or_default();
        let desc = m.series_description.clone().unwrap_or_default();
        sanitize_component(&format!("{}_{}_{}_{}", modl, sn, desc, series_uid))
    } else {
        sanitize_component(&series_uid)
    };

    let file_name = format!("{:05}_{}.dcm", order_index + 1, sanitize_component(&sop_uid));

    match layout {
        Layout::PatientStudySeries => out_dir.join(patient).join(study).join(series).join(file_name),
        Layout::StudySeries => out_dir.join(study).join(series).join(file_name),
        Layout::SeriesOnly => out_dir.join(series).join(file_name),
        Layout::Flat => out_dir.join(file_name),
    }
}
