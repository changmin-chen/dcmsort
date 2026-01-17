use anyhow::{Context, Result};
use dicom_dictionary_std::tags;
use dicom_object::{DefaultDicomObject, OpenFileOptions, Tag};
use dicom_object::file::ReadPreamble;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct DicomMeta {
    pub path: PathBuf,

    pub patient_id: Option<String>,
    pub patient_name: Option<String>,

    pub study_uid: Option<String>,
    pub series_uid: Option<String>,
    pub sop_uid: Option<String>,

    pub modality: Option<String>,
    pub study_date: Option<String>,
    pub series_number: Option<i32>,
    pub instance_number: Option<i32>,

    pub study_description: Option<String>,
    pub series_description: Option<String>,

    pub image_position_patient: Option<[f64; 3]>,
    pub image_orientation_patient: Option<[f64; 6]>,
}

pub fn read_meta(path: &Path) -> Result<DicomMeta> {
    // Header-only read: stop before Pixel Data (7FE0,0010).
    // This avoids loading huge pixel payloads into memory.
    let obj: DefaultDicomObject = OpenFileOptions::new()
        .read_preamble(ReadPreamble::Auto)
        .read_until(tags::PIXEL_DATA)
        .open_file(path)
        .with_context(|| format!("open DICOM (header-only): {}", path.display()))?;

    Ok(DicomMeta {
        path: path.to_path_buf(),

        patient_id: opt_str(&obj, tags::PATIENT_ID),
        patient_name: opt_str(&obj, tags::PATIENT_NAME),

        study_uid: opt_str(&obj, tags::STUDY_INSTANCE_UID),
        series_uid: opt_str(&obj, tags::SERIES_INSTANCE_UID),
        sop_uid: opt_str(&obj, tags::SOP_INSTANCE_UID),

        modality: opt_str(&obj, tags::MODALITY),
        study_date: opt_str(&obj, tags::STUDY_DATE),
        series_number: opt_i32(&obj, tags::SERIES_NUMBER),
        instance_number: opt_i32(&obj, tags::INSTANCE_NUMBER),

        study_description: opt_str(&obj, tags::STUDY_DESCRIPTION),
        series_description: opt_str(&obj, tags::SERIES_DESCRIPTION),

        image_position_patient: opt_f64_vec(&obj, tags::IMAGE_POSITION_PATIENT)
            .and_then(|v| v_to_3(v)),
        image_orientation_patient: opt_f64_vec(&obj, tags::IMAGE_ORIENTATION_PATIENT)
            .and_then(|v| v_to_6(v)),
    })
}

impl DicomMeta {
    /// A geometry-based ordering scalar:
    /// dot( ImagePositionPatient, cross(row, col) )
    /// Returns None if required tags are missing or invalid.
    pub fn geom_order(&self) -> Option<f64> {
        let ipp = self.image_position_patient?;
        let iop = self.image_orientation_patient?;

        let row = [iop[0], iop[1], iop[2]];
        let col = [iop[3], iop[4], iop[5]];

        let n = cross(row, col);
        Some(dot(ipp, n))
    }

    pub fn stable_id(&self) -> String {
        self.sop_uid
            .clone()
            .unwrap_or_else(|| self.path.file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "UNKNOWN".to_string()))
    }
}

fn opt_str(obj: &DefaultDicomObject, tag: Tag) -> Option<String> {
    let s = obj.element(tag).ok()?.to_str().ok()?.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn opt_i32(obj: &DefaultDicomObject, tag: Tag) -> Option<i32> {
    opt_str(obj, tag)?.parse::<i32>().ok()
}

fn opt_f64_vec(obj: &DefaultDicomObject, tag: Tag) -> Option<Vec<f64>> {
    // Multi-valued DS often uses '\' separator.
    let raw = opt_str(obj, tag)?;
    let parts: Vec<f64> = raw
        .split('\\')
        .filter_map(|p| p.trim().parse::<f64>().ok())
        .collect();

    if parts.is_empty() { None } else { Some(parts) }
}

fn v_to_3(v: Vec<f64>) -> Option<[f64; 3]> {
    if v.len() == 3 { Some([v[0], v[1], v[2]]) } else { None }
}

fn v_to_6(v: Vec<f64>) -> Option<[f64; 6]> {
    if v.len() == 6 { Some([v[0], v[1], v[2], v[3], v[4], v[5]]) } else { None }
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
