#!/usr/bin/env bash
# Prints the part of a SPICE reference's cache key that is the same for every Setting. Run from
# the Project directory. A reference capture depends on the Netlist, the SPICE runner, the exact
# Pack and the ngspice version; validate.sh adds the Setting's control values and simulates again
# when the key differs from the one stored beside the capture.
set -euo pipefail
inputs_hash=$(shasum reference/circuit.cir harness/src/analog_harness/spice.py validation/pack/pack.json validation/pack/pack.wav | shasum | cut -d' ' -f1)
spice_version=$(ngspice --version 2>/dev/null | tr '\n' ' ' || true)
echo "$inputs_hash $spice_version"
