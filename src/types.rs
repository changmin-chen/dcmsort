use clap::ValueEnum;

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Mode {
    Copy,
    Move,
    HardLink,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Layout {
    PatientStudySeries,
    StudySeries,
    SeriesOnly,
    Flat,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum SortBy {
    Auto,
    Instance,
    Geometry,
}
