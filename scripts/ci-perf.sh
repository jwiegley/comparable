#!/usr/bin/env bash
# CI performance gate.
#
# Benchmarks the base revision and the current checkout on the *same* machine
# (so the numbers are comparable) and fails if any benchmark's median time
# regressed by more than PERF_THRESHOLD percent (default 5).
#
#     scripts/ci-perf.sh [BASE_REF]
#
# BASE_REF defaults to origin/main.  Micro-benchmarks of pure functions are
# fairly stable even on shared runners, but if this proves noisy, raise
# PERF_THRESHOLD.
set -euo pipefail

cd "$(dirname "$0")/.."

threshold=${PERF_THRESHOLD:-5}
base_ref=${1:-origin/main}
target_dir=${CARGO_TARGET_DIR:-target}
export CRITERION_HOME="$PWD/$target_dir/criterion"

git fetch --no-tags --depth=1 origin "${base_ref#origin/}" 2>/dev/null || true

worktree=$(mktemp -d)
cleanup() { git worktree remove --force "$worktree" 2>/dev/null || true; }
trap cleanup EXIT

git worktree add --detach --force "$worktree" "$base_ref"

echo "==> benchmarking base ($base_ref)"
(
	cd "$worktree"
	CRITERION_HOME="$CRITERION_HOME" cargo bench --bench comparison -- --save-baseline base --noplot
) || echo "base revision has no 'comparison' benchmark; treating all benchmarks as new"

echo "==> benchmarking HEAD"
cargo bench --bench comparison -- --save-baseline head --noplot

echo "==> comparing"
fail=0
while IFS= read -r est; do
	id=${est#"$CRITERION_HOME"/}
	id=${id%/head/estimates.json}
	base_est="$CRITERION_HOME/$id/base/estimates.json"
	if [ ! -f "$base_est" ]; then
		echo "  $id: NEW (no base)"
		continue
	fi
	b=$(jq '.median.point_estimate' "$base_est")
	h=$(jq '.median.point_estimate' "$est")
	pct=$(jq -n --argjson b "$b" --argjson h "$h" '(($h - $b) / $b * 100) * 10 | round / 10')
	worse=$(jq -n --argjson b "$b" --argjson h "$h" --argjson t "$threshold" '($h > $b * (1 + $t / 100))')
	printf '  %s: %.0fns -> %.0fns (%s%%)\n' "$id" "$b" "$h" "$pct"
	if [ "$worse" = "true" ]; then
		echo "    REGRESSION > ${threshold}%"
		fail=1
	fi
done < <(find "$CRITERION_HOME" -path '*/head/estimates.json' | sort)

if [ "$fail" -ne 0 ]; then
	echo "performance regression detected" >&2
	exit 1
fi
echo "performance OK (no regression > ${threshold}%)"
