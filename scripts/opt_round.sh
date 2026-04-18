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
4) cargo test --all-targets
5) cargo bench --bench benchmark -- --baseline <name> (re-run on detected regressions)
6) Aggregate Criterion change estimates and gate pass/fail

Options:
  --baseline <name>           Criterion baseline to compare against (required)
  --require-all-improved      Fail unless every benchmark is statistically improved
  --eps <float>               Noise epsilon for significance checks (default: 0)
  --reruns-on-regression <n>  Extra benchmark reruns after a regressed first pass (default: 2)
  --save-report               Write markdown report under .omx/reports (default: on)
  --no-save-report            Do not write markdown report
  --skip-checks               Skip fmt/check/clippy/test and run bench+gate only
  --help                      Show this help

Exit codes:
  0  Gate passed
  2  Gate failed (regression or not all improved in strict mode)
  3  Missing or invalid benchmark artifacts
  4  Benchmark environment unstable; result inconclusive
EOF
}

BASELINE=""
REQUIRE_ALL_IMPROVED=0
EPS="0"
RERUNS_ON_REGRESSION=2
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
    --reruns-on-regression)
      RERUNS_ON_REGRESSION="${2:-}"
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

if ! [[ "$RERUNS_ON_REGRESSION" =~ ^[0-9]+$ ]]; then
  echo "Error: --reruns-on-regression must be a non-negative integer" >&2
  exit 1
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
  cargo test --all-targets
else
  echo "[opt-round] Skipping fmt/check/clippy/test (--skip-checks)"
fi

timestamp="$(date '+%Y%m%d-%H%M%S')"
report_dir="$ROOT_DIR/.omx/reports"
report_path="$report_dir/opt-round-$timestamp.md"
mkdir -p "$report_dir"

run_report_sections=()
bench_improved_runs=()
bench_regressed_runs=()
bench_unchanged_runs=()
bench_last_point_pct=()
bench_last_ci_pct=()
total_bench_runs=0
initial_regressions=0

reset_series_state() {
  run_report_sections=()
  bench_improved_runs=()
  bench_regressed_runs=()
  bench_unchanged_runs=()
  bench_last_point_pct=()
  bench_last_ci_pct=()
  total_bench_runs=0

  for idx in "${!BENCH_NAMES[@]}"; do
    bench_improved_runs[$idx]=0
    bench_regressed_runs[$idx]=0
    bench_unchanged_runs[$idx]=0
    bench_last_point_pct[$idx]=""
    bench_last_ci_pct[$idx]=""
  done
}

reset_series_state

classify_change() {
  local lower="$1"
  local upper="$2"
  awk -v lo="$lower" -v hi="$upper" -v eps="$EPS" 'BEGIN {
    if (lo > eps) print "regressed";
    else if (hi < -eps) print "improved";
    else print "unchanged";
  }'
}

run_bench_pass() {
  local run_number="$1"
  local run_improved=0
  local run_regressed=0
  local run_unchanged=0
  local run_rows=()

  echo "[opt-round] Running $SERIES_NAME pass $run_number against baseline: $SERIES_BASELINE"
  cargo bench --bench benchmark -- --baseline "$SERIES_BASELINE"

  for idx in "${!BENCH_NAMES[@]}"; do
    bench="${BENCH_NAMES[$idx]}"
    estimates="target/criterion/$bench/change/estimates.json"
    if [[ ! -f "$estimates" ]]; then
      echo "Error: missing Criterion change estimates: $estimates" >&2
      exit 3
    fi

    point="$(jq -r '.mean.point_estimate' "$estimates")"
    lower="$(jq -r '.mean.confidence_interval.lower_bound' "$estimates")"
    upper="$(jq -r '.mean.confidence_interval.upper_bound' "$estimates")"
    classification="$(classify_change "$lower" "$upper")"

    case "$classification" in
      improved)
        ((run_improved+=1))
        ((bench_improved_runs[$idx]+=1))
        ;;
      regressed)
        ((run_regressed+=1))
        ((bench_regressed_runs[$idx]+=1))
        ;;
      unchanged)
        ((run_unchanged+=1))
        ((bench_unchanged_runs[$idx]+=1))
        ;;
    esac

    point_pct="$(awk -v p="$point" 'BEGIN { printf "%.3f%%", p * 100 }')"
    lo_pct="$(awk -v p="$lower" 'BEGIN { printf "%.3f%%", p * 100 }')"
    hi_pct="$(awk -v p="$upper" 'BEGIN { printf "%.3f%%", p * 100 }')"
    bench_last_point_pct[$idx]="$point_pct"
    bench_last_ci_pct[$idx]="[$lo_pct, $hi_pct]"
    run_rows+=("| $bench | $classification | $point_pct | [$lo_pct, $hi_pct] |")
  done

  run_report_sections+=("### $SERIES_NAME Pass $run_number")
  run_report_sections+=("")
  run_report_sections+=("| Benchmark | Status | Mean point estimate | Mean CI |")
  run_report_sections+=("| --- | --- | --- | --- |")
  for row in "${run_rows[@]}"; do
    run_report_sections+=("$row")
  done
  run_report_sections+=("")

  total_bench_runs=$((total_bench_runs + 1))
  CURRENT_RUN_IMPROVED="$run_improved"
  CURRENT_RUN_REGRESSED="$run_regressed"
  CURRENT_RUN_UNCHANGED="$run_unchanged"
}

finalize_series() {
  improved_count=0
  regressed_count=0
  unchanged_count=0
  rows=()

  for idx in "${!BENCH_NAMES[@]}"; do
    bench="${BENCH_NAMES[$idx]}"
    improved_runs="${bench_improved_runs[$idx]}"
    regressed_runs="${bench_regressed_runs[$idx]}"
    unchanged_runs="${bench_unchanged_runs[$idx]}"

    if (( regressed_runs * 2 > total_bench_runs )); then
      classification="regressed"
      ((regressed_count+=1))
    elif (( improved_runs * 2 > total_bench_runs )); then
      classification="improved"
      ((improved_count+=1))
    else
      classification="unchanged"
      ((unchanged_count+=1))
    fi

    rows+=("| $bench | $classification | ${improved_runs}/${regressed_runs}/${unchanged_runs} | ${bench_last_point_pct[$idx]} | ${bench_last_ci_pct[$idx]} |")
  done
}

run_bench_series() {
  reset_series_state
  run_bench_pass 1
  initial_regressions="$CURRENT_RUN_REGRESSED"

  if [[ "$initial_regressions" -gt 0 && "$RERUNS_ON_REGRESSION" -gt 0 ]]; then
    echo "[opt-round] Detected regressions on ${SERIES_NAME,,} pass 1; running $RERUNS_ON_REGRESSION additional pass(es) to filter benchmark noise"
    rerun=0
    while [[ "$rerun" -lt "$RERUNS_ON_REGRESSION" ]]; do
      rerun=$((rerun + 1))
      run_bench_pass "$((rerun + 1))"
    done
  fi

  finalize_series
}

SERIES_NAME="Baseline Comparison"
SERIES_BASELINE="$BASELINE"
run_bench_series

primary_improved_count="$improved_count"
primary_regressed_count="$regressed_count"
primary_unchanged_count="$unchanged_count"
primary_total_bench_runs="$total_bench_runs"
primary_rows=("${rows[@]}")
primary_run_report_sections=("${run_report_sections[@]}")

control_enabled=0
control_improved_count=0
control_regressed_count=0
control_unchanged_count=0
control_total_bench_runs=0
control_rows=()
control_run_report_sections=()
control_baseline=""

gate_pass=1
gate_exit_code=0
gate_reason="no statistically significant regressions"

if [[ "$primary_regressed_count" -gt 0 ]]; then
  control_enabled=1
  control_baseline="control-$timestamp"
  echo "[opt-round] Running control self-check with fresh baseline: $control_baseline"
  cargo bench --bench benchmark -- --save-baseline "$control_baseline"

  SERIES_NAME="Control Self-Check"
  SERIES_BASELINE="$control_baseline"
  run_bench_series

  control_improved_count="$improved_count"
  control_regressed_count="$regressed_count"
  control_unchanged_count="$unchanged_count"
  control_total_bench_runs="$total_bench_runs"
  control_rows=("${rows[@]}")
  control_run_report_sections=("${run_report_sections[@]}")

  if [[ $((control_improved_count + control_regressed_count)) -gt 0 ]]; then
    gate_pass=0
    gate_exit_code=4
    gate_reason="benchmark environment unstable: fresh control baseline still produced $((control_improved_count + control_regressed_count)) stable non-unchanged benchmark(s) across $control_total_bench_runs pass(es)"
  else
    gate_pass=0
    gate_exit_code=2
    gate_reason="detected $primary_regressed_count stable regression(s) across $primary_total_bench_runs benchmark pass(es)"
  fi
elif [[ "$REQUIRE_ALL_IMPROVED" -eq 1 && "$primary_unchanged_count" -gt 0 ]]; then
  gate_pass=0
  gate_exit_code=2
  gate_reason="strict mode requires all stable improvements, but $primary_unchanged_count benchmark(s) are unchanged across $primary_total_bench_runs benchmark pass(es)"
elif [[ "$primary_total_bench_runs" -gt 1 ]]; then
  gate_reason="no stable regressions across $primary_total_bench_runs benchmark pass(es)"
fi

echo "[opt-round] Summary:"
echo "  benchmark passes: $primary_total_bench_runs"
echo "  improved : $primary_improved_count"
echo "  regressed: $primary_regressed_count"
echo "  unchanged: $primary_unchanged_count"
if [[ "$control_enabled" -eq 1 ]]; then
  echo "  control benchmark passes: $control_total_bench_runs"
  echo "  control stable non-unchanged: $((control_improved_count + control_regressed_count))"
fi
echo "  gate     : $([[ "$gate_pass" -eq 1 ]] && echo PASS || echo FAIL) ($gate_reason)"

if [[ "$SAVE_REPORT" -eq 1 ]]; then
  {
    echo "# Optimization Round Report"
    echo
    echo "- Timestamp: $timestamp"
    echo "- Baseline: \`$BASELINE\`"
    echo "- Epsilon: \`$EPS\`"
    echo "- Strict mode: \`$REQUIRE_ALL_IMPROVED\`"
    echo "- Reruns on regression: \`$RERUNS_ON_REGRESSION\`"
    echo "- Benchmark passes executed: \`$primary_total_bench_runs\`"
    if [[ "$control_enabled" -eq 1 ]]; then
      echo "- Control baseline: \`$control_baseline\`"
      echo "- Control benchmark passes executed: \`$control_total_bench_runs\`"
    fi
    echo
    echo "## Result"
    echo
    echo "- Gate: **$([[ "$gate_pass" -eq 1 ]] && echo PASS || echo FAIL)**"
    echo "- Reason: $gate_reason"
    echo
    echo "## Stable Benchmark Changes"
    echo
    echo "| Benchmark | Stable status | Run votes (I/R/U) | Last mean point estimate | Last mean CI |"
    echo "| --- | --- | --- | --- | --- |"
    printf '%s\n' "${primary_rows[@]}"
    if [[ "$control_enabled" -eq 1 ]]; then
      echo
      echo "## Control Self-Check"
      echo
      echo "| Benchmark | Stable status | Run votes (I/R/U) | Last mean point estimate | Last mean CI |"
      echo "| --- | --- | --- | --- | --- |"
      printf '%s\n' "${control_rows[@]}"
    fi
    if [[ "${#primary_run_report_sections[@]}" -gt 0 || "${#control_run_report_sections[@]}" -gt 0 ]]; then
      echo
      echo "## Per-Pass Benchmark Details"
      echo
      if [[ "${#primary_run_report_sections[@]}" -gt 0 ]]; then
        printf '%s\n' "${primary_run_report_sections[@]}"
      fi
      if [[ "${#control_run_report_sections[@]}" -gt 0 ]]; then
        printf '%s\n' "${control_run_report_sections[@]}"
      fi
    fi
  } > "$report_path"
  echo "[opt-round] Report written: $report_path"
fi

if [[ "$gate_pass" -eq 1 ]]; then
  exit 0
fi
exit "$gate_exit_code"
