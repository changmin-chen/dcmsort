# Design Document: DICOM Sorter

## Overview

This document describes the design decisions, sorting rules, layout strategies, and PHI (Protected Health Information) policies for the DICOM Sorter application.

## Core Design Principles

### 1. Memory Efficiency

**Problem**: DICOM files can contain large pixel data (hundreds of MB per file). Loading thousands of files into memory would cause RAM exhaustion.

**Solution**: Use `OpenFileOptions::read_until(PIXEL_DATA)` to stop reading before the Pixel Data tag (7FE0,0010). This converts the problem from "RAM explosion" to "I/O intensive but manageable."

### 2. Reliability Over Features

**Approach**: 
- Use stable, well-maintained crates (dicom-object 0.9, clap 4.5, walkdir 2)
- Avoid experimental features (DICOM network, multi-frame splitting, anonymization)
- Focus on single-frame CT/MR images (most common use case)
- Treat enhanced multi-frame as "one file = one instance"

### 3. PHI Safety by Default

**Policy**: Protected Health Information should NOT appear in folder names by default.

**Rationale**: Organizing files by patient name or study description makes PHI more discoverable and increases privacy risk.

**Implementation**: `--include-phi` flag is OFF by default. When enabled, folder names can include:
- PatientName
- StudyDescription
- SeriesDescription

## Sorting Rules

### Hierarchical Grouping

Files are grouped in a three-level hierarchy:

1. **Patient Level**: PatientID (or "UNKNOWN_PATIENT")
2. **Study Level**: StudyInstanceUID (or "UNKNOWN_STUDY")
3. **Series Level**: SeriesInstanceUID (or "UNKNOWN_SERIES")

### Sorting Within a Series

Three strategies are available:

#### Auto Mode (Default)

```
if all images have ImagePositionPatient + ImageOrientationPatient:
    use geometry-based sorting
else:
    use instance number sorting
```

**Rationale**: Geometry is more reliable for 3D volumes (CT, MR), but not all DICOM files have these tags (e.g., secondary captures, reports).

#### Geometry Mode

**Formula**: 
```
normal = cross(row_direction, col_direction)
order = dot(ImagePositionPatient, normal)
```

**Use case**: CT/MR axial/sagittal/coronal slices where spatial position is critical.

**Fallback**: If geometry tags are missing, falls back to InstanceNumber, then SOPInstanceUID.

#### Instance Mode

**Formula**: Sort by InstanceNumber tag.

**Use case**: When you trust the InstanceNumber is correct, or geometry is not relevant.

**Fallback**: If InstanceNumber is missing, uses SOPInstanceUID (lexicographic).

### Tie-Breaking

When primary sort criteria are equal:
1. Try InstanceNumber (if not already used)
2. Use SOPInstanceUID (guaranteed unique per instance)
3. If SOPInstanceUID is missing (non-compliant DICOM), use filename

## Layout Strategies

### PatientStudySeries (Default)

```
output/
├── {PatientID}/
│   └── {StudyInstanceUID}/
│       └── {SeriesInstanceUID}/
│           ├── 00001_{SOPInstanceUID}.dcm
│           ├── 00002_{SOPInstanceUID}.dcm
│           └── ...
```

**Use case**: Full organization for multi-patient datasets.

### StudySeries

```
output/
├── {StudyInstanceUID}/
│   └── {SeriesInstanceUID}/
│       ├── 00001_{SOPInstanceUID}.dcm
│       └── ...
```

**Use case**: Single-patient datasets or when PatientID is unreliable.

### SeriesOnly

```
output/
├── {SeriesInstanceUID}/
│   ├── 00001_{SOPInstanceUID}.dcm
│   └── ...
```

**Use case**: Single study, multiple series.

### Flat

```
output/
├── 00001_{SOPInstanceUID}.dcm
├── 00002_{SOPInstanceUID}.dcm
└── ...
```

**Use case**: Simple renaming/numbering without hierarchy.

## PHI Policy

### Default Behavior (--include-phi OFF)

Folder names use only:
- PatientID (not PatientName)
- StudyInstanceUID (not StudyDescription)
- SeriesInstanceUID (not SeriesDescription)
- Modality and SeriesNumber (not descriptive text)

**Example**:
```
PATIENT_001/1.2.840.113619.2.55.../1.2.840.113619.2.55.../
```

### With --include-phi

Folder names include descriptive fields:

**Example**:
```
PATIENT_001_DOE_JOHN/20260117_CT_CHEST_1.2.840.../CT_1_CHEST_ROUTINE_1.2.840.../
```

**Warning**: This makes PHI more visible in file paths, logs, and backups. Only use in secure environments.

## File Operations

### Copy (Default)

- Safest option
- Preserves original files
- Requires 2x storage space

### Move

- Saves storage space
- Modifies source directory
- Uses `fs::rename` with fallback to copy+delete for cross-device moves

### HardLink

- Saves storage space (same inode)
- Preserves original directory structure
- Only works on same filesystem
- Falls back to copy if hardlink fails

### Collision Handling

If destination file exists:
1. Append `_1`, `_2`, etc. to filename stem
2. Try up to 10,000 variations
3. If all fail, use original path (will error on write)

## Error Handling

### Non-DICOM Files

- Silently skipped during metadata extraction
- Counted in "files found" but not in "headers parsed"

### Missing Tags

- Use fallback values: "UNKNOWN_PATIENT", "UNKNOWN_STUDY", etc.
- Still processable, but may group incorrectly

### Malformed DICOM

- Logged as error
- Skipped from processing
- Reported in final count

## Performance Considerations

### Single-threaded (Default)

- Simple, predictable
- Suitable for <10,000 files

### Parallel (--features parallel)

- Uses rayon for parallel metadata extraction
- 2-4x faster on multi-core systems
- Suitable for >10,000 files
- Slightly higher memory usage

### I/O Bottleneck

- Most time spent in disk I/O, not CPU
- SSD vs HDD makes bigger difference than parallelization
- Network drives are significantly slower

## Validation Strategy

See `validation.md` for detailed validation procedures.

## Future Considerations (Out of Scope for MVP)

- DICOMDIR parsing
- Enhanced multi-frame splitting
- Transfer syntax conversion
- Anonymization pipeline
- DICOM network (C-FIND/C-MOVE)
- Incremental updates (cache metadata)

These features would significantly increase complexity and are better suited for specialized tools or future iterations.
