# Benchmark Refactor Script

## Overview

`benchmark_refactor.py` is a comprehensive benchmarking tool that compares `cargo build` and `cargo test` performance across different:
- **Branches**: main vs staging (to evaluate refactor impact)
- **Platforms**: Linux, macOS, Windows
- **Linkers**: default, lld, mold (where applicable)

## Features

- ✅ Automatic platform detection
- ✅ Multiple linker support (default, lld, mold)
- ✅ Branch comparison (main vs staging)
- ✅ Detailed timing and memory statistics
- ✅ JSON output for further analysis
- ✅ Human-readable reports with performance comparisons
- ✅ Automatic cleanup and branch restoration

### Platform Support Matrix

| Feature | Linux | macOS | Windows |
|---------|-------|-------|---------|
| Build timing ⏱️ | ✅ | ✅ | ✅ |
| Test timing ⏱️ | ✅ | ✅ | ✅ |
| Memory stats 💾 | ✅ | ✅ | ❌ |
| Default linker | ✅ | ✅ | ✅ |
| LLD linker | ✅ | ✅ | ✅ |
| Mold linker | ✅ | ❌ | ❌ |

## Prerequisites

### All Platforms
- Python 3.6+
- Rust/Cargo installed
- Git repository with `main` and `staging` branches

### Linux
```bash
sudo apt install time  # GNU time for memory stats
rustup component add llvm-tools-preview  # for lld
# For mold: https://github.com/rui314/mold#installation
```

### macOS
```bash
brew install gnu-time  # for memory stats
rustup component add llvm-tools-preview  # for lld
```

### Windows
```bash
rustup component add llvm-tools-preview  # for lld
# Note: Memory stats not available on Windows
# Note: Mold linker not available on Windows (Linux-only)
```

**Windows Limitations:**
- Memory statistics are not available (requires GNU time, which is Linux/macOS only)
- Mold linker is not available (Linux-only; Windows tests default and lld only)
- Timing measurements still work accurately using Python's built-in timing



## Usage

### Basic Usage
Run from anywhere in the project:
```bash
./scripts/benchmarks/benchmark_refactor.py
```

Or run directly with Python:
```bash
python3 scripts/benchmarks/benchmark_refactor.py
```

### What It Does

1. Detects your current platform (Linux/macOS/Windows)
2. Saves your current branch
3. For each branch (main, staging):
   - Checks out the branch
   - For each applicable linker (default, lld, mold):
     - Runs `cargo clean`
     - Runs `cargo build --release` (timed)
     - Runs `cargo test --release` (timed)
     - Captures memory usage (if GNU time available)
4. Restores your original branch
5. Generates results in `benchmark-results/`

### Output Files

Results are saved in `benchmark-results/` directory:

```
benchmark-results/
├── benchmark_Linux_20260214_143022.json      # Raw data
├── benchmark_Linux_20260214_143022.txt       # Human-readable report
├── benchmark_macOS_20260214_150133.json
├── benchmark_macOS_20260214_150133.txt
└── ...
```

### Example Output

```
Benchmark Results - Linux
Generated: 2026-02-14 14:30:22
================================================================================

Linker: DEFAULT
--------------------------------------------------------------------------------
Branch          Build Time      Test Time       Total Time      Build Mem   
--------------------------------------------------------------------------------
main            45.23s          120.45s         165.68s         2847MB      
staging         42.10s          115.20s         157.30s         2652MB      

Comparison (staging vs main):
  Build time:  -3.13s (-6.9%) 📉
  Test time:   -5.25s (-4.4%) 📉
  Total time:  -8.38s (-5.1%) 📉

Linker: LLD
--------------------------------------------------------------------------------
Branch          Build Time      Test Time       Total Time      Build Mem   
--------------------------------------------------------------------------------
main            38.92s          118.34s         157.26s         2801MB      
staging         36.45s          113.89s         150.34s         2598MB      

Comparison (staging vs main):
  Build time:  -2.47s (-6.3%) 📉
  Test time:   -4.45s (-3.8%) 📉
  Total time:  -6.92s (-4.4%) 📉
```

## Running on Multiple Platforms

To get comprehensive results across platforms:

### Option 1: GitHub Actions (Recommended)

A GitHub Actions workflow is already configured at `.github/workflows/benchmarks.yml`.

**To run benchmarks across all platforms:**

1. Go to your GitHub repository
2. Click on "Actions" tab
3. Select "Performance Benchmarks" workflow
4. Click "Run workflow" button
5. Optionally customize branches to compare
6. Click "Run workflow"

The workflow will:
- Run benchmarks on Linux, macOS, and Windows in parallel
- Install all necessary dependencies (GNU time, mold, etc.)
- Upload results as artifacts
- Generate a combined summary report
- Display results in the workflow summary

**Download results:**
- After the workflow completes, go to the workflow run page
- Scroll to "Artifacts" section
- Download individual platform results or the aggregated summary

**Benefits:**
- ✅ Runs on clean, consistent environments
- ✅ Parallel execution across all platforms
- ✅ No local machine setup required
- ✅ Results stored for 30 days
- ✅ Easy to compare across multiple runs

### Option 2: Local Runs
Run the script on each platform separately:

```bash
# On Linux machine
./scripts/benchmarks/benchmark_refactor.py

# On macOS machine
./scripts/benchmarks/benchmark_refactor.py

# On Windows machine
python scripts/benchmarks/benchmark_refactor.py
```

Then collect the `benchmark-results/` directory from each machine.

## Customization

### Customizing Branches (Local Script)

To modify branches or add configurations, edit the script:

```python
# Change branches to compare
branches = ["main", "staging"]  # Edit this line

# Or add more branches
branches = ["main", "staging", "develop"]
```

### Customizing GitHub Actions Workflow

The workflow supports runtime customization when manually triggered:

**branches**: Comma-separated list of branches to compare (default: `main,staging`)
**upload_results**: Whether to upload results as artifacts (default: `true`)

To customize the workflow itself, edit `.github/workflows/benchmarks.yml`:

```yaml
# Change default branches
inputs:
  branches:
    default: 'main,staging,feature-branch'

# Add more platforms
matrix:
  os: [ubuntu-latest, macos-latest, windows-latest, macos-13]

# Adjust artifact retention
retention-days: 30  # Change to desired number of days
```

## Troubleshooting

### "GNU time not found"
- **Linux**: `sudo apt install time`
- **macOS**: `brew install gnu-time`
- **Windows**: Memory stats not available, but timing still works

### "Failed to checkout branch"
- Ensure branches exist: `git branch -a`
- Commit or stash any uncommitted changes

### Linker not found
- Install lld: `rustup component add llvm-tools-preview`
- Install mold (Linux only): Follow [mold installation guide](https://github.com/rui314/mold#installation)

## Performance Tips

- Close other applications to get accurate measurements
- Run multiple times and average results for consistency
- Use `--release` builds (script does this automatically)
- Ensure sufficient disk space for build artifacts

## License

Same as parent project.
