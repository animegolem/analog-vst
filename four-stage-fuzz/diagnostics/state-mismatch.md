# Default harmonic discrepancy diagnostic

The full unmodified harness reported H2 13.88 dB error at 1 kHz/-40 dBFS, even as waveform null was -73.15 dB. It runs each SPICE test section from DC operating point, but the model receives the entire pack continuously. The immediately preceding 100 Hz high-level tests can leave long-lived feedback-capacitor charge.

Measured 50 ms windows, relative H2 (dBc):

| Time into tone | SPICE section | Model in full pack | Model isolated from rest |
|---:|---:|---:|---:|
| 0.05 s | -55.311 | -37.577 | -55.274 |
| 0.10 s | -55.154 | -38.260 | -55.132 |
| 0.20 s | -54.890 | -39.581 | -54.873 |
| 0.30 s | -54.671 | -40.854 | -54.656 |
| 0.40 s | -54.491 | -42.076 | -54.478 |

The isolated model used two seconds of initial zero signal followed by the same -40 dBFS 1 kHz tone (`isolated.wav`, `isolated-model.wav`). Its H2 agrees with SPICE within 0.04 dB in these windows. This strongly supports different prior state as the cause, rather than proving every harmonic is otherwise accurate. Do not claim the full validation passed: it did not. To validate continuous dynamics fairly, run SPICE continuously over the full pack or initialize both systems to the same per-section state; rerun all checks unchanged and assess the separately important continuous response. Do not simply skip the failing check or relax tolerances.
