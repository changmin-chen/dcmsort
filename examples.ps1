# DICOM Sorter - Quick Start Examples

# Example 1: Dry-run to preview operations
# This is ALWAYS recommended as the first step
.\target\release\dicom-sorter.exe `
    --input "C:\path\to\raw\dicom\files" `
    --output "C:\path\to\sorted\output" `
    --dry-run

# Example 2: Copy files with default settings (Patient/Study/Series hierarchy)
.\target\release\dicom-sorter.exe `
    --input "C:\path\to\raw\dicom\files" `
    --output "C:\path\to\sorted\output"

# Example 3: Move files instead of copying (saves disk space)
.\target\release\dicom-sorter.exe `
    --input "C:\path\to\raw\dicom\files" `
    --output "C:\path\to\sorted\output" `
    --mode move

# Example 4: Use geometry-based sorting (recommended for CT/MR volumes)
.\target\release\dicom-sorter.exe `
    --input "C:\path\to\raw\dicom\files" `
    --output "C:\path\to\sorted\output" `
    --sort-by geometry

# Example 5: Generate metadata report for validation
.\target\release\dicom-sorter.exe `
    --input "C:\path\to\raw\dicom\files" `
    --output "C:\path\to\sorted\output" `
    --report "metadata_report.json"

# Example 6: Simpler folder structure (Study/Series only)
.\target\release\dicom-sorter.exe `
    --input "C:\path\to\raw\dicom\files" `
    --output "C:\path\to\sorted\output" `
    --layout study-series

# Example 7: Include PHI in folder names (use with caution!)
# WARNING: This will expose patient names and descriptions in folder paths
.\target\release\dicom-sorter.exe `
    --input "C:\path\to\raw\dicom\files" `
    --output "C:\path\to\sorted\output" `
    --include-phi

# Example 8: Enable debug logging
$env:RUST_LOG="debug"
.\target\release\dicom-sorter.exe `
    --input "C:\path\to\raw\dicom\files" `
    --output "C:\path\to\sorted\output"

# Example 9: Complete workflow with validation
# Step 1: Dry-run
.\target\release\dicom-sorter.exe `
    --input ".\raw" `
    --output ".\sorted" `
    --dry-run | Out-File "plan.txt"

# Step 2: Review plan.txt, then execute
.\target\release\dicom-sorter.exe `
    --input ".\raw" `
    --output ".\sorted" `
    --report "metadata.json"

# Step 3: Validate file count
Write-Host "Input files: $((Get-ChildItem -Path .\raw -Recurse -File).Count)"
Write-Host "Output files: $((Get-ChildItem -Path .\sorted -Recurse -File -Filter *.dcm).Count)"

# Step 4: Check metadata report
Get-Content "metadata.json" | ConvertFrom-Json | Measure-Object
