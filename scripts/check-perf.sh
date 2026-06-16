#!/usr/bin/env bash
# Compare the most recent criterion run against a saved local baseline and fail
# if any benchmark's median time regressed by more than PERF_THRESHOLD percent
# (default 5).
#
#     scripts/check-perf.sh          compare against the saved baseline
#     scripts/check-perf.sh --save   save the current run as the baseline
#
# Run `cargo bench` first so that the criterion results exist.  The baseline
# lives under the target directory and is therefore hardware-specific and not
# committed; CI uses scripts/ci-perf.sh to compare base vs. head instead.
set -euo pipefail

cd "$(dirname "$0")/.."

threshold=${PERF_THRESHOLD:-5}
target_dir=${CARGO_TARGET_DIR:-target}
criterion_dir="$target_dir/criterion"
baseline="$target_dir/perf-baseline.json"

if [ ! -d "$criterion_dir" ]; then
	echo "no criterion results in $criterion_dir; run 'cargo bench' first" >&2
	exit 1
fi

# Emit {benchmark_id: median_ns} for every benchmark in the latest run.
collect() {
	local est id
	while IFS= read -r est; do
		id=${est#"$criterion_dir"/}
		id=${id%/new/estimates.json}
		jq -n --arg id "$id" \
			--argjson v "$(jq '.median.point_estimate' "$est")" \
			'{($id): $v}'
	done < <(find "$criterion_dir" -path '*/new/estimates.json' | sort)
}

current=$(collect | jq -s 'add // {}')

if [ "${1:-}" = "--save" ]; then
	echo "$current" >"$baseline"
	echo "saved performance baseline to $baseline"
	exit 0
fi

if [ ! -f "$baseline" ]; then
	echo "no baseline at $baseline; create one with 'make perf-baseline'" >&2
	exit 1
fi

report=$(jq -n \
	--slurpfile base "$baseline" \
	--argjson cur "$current" '
	($base[0]) as $b
	| $cur | to_entries
	| map({
		name: .key,
		base: ($b[.key] // null),
		cur: .value,
		pct: (if ($b[.key] // null) == null then null
		      else ((.value - $b[.key]) / $b[.key] * 100) end)
	  })')

echo "$report" | jq -r '.[]
	| if .base == null then "  \(.name): NEW (no baseline)"
	  else "  \(.name): \(.base | floor)ns -> \(.cur | floor)ns (\(.pct * 10 | round / 10)%)"
	  end'

regressions=$(echo "$report" | jq --argjson t "$threshold" \
	'[.[] | select(.pct != null and .pct > $t)] | length')

if [ "$regressions" -gt 0 ]; then
	echo "performance regression > ${threshold}% detected" >&2
	exit 1
fi
echo "performance OK (no regression > ${threshold}%)"
