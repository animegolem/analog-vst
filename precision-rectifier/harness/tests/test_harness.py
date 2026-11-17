import json
import shutil
import subprocess
from pathlib import Path

import numpy as np
import pytest
from scipy import signal as sps

from analog_harness import analysis, compare, signals, spice

FS = 48_000


@pytest.fixture(scope="module")
def pack():
    return signals.build(FS)


@pytest.fixture(scope="module")
def identity(pack):
    stim, manifest = pack
    return analysis.analyze(stim, stim, manifest, analysis.CHIRP)


def test_identity_measures_flat_and_clean(identity):
    assert identity["delay_samples"] == 0 and identity["polarity"] == 1
    for s in identity["sweeps"]:
        assert np.max(np.abs(s["db"])) < 1e-3
    for s in identity["steps"]:
        assert abs(s["gain_db"]) < 1e-3
        assert max(s["harmonics_dbc"].values()) < -100
    assert identity["bursts"][0]["attack_s"] is None


def test_frequency_response_matches_analytic_filter(pack):
    stim, manifest = pack
    sos = sps.butter(2, 2000, fs=FS, output="sos")
    sweep = analysis.analyze(stim, sps.sosfilt(sos, stim), manifest, analysis.CHIRP)["sweeps"][0]
    _, h = sps.sosfreqz(sos, worN=sweep["freqs"], fs=FS)
    expected = 20 * np.log10(np.abs(h))
    usable = expected > -40
    assert np.max(np.abs(np.array(sweep["db"])[usable] - expected[usable])) < 0.05


def test_harmonics_match_polynomial_theory(pack):
    stim, manifest = pack
    a2, a3 = 0.05, 0.02
    steps = analysis.analyze(stim, stim + a2 * stim**2 + a3 * stim**3, manifest, analysis.CHIRP)["steps"]
    s = next(s for s in steps if s["freq"] == 1000 and s["level_db"] == -4)
    amp = signals.db(-4)
    fundamental = amp + 0.75 * a3 * amp**3
    assert s["harmonics_dbc"][2] == pytest.approx(20 * np.log10(a2 * amp**2 / 2 / fundamental), abs=0.01)
    assert s["harmonics_dbc"][3] == pytest.approx(20 * np.log10(a3 * amp**3 / 4 / fundamental), abs=0.01)


def test_attack_and_release_times(pack):
    stim, manifest = pack
    sec = next(s for s in manifest["sections"] if s["kind"] == "burst")
    t1, t2, _ = (np.cumsum(sec["segments_s"]) * FS).astype(int)
    tau_a, tau_r = 0.005, 0.05
    t = np.arange(sec["length"]) / FS
    g = np.ones(sec["length"])
    g[t1:t2] = 0.5 + 0.5 * np.exp(-t[: t2 - t1] / tau_a)
    g[t2:] = 1 - (1 - g[t2 - 1]) * np.exp(-t[: len(g) - t2] / tau_r)
    gain = np.ones_like(stim)
    gain[sec["start"] : sec["start"] + sec["length"]] = g

    # The 90% point of an exponential in dB, for a 6 dB change, is close to (not exactly) tau*ln(10).
    burst = analysis.analyze(stim, stim * gain, manifest, analysis.CHIRP)["bursts"][0]
    db_curve = 20 * np.log10(g)
    target_a = db_curve[t1 - 1] + 0.9 * (db_curve[t2 - 1] - db_curve[t1 - 1])
    expected_a = np.argmax(db_curve[t1:t2] <= target_a) / FS
    target_r = db_curve[t2 - 1] + 0.9 * (db_curve[-1] - db_curve[t2 - 1])
    expected_r = np.argmax(db_curve[t2:] >= target_r) / FS
    assert burst["attack_s"] == pytest.approx(expected_a, rel=0.03)
    assert burst["release_s"] == pytest.approx(expected_r, rel=0.03)


def test_latency_and_polarity_are_recovered(pack):
    stim, manifest = pack
    result = analysis.analyze(stim, -np.concatenate([np.zeros(37), stim]), manifest, analysis.CHIRP)
    assert (result["delay_samples"], result["polarity"]) == (37, -1)
    assert np.max(np.abs(result["sweeps"][2]["db"])) < 1e-3


def test_compare_passes_identity_and_flags_small_errors(pack, tmp_path):
    stim, manifest = pack
    assert compare.run(stim, manifest, stim, stim, tmp_path / "same", analysis.CHIRP)
    assert (tmp_path / "same" / "frequency_response.png").exists()

    assert not compare.run(stim, manifest, stim, stim * signals.db(0.2), tmp_path / "gain", analysis.CHIRP)
    failed = {
        c["name"]
        for c in compare.checks(
            analysis.analyze(stim, stim, manifest, analysis.CHIRP),
            analysis.analyze(stim, np.tanh(3 * stim) / 3, manifest, analysis.CHIRP),
            -100,
        )
        if not c["passed"]
    }
    assert {"harmonics", "static gain"} <= failed


def test_known_alignment_keeps_every_section_window_where_the_pack_puts_it(pack, tmp_path):
    """A defect in the last 5 ms of the burst is found, for a Circuit that defeats the sync chirp.

    A rectifier removes the fundamental. Chirp recovery then locks onto the second-harmonic image,
    15 ms early with the sign flipped, for the reference and the model alike. With that shift the
    burst window ends 15 ms before the burst does and the defect goes unseen.
    """
    stim, manifest = pack
    reference = np.abs(stim)
    sec = next(s for s in manifest["sections"] if s["kind"] == "burst")
    end = sec["start"] + sec["length"]
    model = reference.copy()
    model[end - int(0.005 * FS) : end] += 0.1

    recovered = analysis.place(stim, reference, manifest, analysis.CHIRP)[1:]
    assert recovered[0] < -int(0.005 * FS) and recovered[1] == -1, "the rectified pack no longer defeats the chirp"

    placed, delay, polarity = analysis.place(stim, model, manifest, analysis.KNOWN)
    assert placed is model and (delay, polarity) == (0, 1)
    assert compare.run(stim, manifest, reference, reference, tmp_path / "same", analysis.KNOWN)
    assert not compare.run(stim, manifest, reference, model, tmp_path / "tail", analysis.KNOWN)
    report = json.loads((tmp_path / "tail" / "report.json").read_text())
    assert [c["name"] for c in report["checks"] if not c["passed"]] == ["burst waveform null"]
    for side in ("reference", "model"):
        assert (report[side]["delay_samples"], report[side]["polarity"]) == (0, 1)
    assert "delay 0 samples, polarity +1" in (tmp_path / "tail" / "report.md").read_text()


def test_alignment_is_a_stated_choice(pack):
    stim, manifest = pack
    with pytest.raises(TypeError):
        analysis.analyze(stim, stim, manifest)
    with pytest.raises(AssertionError):
        analysis.analyze(stim, stim, manifest, "auto")
    # A capture that is not on the pack's time axis cannot be called aligned.
    with pytest.raises(ValueError):
        analysis.analyze(stim, np.concatenate([np.zeros(37), stim]), manifest, analysis.KNOWN)


def test_reference_key_follows_the_runner_and_the_pack(tmp_path):
    """validate.sh reuses a SPICE reference only while reference-key.sh prints the same key."""
    script = Path(__file__).resolve().parents[2] / "reference-key.sh"
    inputs = [
        "reference/circuit.cir",
        "harness/src/analog_harness/spice.py",
        "validation/pack/pack.json",
        "validation/pack/pack.wav",
    ]
    for name in inputs:
        (tmp_path / name).parent.mkdir(parents=True, exist_ok=True)
        (tmp_path / name).write_text(f"first {name}\n")

    def key() -> str:
        return subprocess.run(["bash", str(script)], cwd=tmp_path, check=True, capture_output=True, text=True).stdout

    first = key()
    assert first.strip() and key() == first
    for name in inputs:
        original = (tmp_path / name).read_text()
        (tmp_path / name).write_text(original + "changed\n")
        assert key() != first, f"a change to {name} left the key unchanged"
        (tmp_path / name).write_text(original)
        assert key() == first
    (tmp_path / inputs[-1]).unlink()
    assert subprocess.run(["bash", str(script)], cwd=tmp_path, check=False, capture_output=True).returncode != 0


@pytest.mark.skipif(not shutil.which("ngspice"), reason="needs ngspice")
def test_spice_reference_matches_analytic_rc(tmp_path):
    """An RC low-pass through the SPICE runner matches its analytic response."""
    netlist = tmp_path / "rc.cir"
    netlist.write_text("* RC\n$SOURCE\nR1 in out $R\nC1 out 0 10n\n$ANALYSIS\n.end\n")
    sweep = signals.sweep(FS, 10, 0.45 * FS, 0.5)
    gap = np.zeros(FS // 2)
    stim = np.concatenate([gap, sweep, gap])
    manifest = {
        "sample_rate": FS,
        "sections": [{"kind": "sweep", "start": len(gap), "length": len(sweep), "f1": 10, "f2": 0.45 * FS}],
    }
    y = spice.render(netlist, stim, manifest, {"R": 1000})
    fr = analysis.frequency_response(stim, y, manifest["sections"][0], FS)
    f = np.array(fr["freqs"])
    expected = -10 * np.log10(1 + (2 * np.pi * f * 1000 * 10e-9) ** 2)
    assert np.max(np.abs(np.array(fr["db"]) - expected)) < 0.01
