//! Renders a WAV file through the circuit model, using the same code as the plugin.

use dsp::model::CONTROLS;
use dsp::Processor;
use std::process::ExitCode;

const USAGE: &str = "usage: render <in.wav> <out.wav> [control=rotation ...]
       render --plain [control=rotation ...]   print control values in physical units
       render --list                           list controls
  rotation is a knob position from 0 to 1";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("--list") => {
            for c in CONTROLS {
                println!("{}\tdefault {}", c.name.to_lowercase(), c.default);
            }
            Ok(())
        }
        Some("--plain") => rotations(&args[1..]).map(|r| {
            for (spec, rot) in CONTROLS.iter().zip(r) {
                println!("{}={}", spec.name.to_uppercase(), spec.plain(rot));
            }
            // Read by the harness to drive SPICE at the model's signal level.
            println!("_VOLTS_PER_FULL_SCALE={}", dsp::model::VOLTS_PER_FULL_SCALE);
        }),
        _ => run(&args),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn rotations(args: &[String]) -> Result<Vec<f32>, String> {
    let mut r: Vec<f32> = CONTROLS.iter().map(|c| c.default).collect();
    for a in args {
        let (key, value) = a.split_once('=').ok_or(format!("bad control {a}"))?;
        let i = CONTROLS
            .iter()
            .position(|c| c.name.eq_ignore_ascii_case(key))
            .ok_or(format!("unknown control {key}"))?;
        r[i] = value.parse().map_err(|_| format!("bad value {value}"))?;
    }
    Ok(r)
}

fn run(args: &[String]) -> Result<(), String> {
    let [input, output, controls @ ..] = args else {
        return Err("missing arguments".into());
    };
    let rot = rotations(controls)?;
    let mut reader = hound::WavReader::open(input).map_err(|e| format!("{input}: {e}"))?;
    let spec = reader.spec();
    let channels = spec.channels as usize;
    if channels > 2 {
        return Err("only mono and stereo files are supported".into());
    }
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().collect::<Result<_, _>>(),
        hound::SampleFormat::Int => {
            let scale = 1.0 / (1u64 << (spec.bits_per_sample - 1)) as f32;
            reader.samples::<i32>().map(|s| s.map(|v| v as f32 * scale)).collect()
        }
    }
    .map_err(|e| e.to_string())?;
    let mut left: Vec<f32> = samples.iter().step_by(channels).copied().collect();
    let mut right: Vec<f32> = samples.iter().skip(channels - 1).step_by(channels).copied().collect();

    let mut processor = Processor::new(spec.sample_rate as f64, &rot);
    // Run past the end by the latency so the output lines up with the input.
    let latency = processor.latency();
    left.extend(std::iter::repeat_n(0.0, latency));
    right.extend(std::iter::repeat_n(0.0, latency));
    processor.process(&mut left, &mut right);
    // A held sample is not the Circuit's response; do not write a capture that contains one.
    let failed = processor.failed_solves();
    if failed > 0 {
        return Err(format!(
            "the solver failed on {failed} circuit-rate samples and held them; no output written"
        ));
    }

    let out_spec = hound::WavSpec {
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
        ..spec
    };
    let mut writer = hound::WavWriter::create(output, out_spec).map_err(|e| format!("{output}: {e}"))?;
    for i in latency..left.len() {
        writer.write_sample(left[i]).map_err(|e| e.to_string())?;
        if channels == 2 {
            writer.write_sample(right[i]).map_err(|e| e.to_string())?;
        }
    }
    writer.finalize().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_render_with_failed_solves_fails_and_writes_no_file() {
        // 2048 samples of a 1 kHz sine at 48 kHz with a normalized peak of 1e6 (5e6 V), Level 1.
        // With the production iteration bound some solves fail and are held. The render must
        // then return an error and must not create the output file.
        const RATE: u32 = 48_000;
        let samples: Vec<f32> = (0..2048)
            .map(|k| 1e6 * (std::f32::consts::TAU * 1000.0 * k as f32 / RATE as f32).sin())
            .collect();

        // The same processing as `run` does for a mono file, to establish the failure count.
        let mut processor = Processor::new(RATE as f64, &[1.0]);
        let mut left = samples.clone();
        left.extend(std::iter::repeat_n(0.0, processor.latency()));
        let mut right = left.clone();
        processor.process(&mut left, &mut right);
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

        let args = [
            input.display().to_string(),
            output.display().to_string(),
            "level=1".into(),
        ];
        let result = run(&args);
        let created = output.exists();
        std::fs::remove_dir_all(&dir).unwrap();
        let message = result.expect_err("a render with held samples must fail");
        assert!(message.contains(&format!("failed on {failed} ")), "{message}");
        assert!(!created, "a failed render must not create the output file");
    }
}
