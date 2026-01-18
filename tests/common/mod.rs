use std::path::{Path, PathBuf};
use std::fs;
use std::io::Cursor;

/// Downloads and sets up test DICOM data from dcm_qa_ct repository
/// Source: https://github.com/neurolabusc/dcm_qa_ct
/// License: BSD 2-Clause
pub fn setup_test_data() -> PathBuf {
    let target_dir = Path::new("target").join("test-data");
    let data_dir = target_dir.join("dcm_qa_ct-master");

    // Skip download if already exists
    if data_dir.exists() {
        println!("Test data already exists at: {}", data_dir.display());
        return data_dir;
    }

    fs::create_dir_all(&target_dir).expect("Failed to create test data directory");

    let url = "https://github.com/neurolabusc/dcm_qa_ct/archive/refs/heads/master.zip";
    println!("Downloading test data from {}", url);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .expect("Failed to build reqwest client");

    let response = client.get(url)
        .send()
        .expect("Failed to download test data");
    let content = response.bytes()
        .expect("Failed to read response bytes");

    let mut archive = zip::ZipArchive::new(Cursor::new(content))
        .expect("Failed to open zip archive");
    archive.extract(&target_dir)
        .expect("Failed to extract zip archive");

    println!("Test data extracted to: {}", data_dir.display());
    data_dir
}