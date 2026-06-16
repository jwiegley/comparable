#!/usr/bin/env bash
# Run the test suite under cargo-llvm-cov and fail if total line coverage drops
# below the committed baseline in .coverage-baseline.
#
# Regenerate the baseline with `make coverage-baseline`.
set -euo pipefail

cd "$(dirname "$0")/.."

baseline_file=".coverage-baseline"
if [ ! -f "$baseline_file" ]; then
	echo "missing $baseline_file; create one with 'make coverage-baseline'" >&2
	exit 1
fi
baseline=$(tr -d '[:space:]' <"$baseline_file")

# Collect coverage, then export a JSON summary and read total line coverage.
cargo llvm-cov --workspace --no-report
current=$(cargo llvm-cov report --json | jq '.data[0].totals.lines.percent')

printf 'baseline line coverage: %s%%\n' "$baseline"
printf 'current  line coverage: %s%%\n' "$current"

# Fail only on a real drop; allow a small epsilon for rounding jitter.
if awk -v c="$current" -v b="$baseline" 'BEGIN { exit !(c + 0.05 < b) }'; then
	printf 'ERROR: coverage dropped from %s%% to %s%%\n' "$baseline" "$current" >&2
	exit 1
fi
printf 'coverage OK (>= baseline)\n'
