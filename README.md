# dcmsort

A Rust console application for sorting medical imaging DICOM files by metadata, designed for efficiency and safety.

## Features

- **Header-only reading**: Stops before Pixel Data to avoid memory issues with large datasets
- **Flexible sorting**: Auto-detect geometry-based or instance number sorting
- **Multiple layouts**: Patient/Study/Series hierarchy or flatter structures
- **PHI safety**: By default, excludes Protected Health Information from folder names
- **File operations**: Copy, move, or hardlink with automatic fallbacks
- **Dry-run mode**: Preview operations before executing
- **JSON reports**: Export metadata for validation and post-processing
- **Optional parallelization**: Enable with `--features parallel`

## Installation

```bash
cargo build --release
```

For parallel processing (faster on multi-core systems):

```bash
cargo build --release --features parallel
```

## Usage

Basic usage:

```bash
dcmsort --input /path/to/dicom/files --output /path/to/sorted --dry-run
```

### Command-line Options

- `--input <DIR>`: Input directory containing DICOM files (required)
- `--output <DIR>`: Output directory for sorted files (required)
- `--mode <MODE>`: File operation mode: `copy`, `move`, or `hard-link` (default: `copy`)
- `--dry-run`: Print planned operations without touching the filesystem
- `--follow-symlinks`: Follow symbolic links while scanning
- `--layout <LAYOUT>`: Folder layout strategy (default: `patient-study-series`)
  - `patient-study-series`: Full hierarchy
  - `study-series`: Skip patient level
  - `series-only`: Only series folders
  - `flat`: All files in output root
- `--sort-by <STRATEGY>`: Sorting strategy within a series (default: `auto`)
  - `auto`: Use geometry if available, otherwise instance number
  - `instance`: Always use instance number
  - `geometry`: Always use geometric position
- `--include-phi`: Allow PHI fields (PatientName, descriptions) in folder names (default: off)
- `--report <FILE>`: Write JSON report with metadata

### Examples

**Dry-run to preview operations:**

```bash
dcmsort --input ./raw --output ./sorted --dry-run
```

**Copy files with default settings:**

```bash
dcmsort --input ./raw --output ./sorted
```

**Move files with geometry-based sorting:**

```bash
dcmsort --input ./raw --output ./sorted --mode move --sort-by geometry
```

**Generate metadata report:**

```bash
dcmsort --input ./raw --output ./sorted --report metadata.json
```

**Include PHI in folder names (use with caution):**

```bash
dcmsort --input ./raw --output ./sorted --include-phi
```

## Output Structure

Default layout (`patient-study-series`) without PHI:

```
output/
├── PATIENT_001/
│   └── 1.2.840.113619.../  (StudyInstanceUID)
│       └── 1.2.840.113619.../  (SeriesInstanceUID)
│           ├── 00001_1.2.840....dcm
│           ├── 00002_1.2.840....dcm
│           └── ...
```

With `--include-phi`:

```
output/
├── PATIENT_001_DOE_JOHN/
│   └── 20260117_CT_CHEST_1.2.840.../
│       └── CT_1_CHEST_ROUTINE_1.2.840.../
│           ├── 00001_1.2.840....dcm
│           └── ...
```

## Technical Details

### Dependencies

- **dicom-object 0.9**: DICOM parsing with header-only reading
- **clap 4.5**: CLI argument parsing
- **walkdir 2**: Recursive directory scanning
- **serde & serde_json**: JSON report generation
- **tracing**: Structured logging
- **rayon 1.11** (optional): Parallel processing

### Sorting Logic

1. **Grouping**: Files are grouped by (StudyInstanceUID, SeriesInstanceUID)
2. **Sorting within series**:
   - **Geometry mode**: Uses `dot(ImagePositionPatient, cross(row, col))` from ImageOrientationPatient
   - **Instance mode**: Uses InstanceNumber tag
   - **Auto mode**: Uses geometry if all images have required tags, otherwise falls back to instance number
3. **Tie-breaking**: SOPInstanceUID or filename if primary sort is equal

### Path Sanitization

- Removes Windows-unsafe characters: `<>:"/\|?*`
- Avoids reserved device names: `CON`, `PRN`, `AUX`, `NUL`, `COM1-4`, `LPT1-3`
- Limits component length to 80 characters
- Handles collision with automatic numbering

### Logging

Control verbosity with `RUST_LOG` environment variable:

```bash
# Windows PowerShell
$env:RUST_LOG="debug"; dcmsort --input ./raw --output ./sorted

# Linux/macOS
RUST_LOG=debug dcmsort --input ./raw --output ./sorted
```

## Safety Considerations

⚠️ **Medical Imaging Disclaimer**: This is a file organization tool, not a clinical diagnostic tool. Always validate sorted files before use in clinical workflows.

⚠️ **PHI Protection**: By default, `--include-phi` is OFF to prevent accidentally exposing Protected Health Information in folder names. Only enable if you understand the privacy implications.

⚠️ **Backup**: Always test with `--dry-run` first and maintain backups of original data.

## License

[Specify your license here]

## Contributing

[Specify contribution guidelines here]
