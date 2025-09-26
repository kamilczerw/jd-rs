#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
DATASET_DIR="$REPO_ROOT/docs/parity/upstream/jd-v2.2.2"

if [[ ! -d "$DATASET_DIR" ]]; then
  echo "parity dataset not found: $DATASET_DIR" >&2
  exit 1
fi

if [[ -n "${JD_PARITY_BIN:-}" ]]; then
  JD_BIN="$JD_PARITY_BIN"
else
  echo "Building jd CLI binary (release profile)..." >&2
  cargo build --quiet --release -p jd-cli
  JD_BIN="$REPO_ROOT/target/release/jd"
fi

if [[ ! -x "$JD_BIN" ]]; then
  echo "jd binary not found or not executable: $JD_BIN" >&2
  exit 1
fi

tmp_root=$(mktemp -d -t jd-parity.XXXXXX)
trap 'rm -rf "$tmp_root"' EXIT

declare -a failures=()

declare -A stdout_expectations=(
  [color-output]=diff.color
  [default-nested-structures]=diff.jd
  [default-object]=diff.jd
  [format-merge]=diff.merge.json
  [format-patch]=diff.patch
  [precision]=diff.jd
  [precision-array]=diff.jd
  [yaml]=diff.jd
)

declare -A file_expectations=(
  [output-flag]=diff.jd
  [output-flag-dash-filename]=-
  [output-flag-format-merge]=diff.merge
  [output-flag-format-patch]=diff.patch
  [output-flag-yaml]=diff.jd
)

declare -A expected_failures=(
  [arrays-multiset]="-mset is not implemented yet"
  [arrays-multiset-nested]="-mset is not implemented yet"
  [arrays-set]="-set is not implemented yet"
  [arrays-setkeys]="-setkeys is not implemented yet"
  [arrays-setkeys-nested]="-setkeys is not implemented yet"
  [output-flag-patch-mode]="Patch mode is not implemented yet"
  [patch-mode]="Patch mode is not implemented yet"
  [output-flag-translate-jd2patch]="Translate mode is not implemented yet"
  [output-flag-translate-patch2jd]="Translate mode is not implemented yet"
  [translate-jd2patch]="Translate mode is not implemented yet"
  [translate-patch2jd]="Translate mode is not implemented yet"
)

run_stdout() {
  local scenario="$1"
  local cmd="$2"
  local expected_rel="$3"
  local actual_file
  actual_file=$(mktemp)

  local status=0
  if bash -c "$cmd" >"$actual_file"; then
    status=0
  else
    status=$?
  fi
  if [[ $status -ne 0 && $status -ne 1 ]]; then
    failures+=("$scenario: command failed (exit $status)")
    echo "[FAIL] $scenario: command exited with status $status" >&2
    rm -f "$actual_file"
    return
  fi

  local expected_path="$DATASET_DIR/$scenario/$expected_rel"
  local expect_diff=0
  if [[ -s "$expected_path" ]]; then
    expect_diff=1
  fi
  if [[ $expect_diff -ne 0 && $status -ne 1 ]]; then
    failures+=("$scenario: expected exit 1 for diff, got $status")
    echo "[FAIL] $scenario: expected exit 1 for diff output" >&2
  fi

  if ! diff -u "$expected_path" "$actual_file" >"$actual_file.diff"; then
    failures+=("$scenario: stdout mismatch")
    echo "[FAIL] $scenario: output differed from upstream" >&2
    cat "$actual_file.diff" >&2
  else
    echo "[OK]   $scenario" >&2
  fi

  rm -f "$actual_file" "$actual_file.diff"
}

run_file_output() {
  local scenario="$1"
  local workdir="$2"
  local cmd="$3"
  local expected_rel="$4"

  local output_path="$workdir/$expected_rel"
  rm -f "$output_path"

  local status=0
  if bash -c "$cmd"; then
    status=0
  else
    status=$?
  fi
  if [[ $status -ne 0 && $status -ne 1 ]]; then
    failures+=("$scenario: command failed (exit $status)")
    echo "[FAIL] $scenario: command exited with status $status" >&2
    return
  fi

  if [[ ! -f "$output_path" ]]; then
    failures+=("$scenario: expected file missing ($expected_rel)")
    echo "[FAIL] $scenario: expected file not produced ($expected_rel)" >&2
    return
  fi

  if ! diff -u "$DATASET_DIR/$scenario/$expected_rel" "$output_path"; then
    failures+=("$scenario: file mismatch ($expected_rel)")
    echo "[FAIL] $scenario: $expected_rel differed from upstream" >&2
  else
    echo "[OK]   $scenario" >&2
  fi
}

run_expected_failure() {
  local scenario="$1"
  local cmd="$2"
  local expected_message="$3"
  local stderr_file
  stderr_file=$(mktemp)

  local status=0
  if bash -c "$cmd" > /dev/null 2>"$stderr_file"; then
    status=0
  else
    status=$?
  fi
  if [[ $status -eq 0 ]]; then
    failures+=("$scenario: expected failure but command succeeded")
    echo "[FAIL] $scenario: expected failure but command succeeded" >&2
    rm -f "$stderr_file"
    return
  fi

  if ! grep -Fq -- "$expected_message" "$stderr_file"; then
    failures+=("$scenario: error message mismatch")
    echo "[FAIL] $scenario: error message differed" >&2
    echo "--- stderr ---" >&2
    cat "$stderr_file" >&2
    echo "--------------" >&2
  else
    echo "[OK]   $scenario (expected failure)" >&2
  fi

  rm -f "$stderr_file"
}

for scenario_path in "$DATASET_DIR"/*; do
  [[ -d "$scenario_path" ]] || continue
  scenario=$(basename "$scenario_path")
  command_file="$scenario_path/command.txt"
  if [[ ! -f "$command_file" ]]; then
    continue
  fi

  echo "Running scenario: $scenario" >&2

  workdir="$tmp_root/$scenario"
  mkdir -p "$workdir"
  cp -R "$scenario_path/." "$workdir"

  cmd=$(grep -v '^#' "$command_file" | sed -e '/^$/d')
  if [[ -z "$cmd" ]]; then
    echo "[WARN] $scenario: no command found" >&2
    continue
  fi

  cmd=${cmd//\/tmp\/jd/$JD_BIN}

  pushd "$workdir" >/dev/null
  if [[ -n "${stdout_expectations[$scenario]:-}" ]]; then
    run_stdout "$scenario" "$cmd" "${stdout_expectations[$scenario]}"
  elif [[ -n "${file_expectations[$scenario]:-}" ]]; then
    run_file_output "$scenario" "$workdir" "$cmd" "${file_expectations[$scenario]}"
  elif [[ -n "${expected_failures[$scenario]:-}" ]]; then
    run_expected_failure "$scenario" "$cmd" "${expected_failures[$scenario]}"
  else
    echo "[WARN] $scenario: no expectation mapping, skipping" >&2
  fi
  popd >/dev/null
  echo >&2

done

if (( ${#failures[@]} > 0 )); then
  echo "Parity check failed for ${#failures[@]} scenario(s):" >&2
  for failure in "${failures[@]}"; do
    echo "  - $failure" >&2
  done
  exit 1
fi

echo "All parity scenarios succeeded" >&2
