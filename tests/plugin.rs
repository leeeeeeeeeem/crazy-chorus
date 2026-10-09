//! Plugin-level tests: the chorus driven through truce the way a host drives
//! it (activation, block splitting, param smoothing, bus layouts, state).
//! The DSP itself is unit-tested in `src/dsp/`.

use std::time::Duration;

use crazy_chorus::{CrazyChorus, CrazyChorusParams, CrazyChorusParamsParamId as P, Plugin};
use truce::prelude::*;
use truce_test::{BlockRunner, InputSource, assertions, driver};

const SAMPLE_RATES: [f64; 3] = [44100.0, 48000.0, 96000.0];

/// Normalized [0, 1] value for a param's plain value, using the plugin's own
/// ranges (so the log `rate` range is handled correctly).
fn norm(id: P, plain: f64) -> f64 {
    let params = CrazyChorusParams::new();
    let range = match id {
        P::Rate => &params.rate.info.range,
        P::Depth => &params.depth.info.range,
        P::Delay => &params.delay.info.range,
        P::Mix => &params.mix.info.range,
    };
    range.normalize(plain)
}

/// Deterministic noise in [-1, 1) from a frame index (no `rand` dependency).
fn noise(frame: usize) -> f32 {
    let mut x = (frame as u32).wrapping_mul(0x9E37_79B9) ^ 0x5bd1_e995;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2c1b_3c6d);
    x ^= x >> 12;
    (x >> 8) as f32 / (1 << 23) as f32 - 1.0
}

// --- static checks (no audio) ---

#[test]
fn plugin_info_and_identifiers_are_valid() {
    truce_test::assert_valid_info::<Plugin>();
    truce_test::assert_au_type_codes_ascii::<Plugin>();
    truce_test::assert_fourcc_roundtrip::<Plugin>();
    truce_test::assert_bus_config_effect::<Plugin>();
    truce_test::assert_has_editor::<Plugin>();
}

#[test]
fn params_are_well_formed() {
    truce_test::assert_param_defaults_match::<Plugin>();
    truce_test::assert_param_normalized_clamped::<Plugin>();
    truce_test::assert_param_normalized_roundtrip::<Plugin>();
    truce_test::assert_param_count_matches::<Plugin>();
    truce_test::assert_no_duplicate_param_ids::<Plugin>();
}

#[test]
fn state_save_and_restore() {
    truce_test::assert_state_round_trip::<Plugin>();
    truce_test::assert_corrupt_state_no_crash::<Plugin>();
    truce_test::assert_empty_state_no_crash::<Plugin>();
}

// --- audio runs ---

#[test]
fn silence_in_gives_silence_out() {
    for sample_rate in SAMPLE_RATES {
        let result = driver!(Plugin)
            .sample_rate(sample_rate)
            .duration(Duration::from_millis(200))
            .set_param(P::Depth, norm(P::Depth, 5.0))
            .set_param(P::Mix, 1.0)
            .run();
        for (ch, samples) in result.output.iter().enumerate() {
            assert!(samples.iter().all(|&s| s == 0.0), "{sample_rate} Hz, channel {ch}");
        }
    }
}

#[test]
fn impulse_comes_out_after_base_delay() {
    for sample_rate in SAMPLE_RATES {
        let result = driver!(Plugin)
            .sample_rate(sample_rate)
            .duration(Duration::from_millis(50))
            .set_param(P::Delay, norm(P::Delay, 10.0))
            .set_param(P::Depth, 0.0)
            .set_param(P::Mix, 1.0)
            .input(InputSource::Generator(Box::new(|frame, _| {
                if frame == 0 { 1.0 } else { 0.0 }
            })))
            .run();

        let delay = (sample_rate / 100.0) as usize; // 10 ms
        for (ch, samples) in result.output.iter().enumerate() {
            for (n, &s) in samples.iter().enumerate() {
                let expected = if n == delay { 1.0 } else { 0.0 };
                assert!(
                    (s - expected).abs() < 1e-3,
                    "{sample_rate} Hz, channel {ch}, sample {n}: expected {expected}, got {s}"
                );
            }
        }
    }
}

#[test]
fn output_does_not_depend_on_block_size() {
    let render = |block_size: usize| {
        driver!(Plugin)
            .sample_rate(48000.0)
            .block_size(block_size)
            .duration(Duration::from_millis(300))
            .input(InputSource::Generator(Box::new(|frame, _| noise(frame))))
            // Mid-run automation: sample-accurate events must land on the
            // same sample whatever the block size.
            .script(|s| {
                s.wait_ms(100);
                s.set_param(P::Delay, norm(P::Delay, 22.0));
                s.set_param(P::Depth, norm(P::Depth, 4.0));
                s.wait_ms(100);
                s.set_param(P::Rate, norm(P::Rate, 3.0));
                s.set_param(P::Mix, norm(P::Mix, 0.8));
            })
            .run()
            .output
    };

    let reference = render(64);
    for block_size in [1, 512, 4096] {
        let output = render(block_size);
        for (ch, (out, want)) in output.iter().zip(&reference).enumerate() {
            for (n, (a, b)) in out.iter().zip(want).enumerate() {
                assert_eq!(a.to_bits(), b.to_bits(), "block {block_size}, channel {ch}, sample {n}");
            }
        }
    }
}

// The linear mix smoother lands exactly on 0.0, so after a ramp down the
// plugin is a bit-exact passthrough. (An exponential smoother would only
// approach 0 and this would fail.)
#[test]
fn mix_ramped_to_zero_becomes_bit_exact_passthrough() {
    let sample_rate = 48000.0;
    let result = driver!(Plugin)
        .sample_rate(sample_rate)
        .duration(Duration::from_millis(300))
        .set_param(P::Depth, norm(P::Depth, 5.0))
        .input(InputSource::Generator(Box::new(|frame, _| noise(frame))))
        .script(|s| {
            s.wait_ms(100);
            s.set_param(P::Mix, 0.0);
        })
        .run();

    // 100 ms in, plus the 20 ms ramp, plus margin.
    let settled = (sample_rate * 0.150) as usize;
    let before_change = (sample_rate * 0.090) as usize;
    for (ch, samples) in result.output.iter().enumerate() {
        assert_ne!(samples[before_change].to_bits(), noise(before_change).to_bits(), "channel {ch} is wet before the ramp");
        for (n, &s) in samples.iter().enumerate().skip(settled) {
            assert_eq!(s.to_bits(), noise(n).to_bits(), "channel {ch}, sample {n}");
        }
    }
}

#[test]
fn extreme_automation_stays_finite_and_bounded() {
    for sample_rate in SAMPLE_RATES {
        let result = driver!(Plugin)
            .sample_rate(sample_rate)
            .block_size(256)
            .duration(Duration::from_secs(2))
            .input(InputSource::Generator(Box::new(|frame, _| noise(frame))))
            .script(|s| {
                // Slam every param between its extremes every 10 ms.
                for step in 0..190 {
                    let v = if step % 2 == 0 { 0.0 } else { 1.0 };
                    s.set_param(P::Rate, v);
                    s.set_param(P::Delay, 1.0 - v);
                    s.set_param(P::Depth, v);
                    s.set_param(P::Mix, if step % 3 == 0 { 0.0 } else { 1.0 });
                    s.wait_ms(10);
                }
            })
            .run();
        assertions::assert_no_nans(&result);
        // Cubic Hermite can overshoot full-scale input by up to 25%.
        assertions::assert_peak_below(&result, 1.5);
    }
}

#[test]
fn tail_dies_within_the_longest_delay() {
    let sample_rate = 48000.0;
    let input_ms: u32 = 200;
    let result = driver!(Plugin)
        .sample_rate(sample_rate)
        .duration(Duration::from_millis(400))
        .set_param(P::Delay, 1.0) // 25 ms
        .set_param(P::Depth, 1.0) // ±5 ms
        .set_param(P::Mix, 1.0)
        .input(InputSource::Generator(Box::new(move |frame, sr| {
            if (frame as f64) < sr * f64::from(input_ms) / 1000.0 { noise(frame) } else { 0.0 }
        })))
        .run();

    // Longest delay is 30 ms; allow one extra ms for the Hermite taps.
    assertions::assert_nonzero_between(
        &result,
        Duration::from_millis(input_ms.into()),
        Duration::from_millis((input_ms + 20).into()),
    );
    assertions::assert_silence_after(&result, Duration::from_millis((input_ms + 31).into()));
}

#[test]
fn reported_tail_covers_the_longest_delay() {
    for (sample_rate, expected) in [(44100.0, 1323), (48000.0, 1440), (96000.0, 2880)] {
        let params = CrazyChorusParams::new();
        let mut state = CrazyChorus::default();
        CrazyChorus::reset(&mut state, &params, &AudioConfig::new(sample_rate, 512));
        assert_eq!(CrazyChorus::tail(&state), expected, "{sample_rate} Hz");
    }
}

// --- bus layouts ---

/// Render `frames` of noise through a fresh plugin state, one block, with
/// the given number of input channels and two outputs.
fn render_block(inputs: usize, frames: usize) -> Vec<Vec<f32>> {
    let params = CrazyChorusParams::new();
    params.set_normalized(P::Depth.into(), norm(P::Depth, 5.0));
    params.set_normalized(P::Mix.into(), 1.0);
    params.snap_smoothers();

    let signal: Vec<f32> = (0..frames).map(noise).collect();
    let channels: Vec<&[f32]> = (0..inputs).map(|_| signal.as_slice()).collect();
    let mut runner = BlockRunner::<CrazyChorus>::new(&params).sample_rate(48000.0).outputs(2, frames);
    runner.run(&params, &channels, &EventList::with_capacity(0)).audio
}

#[test]
fn mono_in_stereo_out_matches_stereo_with_duplicated_input() {
    let mono = render_block(1, 24000);
    let stereo = render_block(2, 24000);
    assert_eq!(mono, stereo);

    // And the stereo LFO offset still makes the two outputs differ.
    let max_diff = mono[0].iter().zip(&mono[1]).map(|(l, r)| (l - r).abs()).fold(0.0, f32::max);
    assert!(max_diff > 0.1, "mono source gives no stereo width: max diff {max_diff}");
}

// --- real-time safety ---

// Only meaningful with the allocation checker installed:
// `cargo test --features rt-paranoid`.
#[cfg(feature = "rt-paranoid")]
#[test]
fn process_is_realtime_clean() {
    truce_test::assert_realtime_clean(|| {
        driver!(Plugin)
            .duration(Duration::from_millis(500))
            .input(InputSource::Generator(Box::new(|frame, _| noise(frame))))
            .script(|s| {
                s.wait_ms(100);
                s.set_param(P::Delay, 1.0);
                s.set_param(P::Rate, 1.0);
                s.set_param(P::Mix, 0.0);
            })
            .run()
    });
}
