use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    let s = serde_json::to_string_pretty(value).context("serialize json")?;
    fs::write(path, s).with_context(|| format!("write report: {}", path.display()))?;
    Ok(())
}
