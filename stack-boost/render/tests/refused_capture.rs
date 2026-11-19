//! The render binary refuses a capture that contains a held sample or a sample that is not
//! finite: it exits non-zero with the counts, leaves no file at the output path (an earlier one
//! is removed), and never removes its input. Shared by every Project: the input is far enough
//! over full scale that every Model either fails solves or overflows the 32-bit output.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const RATE: u32 = 48_000;
const SAMPLES: usize = 128;

/// A 1 kHz sine with the largest finite peak a 32-bit float file can hold.
fn overload() -> Vec<f32> {
    (0..SAMPLES)
        .map(|k| f32::MAX * (std::f32::consts::TAU * 1000.0 * k as f32 / RATE as f32).sin())
        .collect()
}

/// Every Control at the top of its travel, as render arguments and as rotations.
fn controls_at_top() -> (Vec<String>, Vec<f32>) {
    let names = dsp::model::CONTROLS
        .iter()
        .map(|c| format!("{}=1", c.name.to_lowercase()));
    (names.collect(), vec![1.0; dsp::model::CONTROLS.len()])
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("render-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_wav(path: &Path, samples: &[f32]) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: RATE,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(path, spec).unwrap();
    for &x in samples {
        writer.write_sample(x).unwrap();
    }
    writer.finalize().unwrap();
}

fn render(input: &Path, output: &Path, controls: &[String]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_render"))
        .args([input, output])
        .args(controls)
        .output()
        .unwrap()
}

/// What the binary will count for the overload input as a mono file: held circuit-rate samples
/// and non-finite output samples. Computed once: every held sample runs its solve to the bound.
fn expected_counts() -> (u64, usize) {
    static COUNTS: std::sync::OnceLock<(u64, usize)> = std::sync::OnceLock::new();
    *COUNTS.get_or_init(|| {
        let mut processor = dsp::Processor::new(RATE as f64, &controls_at_top().1);
        let mut left = overload();
        left.extend(std::iter::repeat_n(0.0, processor.latency()));
        processor.process(&mut left, &mut []);
        let counts = (
            processor.failed_solves(),
            left.iter().filter(|y| !y.is_finite()).count(),
        );
        println!(
            "{}: {} held samples, {} non-finite output samples",
            dsp::model::NAME,
            counts.0,
            counts.1
        );
        counts
    })
}

/// Asserts the refusal's exit status and message for the overload input.
fn assert_refused(run: &Output) {
    let (failed, non_finite) = expected_counts();
    assert!(
        failed > 0 || non_finite > 0,
        "this input is no longer refused, so it tests nothing"
    );
    let message = String::from_utf8_lossy(&run.stderr);
    assert!(!run.status.success(), "a refused capture must exit non-zero: {message}");
    if failed > 0 {
        assert!(message.contains(&format!("failed on {failed} ")), "{message}");
    }
    if non_finite > 0 {
        assert!(
            message.contains(&format!("{non_finite} output samples are not finite")),
            "{message}"
        );
    }
    assert!(message.contains("no output written"), "{message}");
}

#[test]
fn a_refused_capture_removes_the_file_already_at_the_output_path() {
    let dir = scratch("refused");
    let (input, output, bystander) = (dir.join("in.wav"), dir.join("out.wav"), dir.join("other.wav"));
    write_wav(&input, &overload());
    // An earlier capture at the output path, and a file beside it that is none of render's.
    write_wav(&output, &[0.25; 16]);
    write_wav(&bystander, &[0.5; 16]);
    let run = render(&input, &output, &controls_at_top().0);
    let left = (input.exists(), output.exists(), bystander.exists());
    std::fs::remove_dir_all(&dir).unwrap();
    assert_refused(&run);
    assert_eq!(
        left,
        (true, false, true),
        "input kept, earlier capture removed, other file kept"
    );
    assert!(String::from_utf8_lossy(&run.stderr).contains("removed the earlier file"));
}

#[test]
fn a_refused_capture_creates_no_file() {
    let dir = scratch("refused-fresh");
    let (input, output) = (dir.join("in.wav"), dir.join("out.wav"));
    write_wav(&input, &overload());
    let run = render(&input, &output, &controls_at_top().0);
    let created = output.exists();
    std::fs::remove_dir_all(&dir).unwrap();
    assert_refused(&run);
    assert!(!created, "a refused capture must not create the output file");
}

#[test]
fn a_refused_capture_never_removes_its_input() {
    // The output path names the input file, once directly and once through another spelling.
    let dir = scratch("refused-in-place");
    let input = dir.join("in.wav");
    write_wav(&input, &overload());
    let before = std::fs::read(&input).unwrap();
    let runs = [
        render(&input, &input, &controls_at_top().0),
        render(&input, &dir.join(".").join("in.wav"), &controls_at_top().0),
    ];
    let after = std::fs::read(&input);
    std::fs::remove_dir_all(&dir).unwrap();
    for run in &runs {
        assert_refused(run);
        assert!(String::from_utf8_lossy(&run.stderr).contains("left in place"));
    }
    assert_eq!(after.unwrap(), before, "the input file is untouched");
}

#[test]
fn a_capture_within_full_scale_is_written() {
    // The gate does not refuse a sound capture: a -40 dBFS sine at the defaults.
    let dir = scratch("accepted");
    let (input, output) = (dir.join("in.wav"), dir.join("out.wav"));
    let quiet: Vec<f32> = overload().iter().map(|x| 0.01 * (x / f32::MAX)).collect();
    write_wav(&input, &quiet);
    let run = render(&input, &output, &[]);
    let written: Option<Vec<f32>> = hound::WavReader::open(&output)
        .ok()
        .map(|mut r| r.samples::<f32>().map(|s| s.unwrap()).collect());
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let written = written.expect("the capture exists");
    assert_eq!(written.len(), SAMPLES);
    assert!(written.iter().all(|y| y.is_finite()));
}
