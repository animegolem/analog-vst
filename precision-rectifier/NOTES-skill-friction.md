# Skill/template friction

Each row: what happened, the evidence, and the check that would have caught it.

- **The reference runner's output is not band-limited before it is sampled.** `spice.py` reads
  ngspice's output at `fs * OVERSAMPLE` (384 kHz) with no filter in front, so what the circuit puts
  above 192 kHz folds back. For a rectifier the reference at 4x differs from the reference at 24x
  by 0.83 dB in the frequency-response measure (1 s, 0 dBFS sweep). The docs say the runner "adds
  under 0.002 dB of its own error", which holds only for circuits whose harmonics fall fast.
  Evidence: `diagnostics/sweep_resolution.py`. Check: render one 0 dBFS sweep at two factors and
  refuse when they differ by more than a fraction of the Tolerance.
- **The Model's circuit rate has the same limit**, and `MIN_CIRCUIT_RATE` is documented only for
  frequency warping. On a 1 s, 0 dBFS sweep against the reference at 16x, 576 kHz was 0.40 dB
  off on this circuit, 1152 kHz 0.079 dB and 2304 kHz 0.013 dB. Check: the same sweep at the
  chosen rate and at twice it.
- **The Solver's diode is not ngspice's diode in reverse bias.** ngspice uses
  `-Is (1 + (3 N Vt / (e v))^3)` below -3 N Vt and adds GMIN (1e-12 S) across the junction. The
  template uses the exponential law everywhere and no GMIN. With megohm-scale impedances and the
  template's large `IS` (2.52 nA) this is microvolts: 0.0275 dB of the gain measure at 100 Hz,
  -40 dBFS here. Fixed for `Device::Diode` in this Project only; `DiodePair` and the transistors
  still differ. Check: a DC sweep of one reverse-biased diode through a 1 Mohm resistor, Model
  against ngspice.
- **The template's diode line has `M=0.4 tt=20n`, which the Solver cannot represent.** The
  Netlist and the Model then describe different diodes (2.3 pF to 12.5 pF against a fixed 4 pF).
  Here the Netlist uses `M=0 tt=0`. Check: compare ngspice's reported junction capacitance over
  the working voltage range with the Model's `cjo`.
- **`analysis.align` trusts the sync chirp for any circuit.** For a circuit that removes the
  fundamental it locks onto the second-harmonic image (delay -1446 samples, polarity -1). Both
  captures shift alike, but the comparison does not survive: every Section's window moves 15 ms,
  so the burst null stops 15 ms before the burst ends (a 0.1 full-scale error added to the
  Model's last 5 ms of the burst passed every Check; in the right window it fails at -34.0 dB),
  and the frequency-response value changes (0.0307 dB against 0.0054 dB on the same captures).
  With a different section order it locked onto a sweep instead (delay 81572). Both captures the
  harness makes have a known delay of zero, and this Project's harness now compares them there
  (`--alignment known`); chirp recovery is kept for captures of unknown delay
  (`--alignment chirp`). Check: `test_known_alignment_keeps_every_section_window_where_the_pack_puts_it`
  in `harness/tests/test_harness.py`.
- **The template's reference cache key holds only the Netlist and the control values.** A change
  to `spice.py` (here `OVERSAMPLE` 4 to 16) or to the Pack reused the old reference. This Project
  now keys on the Netlist, `spice.py`, both Pack files and the ngspice version, as green-overdrive
  does (`reference-key.sh`). Check: `test_reference_key_follows_the_runner_and_the_pack`.
- **The harmonics plot is clipped at 0 dBc.** This circuit's second harmonic reads about +41 dBc.
  Check-less (presentation).
- **`oversampling_filter` gives a delay that is not a whole base sample at 48 kHz** (189 taps at
  4x: 94 high-rate samples is 23.5 base samples). The Pack is at 96 kHz, where the delay is whole
  at 4, 8, 16 and 24, so validation is not affected; the harness's own RC test runs at 48 kHz and
  checks magnitude only. Check: assert `(taps - 1) // 2 % OVERSAMPLE == 0`.
- **`new_project.py` leaves a nested `.git` in the new Project**, which has to be removed before
  the Project can be committed to a repository that holds several Projects. Check-less.
- **An empty `CONTROLS` list builds and runs** (`render --plain` prints only the full-scale
  line; the plugin hides all eight slots). `model::pots` then receives an empty slice. This was
  untested upstream; a template test with no Controls would keep it working.
