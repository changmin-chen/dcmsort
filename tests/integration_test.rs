mod common;

use dcmsort::{sort, fs_ops};

#[test]
fn test_sort_real_dicom_series() {
    let data_dir = common::setup_test_data();
    let _dicom_dir = data_dir.join("In"); // Adjust based on actual repo structure if needed
    
    // The zip might extract to dcm_qa_ct-master/In or just dcm_qa_ct-master depending on repo structure.
    // Let's assume the repo root has the dicom files or a folder.
    // checking dcm_qa_ct structure: usually has 'In' folder or the dicoms are in root.
    // We will list files recursively so it should be fine.
    
    // Verify we can find some files
    let files = fs_ops::collect_files(&data_dir, false).expect("Failed to collect files");
    assert!(!files.is_empty(), "Should find some files in the downloaded dataset");

    let metas = sort::scan(&files);
    assert!(!metas.is_empty(), "Should find valid DICOM files");

    let output_dir = data_dir.join("sorted_output");
    // Use proper enums
    let plan = sort::plan_operations(
        &metas, 
        &output_dir, 
        dcmsort::types::Layout::SeriesOnly,
        dcmsort::types::SortBy::Auto, 
        false
    );
    
    assert!(!plan.is_empty());
    
    // Verify some expected grouping
    // This dataset (dcm_qa_ct) typically contains CT series. 
    // We expect at least one series.
    
    let unique_series = metas.iter().map(|m| &m.series_uid).collect::<std::collections::HashSet<_>>();
    println!("Found {} unique series", unique_series.len());
    assert!(unique_series.len() >= 1);
}
