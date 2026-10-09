mod dsp;

use dsp::chorus::{Chorus, MAX_DELAY_MS, MAX_DEPTH_MS};
use truce::prelude::*;
use truce_gui_types::layout::{GridLayout, knob, widgets};

// Ranges must be literals in the attribute, so they duplicate the consts in
// `dsp::chorus`. `tests::param_ranges_match_dsp_consts` keeps them in sync.
//
// Smoothing: delay and depth move the read position, so they glide (50 ms)
// rather than click. Mix uses a *linear* ramp because it lands exactly on its
// target; an exponential one only approaches 0.0, so the DSP's bit-exact
// mix = 0 path would never trigger. Rate uses log smoothing to match its log
// range.
#[derive(Params)]
pub struct CrazyChorusParams {
    #[param(
        name = "Rate",
        range = "log(0.1, 5)",
        default = 0.8,
        unit = "Hz",
        smooth = "log(20)"
    )]
    pub rate: FloatParam,

    #[param(
        name = "Depth",
        range = "linear(0, 5)",
        default = 2,
        unit = "ms",
        smooth = "exp(50)"
    )]
    pub depth: FloatParam,

    #[param(
        name = "Delay",
        range = "linear(7, 25)",
        default = 15,
        unit = "ms",
        smooth = "exp(50)"
    )]
    pub delay: FloatParam,

    // 0-1 internally; the `%` unit displays it as 0-100%.
    #[param(
        name = "Mix",
        range = "linear(0, 1)",
        default = 0.5,
        unit = "%",
        smooth = "linear(20)"
    )]
    pub mix: FloatParam,
}

use CrazyChorusParamsParamId as P;

/// Placeholder rate for the state built before the host's first `reset`.
/// `reset` always replaces it with the real rate before audio runs.
const INITIAL_SAMPLE_RATE: f32 = 48000.0;

// The plugin struct is its own DSP state (`type DspState = Self`).
pub struct CrazyChorus {
    chorus: Chorus,
    sample_rate: f32,
}

// truce builds the state with `Default` (off the audio thread, so allocating
// here is fine). The sample rate is unknown until `reset`, so start from a
// placeholder and rebuild there.
impl Default for CrazyChorus {
    fn default() -> Self {
        Self {
            chorus: Chorus::new(INITIAL_SAMPLE_RATE),
            sample_rate: INITIAL_SAMPLE_RATE,
        }
    }
}

impl PluginLogic for CrazyChorus {
    type Params = CrazyChorusParams;
    type DspState = Self;

    // Stereo, plus mono in / stereo out: a mono source feeds both channels,
    // and the L/R LFO offset still creates width.
    fn bus_layouts() -> Vec<BusLayout> {
        vec![
            BusLayout::stereo(),
            BusLayout::new()
                .with_input("Main", ChannelConfig::Mono)
                .with_output("Main", ChannelConfig::Stereo),
        ]
    }

    // Not real-time: re-allocates the delay lines for the new rate.
    fn reset(state: &mut Self::DspState, _params: &Self::Params, config: &AudioConfig) {
        #[allow(clippy::cast_possible_truncation)]
        let sample_rate = config.sample_rate as f32;
        state.sample_rate = sample_rate;
        state.chorus.set_sample_rate(sample_rate);
        state.chorus.reset();
    }

    fn process(
        state: &mut Self::DspState,
        params: &Self::Params,
        buffer: &mut AudioBuffer,
        _events: &EventList,
        _context: &mut ProcessContext,
    ) -> ProcessStatus {
        let chorus = &mut state.chorus;
        // `for_each_frame_io::<2, 2>` repeats the last input channel when the
        // bus has fewer (mono in -> both inputs) and drops outputs past the
        // bus width, so one path serves every layout.
        buffer.for_each_frame_io::<2, 2, _>(|input, output| {
            // Smoothed values advance one step per `read()`, so read every
            // param once per sample. The setters are a multiply and a clamp.
            chorus.set_rate(params.rate.read());
            chorus.set_delay_ms(params.delay.read());
            chorus.set_depth_ms(params.depth.read());
            chorus.set_mix(params.mix.read());
            let (left, right) = chorus.process(input[0], input[1]);
            output[0] = left;
            output[1] = right;
        });
        ProcessStatus::Normal
    }

    // The wet signal keeps sounding for up to the longest delay after the
    // input stops.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn tail(state: &Self::DspState) -> u32 {
        (state.sample_rate * (MAX_DELAY_MS + MAX_DEPTH_MS) / 1000.0).ceil() as u32
    }

    fn editor(params: Arc<CrazyChorusParams>) -> Box<dyn Editor> {
        truce_gui::default_editor(
            params,
            GridLayout::build(vec![widgets(vec![
                knob(P::Rate, "Rate"),
                knob(P::Depth, "Depth"),
                knob(P::Delay, "Delay"),
                knob(P::Mix, "Mix"),
            ])]),
        )
    }
}

truce::plugin! {
    logic: CrazyChorus,
    params: CrazyChorusParams,
}

// Installs the real-time allocation checker under `--features rt-paranoid`
// (a no-op otherwise). Wrap a driver run in `assert_no_audio_alloc` to
// fail a test if `process` ever allocates. See the audio-testing guide.
truce::enable_rt_paranoid!();

#[cfg(test)]
mod tests {
    use super::*;
    use dsp::chorus::MIN_DELAY_MS;

    // The `#[param]` ranges are string literals, so they can't read the DSP
    // consts directly. This keeps the two from drifting apart.
    #[test]
    fn param_ranges_match_dsp_consts() {
        let params = CrazyChorusParams::new();
        let range = |r: &truce::params::ParamRange| (r.min(), r.max());
        assert_eq!(
            range(&params.delay.info.range),
            (f64::from(MIN_DELAY_MS), f64::from(MAX_DELAY_MS)),
            "delay"
        );
        assert_eq!(range(&params.depth.info.range), (0.0, f64::from(MAX_DEPTH_MS)), "depth");
        assert_eq!(range(&params.mix.info.range), (0.0, 1.0), "mix");
    }
}
