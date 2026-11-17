#!/usr/bin/env bash
# Validates the model against its SPICE reference at every knob setting in settings.txt
# (or the file named by $SETTINGS). QUICK=1 uses a shorter test pack (for CI); VERBOSE=1 shows
# the harness's output.
# Writes validation/<setting>/report.md (with plots) and validation/summary.md.
# Exits non-zero if any check fails. SPICE references are reused while the netlist is unchanged.
set -euo pipefail
cd "$(dirname "$0")"
# Build into ./target even if the user exports a shared CARGO_TARGET_DIR: the scripts and harness
# run ./target/release/render and copy ./target/bundled.
export CARGO_TARGET_DIR="$PWD/target"

for tool in cargo uv ngspice; do
  command -v "$tool" >/dev/null || { echo "missing $tool: see GUIDE.md, step 1" >&2; exit 2; }
done

ngspice_major=$(ngspice --version 2>/dev/null | sed -n 's/.*ngspice-\([0-9][0-9]*\).*/\1/p' | head -1)
if [[ -n "$ngspice_major" && "$ngspice_major" -lt 47 ]]; then
  echo "warning: ngspice $ngspice_major found; use 47 or newer (older versions can be over 1000x slower here)" >&2
fi

cargo build --quiet --release -p render
mkdir -p validation
[ -f validation/pack/pack.json ] || (cd harness && uv run --quiet harness signals ${QUICK:+--quick} ../validation/pack >/dev/null)

# Reference capture depends on the SPICE runner and exact pack as well as the netlist.
netlist_hash=$(shasum reference/circuit.cir harness/src/analog_harness/spice.py validation/pack/pack.json validation/pack/pack.wav | shasum | cut -d' ' -f1)
spice_version=$(ngspice --version 2>/dev/null | tr '\n' ' ')
quiet=/dev/null
[[ -n "${VERBOSE:-}" ]] && quiet=/dev/stderr
status=0
summary="| Setting | Result |"$'\n'"|---|---|"
# Settings come in on fd 3 so commands in the loop can't consume them from stdin.
while read -r -u 3 name controls; do
  [[ -z "$name" || "$name" == \#* ]] && continue
  dir="validation/$name"
  mkdir -p "$dir"
  # shellcheck disable=SC2086  # controls is a space-separated list of arguments
  key="$netlist_hash $spice_version $(./target/release/render --plain $controls | tr '\n' ' ')"
  if [[ ! -f "$dir/reference.wav" || "$(cat "$dir/reference.key" 2>/dev/null)" != "$key" ]]; then
    echo "== $name: simulating the circuit in SPICE (a few minutes)"
    # shellcheck disable=SC2086
    (cd harness && uv run --quiet harness spice ../reference/circuit.cir ../validation/pack/pack.json $controls -o "../$dir/reference.wav" >"$quiet")
    echo "$key" > "$dir/reference.key"
  fi
  echo "== $name: rendering the model and comparing"
  # shellcheck disable=SC2086
  (cd harness && uv run --quiet harness model-sections ../validation/pack/pack.json $controls -o "../$dir/model.wav")
  if (cd harness && uv run --quiet harness compare ../validation/pack/pack.json "../$dir/reference.wav" "../$dir/model.wav" -o "../$dir"); then
    summary+=$'\n'"| $name | PASS |"
  else
    summary+=$'\n'"| $name | FAIL |"
    status=1
  fi
done 3< "${SETTINGS:-settings.txt}"

printf '%s\n' "$summary" | tee validation/summary.md
exit $status
