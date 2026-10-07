#!/usr/bin/env bash
# Panel Commander cumulative build/regression/diagnostic evidence harness.
# Always packages exactly one timestamped evidence ZIP in ~/Downloads, even on failure.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAMP="$(date +%Y%m%d-%H%M%S)"
OUTDIR="${PANEL_COMMANDER_EVIDENCE_DIR:-$HOME/Downloads}"
NAME="Panel-Commander-BuildTest-${STAMP}"
TMP="$(mktemp -d)"
EVID="$TMP/$NAME"
ZIP="$OUTDIR/$NAME.zip"
CONNECTOR="${PANEL_COMMANDER_CONNECTOR:-card1-DP-2}"
RESULT=0
mkdir -p "$EVID/logs" "$EVID/diagnostics" "$OUTDIR"
: > "$EVID/GATES.txt"

cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT

record_result() {
    local name="$1" rc="$2"
    printf '%s=%s\n' "$name" "$rc" >> "$EVID/GATES.txt"
    if [[ "$rc" -ne 0 && "$RESULT" -eq 0 ]]; then RESULT="$rc"; fi
}

run_step() {
    local name="$1"; shift
    local log="$EVID/logs/${name}.log"
    echo "=== $name ===" | tee "$log"
    (cd "$ROOT" && "$@") 2>&1 | tee -a "$log"
    local rc=${PIPESTATUS[0]}
    echo "RESULT_CODE=$rc" | tee -a "$log"
    record_result "$name" "$rc"
    return 0
}

run_capture_step() {
    local name="$1" output="$2"; shift 2
    local log="$EVID/logs/${name}.log"
    echo "=== $name ===" | tee "$log"
    (cd "$ROOT" && "$@") >"$output" 2>&1
    local rc=$?
    cat "$output" | tee -a "$log"
    echo "RESULT_CODE=$rc" | tee -a "$log"
    record_result "$name" "$rc"
    return 0
}

capture() {
    local name="$1"; shift
    { "$@"; } >"$EVID/diagnostics/${name}.txt" 2>&1 || true
}

# Baseline provenance and environment.
capture date date --iso-8601=seconds
capture uname uname -a
capture rustc rustc -Vv
capture cargo cargo -Vv
capture git-status git -C "$ROOT" status --short --branch
capture git-log git -C "$ROOT" log -10 --oneline --decorate
capture git-diff git -C "$ROOT" diff --no-ext-diff
capture git-diff-cached git -C "$ROOT" diff --cached --no-ext-diff
capture cargo-metadata bash -lc "cd '$ROOT' && cargo metadata --format-version 1 --no-deps"
capture i2c-nodes bash -lc 'ls -l /dev/i2c-* 2>/dev/null || true'
capture i2c-adapters bash -lc 'for n in /sys/bus/i2c/devices/i2c-*/name; do [[ -r "$n" ]] && printf "%s: %s\n" "$(basename "$(dirname "$n")")" "$(cat "$n")"; done'
capture drm-connectors bash -lc 'for s in /sys/class/drm/card*-*/status; do [[ -r "$s" ]] || continue; d=$(dirname "$s"); printf "%s status=%s ddc=%s\n" "$(basename "$d")" "$(cat "$s")" "$(readlink -f "$d/ddc" 2>/dev/null || echo -)"; done'
capture dp-aux bash -lc 'for d in /sys/class/drm_dp_aux_dev/*; do [[ -e "$d" ]] || continue; printf "%s name=%s device=%s\n" "$(basename "$d")" "$(cat "$d/name" 2>/dev/null || echo -)" "$(readlink -f "$d/device" 2>/dev/null || echo -)"; done'

# Full regression gates. Continue after failures so the evidence archive is complete.
run_step diff-check git diff --check
run_step fmt cargo fmt --all -- --check
run_step test cargo test --workspace
run_step clippy cargo clippy --workspace --all-targets -- -D warnings
run_step build cargo build --workspace

# CLI diagnostics after successful/partial build.
if [[ -x "$ROOT/target/debug/panelctl" ]]; then
    run_step panelctl-list "$ROOT/target/debug/panelctl" list

    # Read-only hardware diagnostics if root access is already available/non-interactive.
    if [[ ${EUID:-$(id -u)} -eq 0 ]]; then
        run_step panelctl-probe "$ROOT/target/debug/panelctl" probe "$CONNECTOR"
        run_capture_step panelctl-capabilities "$EVID/diagnostics/${CONNECTOR}-capabilities.txt" \
            "$ROOT/target/debug/panelctl" capabilities "$CONNECTOR"
        run_step panelctl-snapshot "$ROOT/target/debug/panelctl" snapshot "$CONNECTOR" \
            "$EVID/diagnostics/${CONNECTOR}-snapshot.json"
    elif sudo -n true >/dev/null 2>&1; then
        run_step panelctl-probe sudo -n "$ROOT/target/debug/panelctl" probe "$CONNECTOR"
        run_capture_step panelctl-capabilities "$EVID/diagnostics/${CONNECTOR}-capabilities.txt" \
            sudo -n "$ROOT/target/debug/panelctl" capabilities "$CONNECTOR"
        run_step panelctl-snapshot sudo -n "$ROOT/target/debug/panelctl" snapshot "$CONNECTOR" \
            "$EVID/diagnostics/${CONNECTOR}-snapshot.json"
    else
        printf 'SKIPPED: read-only hardware DDC diagnostics require root. Run `sudo -v` immediately before this script.\n' \
            >"$EVID/diagnostics/hardware-ddc-skipped.txt"
        printf 'panelctl-probe=SKIPPED\npanelctl-capabilities=SKIPPED\npanelctl-snapshot=SKIPPED\n' \
            >> "$EVID/GATES.txt"
    fi
fi

cat >"$EVID/RESULT.txt" <<EOF2
PANEL_COMMANDER_BUILD_TEST=$RESULT
CONNECTOR=$CONNECTOR
TIMESTAMP=$STAMP
EOF2

# Package with Python so no external zip utility is required.
python3 - "$EVID" "$ZIP" <<'PY'
from pathlib import Path
import sys, zipfile
src=Path(sys.argv[1]); dst=Path(sys.argv[2])
with zipfile.ZipFile(dst, 'w', compression=zipfile.ZIP_DEFLATED) as z:
    for p in sorted(src.rglob('*')):
        if p.is_file(): z.write(p, p.relative_to(src.parent))
PY

SHA256="$(sha256sum "$ZIP" | awk '{print $1}')"
echo "SHA256=$SHA256"
echo "EVIDENCE_ZIP=$ZIP"
echo "FINAL_RESULT_CODE=$RESULT"
exit "$RESULT"
