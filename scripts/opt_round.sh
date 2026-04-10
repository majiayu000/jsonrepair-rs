#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

usage() {
  cat <<'EOF'
Usage:
  scripts/opt_round.sh --baseline <name> [options]

Runs one optimization validation round:
1) cargo fmt
2) cargo check --all-targets
3) cargo clippy --all-targets --all-features -- -D warnings
4) cargo test --all
5) cargo bench --bench benchmark -- --baseline <name>
6) Parse Criterion change estimates and gate pass/fail

Options:
  --baseline <name>           Criterion baseline to compare against (required)
  --require-all-improved      Fail unless every benchmark is statistically improved
  --eps <float>               Noise epsilon for significance checks (default: 0)
  --save-report               Write markdown report under .omx/reports (default: on)
  --no-save-report            Do not write markdown report
  --skip-checks               Skip fmt/check/clippy/test and run bench+gate only
  --help                      Show this help

Exit codes:
  0  Gate passed
  2  Gate failed (regression or not all improved in strict mode)
  3  Missing or invalid benchmark artifacts
EOF
}

BASELINE=""
REQUIRE_ALL_IMPROVED=0
EPS="0"
SAVE_REPORT=1
SKIP_CHECKS=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --baseline)
      BASELINE="${2:-}"
      shift 2
      ;;
    --require-all-improved)
      REQUIRE_ALL_IMPROVED=1
      shift
      ;;
    --eps)
      EPS="${2:-}"
      shift 2
      ;;
    --save-report)
      SAVE_REPORT=1
      shift
      ;;
    --no-save-report)
      SAVE_REPORT=0
      shift
      ;;
    --skip-checks)
      SKIP_CHECKS=1
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ -z "$BASELINE" ]]; then
  echo "Error: --baseline is required" >&2
  usage >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "Error: jq is required but not found in PATH" >&2
  exit 3
fi

BENCH_NAMES=(
  valid_small
  broken_small
  valid_large_1k
  broken_large_1k
  nested_100
  comments_100
  string_escapes_200
)

if [[ "$SKIP_CHECKS" -eq 0 ]]; then
  echo "[opt-round] Running fmt/check/clippy/test..."
  cargo fmt
  cargo check --all-targets
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test --all
else
  echo "[opt-round] Skipping fmt/check/clippy/test (--skip-checks)"
fi

echo "[opt-round] Running benchmark against baseline: $BASELINE"
cargo bench --bench benchmark -- --baseline "$BASELINE"

timestamp="$(date '+%Y%m%d-%H%M%S')"
report_dir="$ROOT_DIR/.omx/reports"
report_path="$report_dir/opt-round-$timestamp.md"
mkdir -p "$report_dir"

improved_count=0
regressed_count=0
unchanged_count=0
rows=()

for bench in "${BENCH_NAMES[@]}"; do
  estimates="target/criterion/$bench/change/estimates.json"
  if [[ ! -f "$estimates" ]]; then
    echo "Error: missing Criterion change estimates: $estimates" >&2
    exit 3
  fi

  # Mean change ratio: negative is faster, positive is slower.
  point="$(jq -r '.mean.point_estimate' "$estimates")"
  lower="$(jq -r '.mean.confidence_interval.lower_bound' "$estimates")"
  upper="$(jq -r '.mean.confidence_interval.upper_bound' "$estimates")"

  classification="$(
    awk -v lo="$lower" -v hi="$upper" -v eps="$EPS" 'BEGIN {
      if (lo > eps) print "regressed";
      else if (hi < -eps) print "improved";
      else print "unchanged";
    }'
  )"

  case "$classification" in
    improved) ((improved_count+=1)) ;;
    regressed) ((regressed_count+=1)) ;;
    unchanged) ((unchanged_count+=1)) ;;
  esac

  point_pct="$(awk -v p="$point" 'BEGIN { printf "%.3f%%", p * 100 }')"
  lo_pct="$(awk -v p="$lower" 'BEGIN { printf "%.3f%%", p * 100 }')"
  hi_pct="$(awk -v p="$upper" 'BEGIN { printf "%.3f%%", p * 100 }')"

  rows+=("| $bench | $classification | $point_pct | [$lo_pct, $hi_pct] |")
done

gate_pass=1
gate_reason="no statistically significant regressions"

if [[ "$regressed_count" -gt 0 ]]; then
  gate_pass=0
  gate_reason="detected $regressed_count statistically significant regression(s)"
elif [[ "$REQUIRE_ALL_IMPROVED" -eq 1 && "$unchanged_count" -gt 0 ]]; then
  gate_pass=0
  gate_reason="strict mode requires all improved, but $unchanged_count benchmark(s) are unchanged"
fi

echo "[opt-round] Summary:"
echo "  improved : $improved_count"
echo "  regressed: $regressed_count"
echo "  unchanged: $unchanged_count"
echo "  gate     : $([[ "$gate_pass" -eq 1 ]] && echo PASS || echo FAIL) ($gate_reason)"

if [[ "$SAVE_REPORT" -eq 1 ]]; then
  {
    echo "# Optimization Round Report"
    echo
    echo "- Timestamp: $timestamp"
    echo "- Baseline: \`$BASELINE\`"
    echo "- Epsilon: \`$EPS\`"
    echo "- Strict mode: \`$REQUIRE_ALL_IMPROVED\`"
    echo
    echo "## Result"
    echo
    echo "- Gate: **$([[ "$gate_pass" -eq 1 ]] && echo PASS || echo FAIL)**"
    echo "- Reason: $gate_reason"
    echo
    echo "## Benchmark Changes"
    echo
    echo "| Benchmark | Status | Mean point estimate | Mean CI |"
    echo "| --- | --- | --- | --- |"
    printf '%s\n' "${rows[@]}"
  } > "$report_path"
  echo "[opt-round] Report written: $report_path"
fi

if [[ "$gate_pass" -eq 1 ]]; then
  exit 0
fi
exit 2
