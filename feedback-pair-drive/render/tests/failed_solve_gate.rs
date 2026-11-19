//! The render binary refuses to write a capture that contains held samples. This lives with the
//! Feedback Pair Drive because it needs an input that makes this Circuit's solve fail.

use std::process::Command;

#[test]
fn a_render_with_failed_solves_fails_and_writes_no_file() {
    // 2048 samples of a 1 kHz sine at 48 kHz with a normalized peak of 1e6 (5e6 V), Level 1.
    // With the production iteration bound some solves fail and are held. The render must then
    // exit with an error that gives the count, and must not create the output file.
    const RATE: u32 = 48_000;
    let samples: Vec<f32> = (0..2048)
        .map(|k| 1e6 * (std::f32::consts::TAU * 1000.0 * k as f32 / RATE as f32).sin())
        .collect();

    // The same processing as the binary does for a mono file, to establish the failure count.
    let mut processor = dsp::Processor::new(RATE as f64, &[1.0]);
    let mut left = samples.clone();
    left.extend(std::iter::repeat_n(0.0, processor.latency()));
    processor.process(&mut left, &mut []);
    let failed = processor.failed_solves();
    assert!(
        failed > 0,
        "this input no longer makes a solve fail, so it tests nothing"
    );

    let dir = std::env::temp_dir().join(format!("render-failed-solves-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (input, output) = (dir.join("in.wav"), dir.join("out.wav"));
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: RATE,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(&input, spec).unwrap();
    for &x in &samples {
        writer.write_sample(x).unwrap();
    }
    writer.finalize().unwrap();

    let run = Command::new(env!("CARGO_BIN_EXE_render"))
        .args([&input, &output])
        .arg("level=1")
        .output()
        .unwrap();
    let created = output.exists();
    std::fs::remove_dir_all(&dir).unwrap();
    let message = String::from_utf8_lossy(&run.stderr);
    assert!(!run.status.success(), "a render with held samples must fail");
    assert!(message.contains(&format!("failed on {failed} ")), "{message}");
    assert!(!created, "a failed render must not create the output file");
}
