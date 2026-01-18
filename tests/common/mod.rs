use std::path::{Path, PathBuf};
use std::fs;
use std::io::Cursor;

pub fn setup_test_data() -> PathBuf {
    let target_dir = Path::new("target").join("test-data");
    let data_dir = target_dir.join("dcm_qa_ct-master");

    if data_dir.exists() {
        return data_dir;
    }

    fs::create_dir_all(&target_dir).expect("Failed to create test data directory");

    let url = "https://github.com/neurolabusc/dcm_qa_ct/archive/refs/heads/master.zip";
    println!("Downloading test data from {}", url);
    let response = reqwest::blocking::get(url).expect("Failed to download test data");
    let content = response.bytes().expect("Failed to read response bytes");

    let mut archive = zip::ZipArchive::new(Cursor::new(content)).expect("Failed to open zip archive");
    archive.extract(&target_dir).expect("Failed to extract zip archive");
    
    // Find the extracted folder
    let entries = fs::read_dir(&target_dir).expect("Failed to read target dir");
    for entry in entries {
        let entry = entry.expect("Bad entry");
        if entry.path().is_dir() {
            println!("Found extracted dir: {}", entry.path().display());
            return entry.path();
        }
    }
    
    panic!("No directory extracted from zip!");
}
