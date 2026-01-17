mod cli;
mod dicom;
mod fs_ops;
mod report;
mod sanitize;
mod sort;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    // Logging: configure via RUST_LOG, e.g. "info" or "dicom_sort=debug"
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .try_init();

    let cli = cli::Cli::parse();

    let files = fs_ops::collect_files(&cli.input, cli.follow_symlinks)?;
    tracing::info!("Found {} files under {}", files.len(), cli.input.display());

    let metas = sort::scan(&files);
    tracing::info!("Parsed {} DICOM headers (others were ignored)", metas.len());

    if let Some(report_path) = &cli.report {
        report::write_json(report_path, &metas)?;
        tracing::info!("Wrote report: {}", report_path.display());
    }

    let plans = sort::plan_operations(&metas, &cli.output, cli.layout, cli.sort_by, cli.include_phi);
    tracing::info!("Planned {} operations", plans.len());

    fs_ops::execute(plans, cli.mode, cli.dry_run)?;
    Ok(())
}
