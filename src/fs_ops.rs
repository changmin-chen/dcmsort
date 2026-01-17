use crate::cli::Mode;
use crate::sort::Plan;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn collect_files(root: &Path, follow_symlinks: bool) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(follow_symlinks)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            files.push(entry.into_path());
        }
    }
    Ok(files)
}

pub fn execute(plans: Vec<Plan>, mode: Mode, dry_run: bool) -> Result<()> {
    for p in plans {
        let dst = unique_path(&p.dst);

        if dry_run {
            println!("{} -> {}", p.src.display(), dst.display());
            continue;
        }

        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create dir: {}", parent.display()))?;
        }

        match mode {
            Mode::Copy => {
                fs::copy(&p.src, &dst)
                    .with_context(|| format!("copy {} -> {}", p.src.display(), dst.display()))?;
            }
            Mode::Move => {
                if let Err(e) = fs::rename(&p.src, &dst) {
                    // Cross-device rename fallback
                    fs::copy(&p.src, &dst)
                        .with_context(|| format!("copy (move fallback) {} -> {}", p.src.display(), dst.display()))?;
                    fs::remove_file(&p.src)
                        .with_context(|| format!("remove (move fallback) {}", p.src.display()))?;
                    // preserve original error context (optional)
                    let _ = e;
                }
            }
            Mode::HardLink => {
                if let Err(_e) = fs::hard_link(&p.src, &dst) {
                    // Fallback to copy if hardlink not possible (different volume, permissions, etc.)
                    fs::copy(&p.src, &dst)
                        .with_context(|| format!("copy (hardlink fallback) {} -> {}", p.src.display(), dst.display()))?;
                }
            }
        }
    }
    Ok(())
}

fn unique_path(dst: &Path) -> PathBuf {
    if !dst.exists() {
        return dst.to_path_buf();
    }

    let parent = dst.parent().unwrap_or_else(|| Path::new("."));
    let stem = dst.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or("file".into());
    let ext = dst.extension().map(|s| s.to_string_lossy().to_string());

    for i in 1..10_000u32 {
        let name = match &ext {
            Some(ext) => format!("{}_{}.{}", stem, i, ext),
            None => format!("{}_{}", stem, i),
        };
        let candidate = parent.join(name);
        if !candidate.exists() {
            return candidate;
        }
    }

    // If you somehow have 10k collisions, congratulations, you found a new hobby.
    dst.to_path_buf()
}
