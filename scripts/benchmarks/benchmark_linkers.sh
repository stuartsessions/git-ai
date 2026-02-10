#!/bin/bash

# Detect which 'time' utility to use
if command -v gtime >/dev/null 2>&1; then
    TIME_CMD="gtime -v"
elif [ -f /usr/bin/time ]; then
    TIME_CMD="/usr/bin/time -v"
else
    echo "Error: GNU time not found. Install with 'brew install gnu-time' (Mac) or 'apt install time' (Linux)."
    exit 1
fi

MODE=${1:-parallel}
JOBS_FLAG=""
[[ "$MODE" == "single" ]] && JOBS_FLAG="-j 1"

# Adjust flag based on OS (Mold is Linux-only, using LLD for Mac comparison)
if [[ "$OSTYPE" == "darwin"* ]]; then
    LINKERS=("Default:" "LLD:-C link-arg=-fuse-ld=lld")
    echo "OS Detected: macOS (Testing LLD instead of Mold)"
else
    LINKERS=("Default:" "Mold:-C link-arg=-fuse-ld=mold")
    echo "OS Detected: Linux"
fi

echo "Benchmarking Mode: $MODE"
echo "-----------------------------------------------------------------------"
printf "%-10s | %-12s | %-12s | %-12s\n" "Linker" "Build Time" "Test Time" "Peak RAM"
echo "-----------------------------------------------------------------------"

for entry in "${LINKERS[@]}"; do
    NAME="${entry%%:*}"
    FLAG="${entry#*:}"

    cargo clean > /dev/null 2>&1

    # Capture detailed stats
    # The '2>&1' is inside the subshell to ensure we grab the time output
    STATS=$($TIME_CMD env RUSTFLAGS="$FLAG" cargo build $JOBS_FLAG 2>&1)
    
    # Improved parsing for both Linux and Mac formats
    BUILD_TIME=$(echo "$STATS" | grep "Elapsed (wall clock)" | awk -F': ' '{print $2}')
    PEAK_KB=$(echo "$STATS" | grep "Maximum resident set size" | awk -F': ' '{print $2}')
    PEAK_MB=$((PEAK_KB / 1024))

    # Measure Test
    START_TEST=$(date +%s)
    RUSTFLAGS="$FLAG" cargo test $JOBS_FLAG > /dev/null 2>&1
    END_TEST=$(date +%s)
    TEST_TIME=$((END_TEST - START_TEST))

    printf "%-10s | %-12s | %-10ss | %-10sMB\n" "$NAME" "$BUILD_TIME" "$TEST_TIME" "$PEAK_MB"
done