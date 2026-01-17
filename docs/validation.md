# Validation Guide

## Purpose

This document describes how to validate that DICOM Sorter has correctly organized your files.

## Pre-Sorting Validation

### 1. Inventory Check

Before sorting, count your files:

```bash
# PowerShell
(Get-ChildItem -Path ./raw -Recurse -File).Count

# Linux/macOS
find ./raw -type f | wc -l
```

### 2. Sample Inspection

Manually inspect a few files to understand the dataset:
- How many patients?
- How many studies per patient?
- How many series per study?
- Are there non-DICOM files mixed in?

### 3. Dry-Run

**Always run with --dry-run first:**

```bash
dicom-sorter --input ./raw --output ./sorted --dry-run > plan.txt
```

Review `plan.txt` to check:
- Are files grouped correctly?
- Are paths reasonable?
- Any unexpected "UNKNOWN_*" folders?

## Post-Sorting Validation

### 1. File Count Verification

**Expected**: Number of DICOM files in output should match "Parsed X DICOM headers" log message.

```bash
# PowerShell
(Get-ChildItem -Path ./sorted -Recurse -File -Filter *.dcm).Count

# Linux/macOS
find ./sorted -type f -name "*.dcm" | wc -l
```

**Common discrepancies**:
- Non-DICOM files in input (expected to be skipped)
- Malformed DICOM files (logged as errors)

### 2. Metadata Report Validation

Generate a JSON report:

```bash
dicom-sorter --input ./raw --output ./sorted --report metadata.json
```

**Check**:
- Are all expected UIDs present?
- Are there many "UNKNOWN_*" values? (indicates missing tags)
- Do InstanceNumbers look sequential?

### 3. Visual Inspection (Medical Imaging Software)

Load sorted series into a DICOM viewer (e.g., 3D Slicer, Horos, OsiriX):

**Check**:
- Slices are in correct anatomical order
- No gaps or duplicates in the stack
- Image orientation is consistent

**Warning signs**:
- Slices appear out of order → geometry sorting may have failed
- Missing slices → files may have been lost or miscategorized
- Mixed orientations in one series → source data may be corrupted

### 4. Series Integrity Check

For each series, verify:

**Slice spacing consistency**:
- In geometry mode, spacing between slices should be uniform
- Large gaps indicate missing slices or incorrect grouping

**Instance number gaps**:
- If using instance mode, check for gaps in numbering
- Gaps may indicate missing files in source data

### 5. UID Uniqueness

**StudyInstanceUID**: Should be unique per study (same patient, same exam session)

**SeriesInstanceUID**: Should be unique per series (same study, same acquisition)

**SOPInstanceUID**: Should be globally unique per instance

**Validation**:
```bash
# Check for duplicate SOPInstanceUIDs (should be zero)
jq -r '.[].sop_uid' metadata.json | sort | uniq -d
```

## Common Failure Signals

### 1. All Files in "UNKNOWN_*" Folders

**Cause**: DICOM files are missing required UIDs

**Action**: 
- Check if files are valid DICOM
- Inspect with `dcmdump` or similar tool
- May need to use `--layout flat` if UIDs are unreliable

### 2. Geometry Sorting Produces Wrong Order

**Symptoms**: 
- Slices are shuffled when viewed in DICOM viewer
- Anatomical progression is incorrect

**Cause**: 
- ImagePositionPatient or ImageOrientationPatient tags are incorrect
- Mixed orientations in same series

**Action**:
- Use `--sort-by instance` to fall back to InstanceNumber
- Investigate source data quality

### 3. Files Missing After Sorting

**Symptoms**: Output has fewer files than input

**Cause**:
- Non-DICOM files in input (expected)
- Malformed DICOM files (check logs for errors)
- Collision handling created duplicates with `_1`, `_2` suffixes

**Action**:
- Review log output for errors
- Check for files with `_1`, `_2` suffixes (indicates collision)
- Verify source data integrity

### 4. Unexpected Grouping

**Symptoms**: 
- Files from different exams in same series folder
- Single series split across multiple folders

**Cause**:
- StudyInstanceUID or SeriesInstanceUID is missing or incorrect
- Source data has UID inconsistencies

**Action**:
- Generate metadata report and inspect UIDs
- May need manual intervention for non-compliant data

## Automated Validation Script (Example)

```python
#!/usr/bin/env python3
import json
import sys
from collections import Counter

with open('metadata.json') as f:
    metas = json.load(f)

print(f"Total files: {len(metas)}")

# Check for missing UIDs
missing_study = sum(1 for m in metas if not m.get('study_uid'))
missing_series = sum(1 for m in metas if not m.get('series_uid'))
missing_sop = sum(1 for m in metas if not m.get('sop_uid'))

print(f"Missing StudyInstanceUID: {missing_study}")
print(f"Missing SeriesInstanceUID: {missing_series}")
print(f"Missing SOPInstanceUID: {missing_sop}")

# Check for duplicate SOPInstanceUIDs
sop_uids = [m['sop_uid'] for m in metas if m.get('sop_uid')]
duplicates = [uid for uid, count in Counter(sop_uids).items() if count > 1]

if duplicates:
    print(f"ERROR: {len(duplicates)} duplicate SOPInstanceUIDs found!")
    sys.exit(1)
else:
    print("✓ All SOPInstanceUIDs are unique")

# Group by series
series_groups = {}
for m in metas:
    key = (m.get('study_uid'), m.get('series_uid'))
    series_groups.setdefault(key, []).append(m)

print(f"\nTotal series: {len(series_groups)}")

# Check instance numbers within series
for (study, series), items in series_groups.items():
    instance_nums = [m.get('instance_number') for m in items if m.get('instance_number')]
    if instance_nums:
        instance_nums.sort()
        gaps = [instance_nums[i+1] - instance_nums[i] for i in range(len(instance_nums)-1)]
        if any(g > 1 for g in gaps):
            print(f"WARNING: Series {series} has gaps in InstanceNumber")

print("\n✓ Validation complete")
```

## Best Practices

1. **Always dry-run first**: Catch issues before modifying files
2. **Generate metadata report**: Enables programmatic validation
3. **Keep backups**: Original data should be preserved
4. **Validate in DICOM viewer**: Visual inspection is critical for medical imaging
5. **Document anomalies**: Note any "UNKNOWN_*" folders or warnings for future reference

## When to Suspect Data Quality Issues

- More than 10% of files in "UNKNOWN_*" folders
- Duplicate SOPInstanceUIDs
- Large gaps in InstanceNumber sequences
- Mixed modalities in same series (e.g., CT and MR)
- Inconsistent slice spacing (>20% variation)

These often indicate problems with the source data, not the sorting tool.
