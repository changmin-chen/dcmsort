use clap::Parser;
use std::path::PathBuf;
use dcmsort::types::{Mode, Layout, SortBy};

#[derive(Parser, Debug)]
#[command(name = "dcmsort", version, about = "Sort DICOM files by metadata (header-only).")]
pub struct Cli {
    /// Input directory containing DICOM files
    #[arg(long, value_name = "DIR")]
    pub input: PathBuf,

    /// Output directory
    #[arg(long, value_name = "DIR")]
    pub output: PathBuf,

    /// File operation mode
    #[arg(long, value_enum, default_value_t = Mode::Copy)]
    pub mode: Mode,

    /// Print planned operations without touching the filesystem
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Follow symlinks while scanning (off by default)
    #[arg(long, default_value_t = false)]
    pub follow_symlinks: bool,

    /// Folder layout strategy
    #[arg(long, value_enum, default_value_t = Layout::PatientStudySeries)]
    pub layout: Layout,

    /// Sorting strategy within a series
    #[arg(long, value_enum, default_value_t = SortBy::Auto)]
    pub sort_by: SortBy,

    /// Allow PHI-like fields (PatientName / descriptions) to appear in folder names.
    /// Default is OFF for safety.
    #[arg(long, default_value_t = false)]
    pub include_phi: bool,

    /// Write a JSON report (metadata only)
    #[arg(long, value_name = "FILE")]
    pub report: Option<PathBuf>,
}
