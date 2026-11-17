# Skill / scaffold friction

- Background tool runs a login shell that sets `CARGO_TARGET_DIR=~/.cargo-target`; foreground tool does not. `install.sh` hard-codes `target/bundled` and fails after writing bundles to the shared directory. `validate.sh` directly runs `target/release/render` while the harness's `cargo run` can run the wrong shared render (our probe produced template GAIN/LEVEL and KeyError B63). Force `export CARGO_TARGET_DIR="$PWD/target"` inside the background shell for both validation and install. Install only after all validation passes. The incorrectly early-installed bundles were removed per parent request.
- The Rev 1 BOM calls out Alpha illuminated faders without value/taper, and the schematic also leaves them blank. Scaffold/skill should emphasize that transients can agree perfectly against SPICE with an unverified component value; hardware fidelity is still unresolved.
- Nix package `poppler_utils` was renamed to `poppler-utils`; the latter worked.
