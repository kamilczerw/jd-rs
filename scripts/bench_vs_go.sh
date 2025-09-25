#!/usr/bin/env bash
set -euo pipefail

TIME_CMD=""
if command -v /usr/bin/time >/dev/null; then
  TIME_CMD="/usr/bin/time"
elif command -v gtime >/dev/null; then
  TIME_CMD="$(command -v gtime)"
else
  echo "warning: /usr/bin/time not found; falling back to Python resource metrics" >&2
fi

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
TARGET_DIR="$REPO_ROOT/target/bench"
mkdir -p "$TARGET_DIR"

RUST_BIN="$REPO_ROOT/target/release/jd"
GO_BIN="$TARGET_DIR/jd-go"
FIXTURES_DIR="$REPO_ROOT/crates/jd-benches/fixtures"

cargo build --release -p jd-cli >/dev/null

go build -C "$REPO_ROOT/scripts" -o "$GO_BIN" github.com/josephburnett/jd/v2/jd >/dev/null

mapfile -t CORPORA < <(find "$FIXTURES_DIR" -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | sort)

printf "%-12s %-24s %-10s %-12s %-5s\n" "Binary" "Corpus" "Seconds" "MaxRSS(KB)" "Exit"
for corpus in "${CORPORA[@]}"; do
  before="$FIXTURES_DIR/$corpus/before.json"
  after="$FIXTURES_DIR/$corpus/after.json"
  if [[ ! -f "$before" || ! -f "$after" ]]; then
    echo "warning: skipping $corpus (missing before/after)" >&2
    continue
  fi

  for impl in rust go; do
    case "$impl" in
      rust)
        bin="$RUST_BIN"
        ;;
      go)
        bin="$GO_BIN"
        ;;
    esac

    metrics=$(mktemp)
    exit_code=0
    if [[ -n "$TIME_CMD" ]]; then
        if ! "$TIME_CMD" -f "%e %M" -o "$metrics" "$bin" "$before" "$after" >/dev/null 2>&1; then
          exit_code=$?
          if [[ $exit_code -ne 0 && $exit_code -ne 1 ]]; then
            cat "$metrics" >&2 || true
            rm -f "$metrics"
            echo "error: $bin failed on $corpus with exit $exit_code" >&2
            exit $exit_code
          fi
      fi
      read -r seconds maxrss <"$metrics"
    else
      if ! python3 - "$bin" "$before" "$after" >"$metrics" <<'PY'
import resource
import subprocess
import sys
import time

cmd = sys.argv[1:]
start = time.perf_counter()
proc = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
elapsed = time.perf_counter() - start
usage = resource.getrusage(resource.RUSAGE_CHILDREN)
print(f"{elapsed:.6f} {usage.ru_maxrss}")
sys.exit(proc.returncode)
PY
      then
        exit_code=$?
        if [[ $exit_code -ne 0 && $exit_code -ne 1 ]]; then
          cat "$metrics" >&2 || true
          rm -f "$metrics"
          echo "error: $bin failed on $corpus with exit $exit_code" >&2
          exit $exit_code
        fi
      fi
      read -r seconds maxrss <"$metrics"
    fi
    rm -f "$metrics"
    printf "%-12s %-24s %-10s %-12s %-5s\n" "$impl" "$corpus" "$seconds" "$maxrss" "$exit_code"
  done
  echo
done
