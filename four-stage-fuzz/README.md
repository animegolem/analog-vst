# Four Stage Sustainer

A four-transistor distortion plugin modeled after the American V3 Big Muff Pi circuit in ElectroSmash's analysis, not other versions. Trademarks belong to their respective owners. No schematic or graphics are redistributed here.

## Source and scope

- ElectroSmash, **Big Muff Pi Analysis**, V3 complete labeled circuit diagram and parts discussion, archived from 2022: https://electrosmash.mas-effects.com/big-muff-pi-analysis.html . The diagram is authoritative where its article text conflicts with it (including R11 10k and R6 15k; R7 is 430k). Every value and node is transcribed explicitly in `reference/circuit.cir`; see its header.
- Q1–Q4 use the reduced 2N5088 model cited by the sibling MSK-011 project, from Fairchild/onsemi https://www.onsemi.com/pdf/datasheet/2n5088-d.pdf . The source analysis explicitly lists 2N5088 as a BC239 substitute. The model retains IS, BF, BR, VAF, IKF, RB, RC, RE; RE defaults to zero. Junction charge storage and leakage are omitted in **both** implementations. That is a substantive approximation: actual junction capacitances, Miller effect, charge storage and low-current beta can change frequency response, dynamics and distortion. The vendor's zero-bias CJC is 4.017 pF versus each explicit 470 pF feedback capacitor (about 0.85% before bias dependence); CJE is 4.973 pF. This simple comparison is not an error or audibility measurement: junction charge, saturation and the uncompensated recovery stage can matter differently. No quantitative estimate of audibility or hardware error has been measured.
- The 1N914 pairs use the template's 1N4148 equivalent; the fixed diode capacitance approximation differs from SPICE's voltage-dependent capacitance. No hardware calibration is claimed.
- Ideal 9 V supply; input source impedance 1k, output load 1M; full-scale input corresponds to 0.2 V peak, typical guitar level. Bypass switch, LED, supply decoupling, noise and component tolerances omitted. Pot minimum resistance is clamped to 1 ohm. Knobs are 100k linear; clockwise tone increases the high-pass branch.
- Tests validate this **reduced model against the matching SPICE model**, not against a physical pedal or full transistor models. A full validation passes at all 7 settings, including volume at 0 (2026-11-17). Validation renders each test section from a fresh DC state, as SPICE does; `diagnostics/state-mismatch.md` records why.

## CPU warning

Measured 337% of one core for one stereo instance at 48 kHz (2026-11-17, circuit rate 672 kHz, one benchmark at a time). The plugin is for offline bounce or freeze only.

The complete coupled nonlinear circuit has an exceptionally high CPU cost. At the first benchmark it used approximately 1028% of one core for one stereo instance at 48 kHz while other work was running; this measurement is not a real-time-performance guarantee and is likely too costly for practical live use.

## Build and license

Run `cargo test`, `cd harness && uv run pytest -q`, `./validate.sh`, and `./install.sh` as appropriate. The scaffold's VST3 bindings use GPL-licensed code; audit all dependency and distribution terms before releasing either format. Source circuit diagram is cited for factual component values, not licensed for redistribution.
