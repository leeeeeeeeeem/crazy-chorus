mod dsp;

use dsp::chorus::{Chorus, MAX_DELAY_MS, MAX_DEPTH_MS};
use truce::prelude::*;
use truce_slint::{PluginContext, SlintEditor};

// Generated from `ui/main.slint` by `build.rs` (defines `ChorusUi`).
slint::include_modules!();

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

    // Fixed size: AU v2 editors can't be resized by the host.
    fn editor(params: Arc<CrazyChorusParams>) -> Box<dyn Editor> {
        Box::new(
            SlintEditor::new(params, EDITOR_SIZE, |ctx: PluginContext<CrazyChorusParams>| {
                let ui = ChorusUi::new().expect("creating ChorusUi");
                wire_knobs(&ui, &ctx);
                // Runs every frame: mirror host-side changes (automation,
                // presets) into the UI.
                Box::new(move |ctx: &PluginContext<CrazyChorusParams>| {
                    ui.set_rate(ctx.get_param(P::Rate));
                    ui.set_depth(ctx.get_param(P::Depth));
                    ui.set_delay(ctx.get_param(P::Delay));
                    ui.set_mix(ctx.get_param(P::Mix));
                    ui.set_rate_text(format_rate(ctx.get_param_plain(P::Rate)).into());
                    ui.set_depth_text(format!("{:.1} ms", ctx.get_param_plain(P::Depth)).into());
                    ui.set_delay_text(format!("{:.1} ms", ctx.get_param_plain(P::Delay)).into());
                    ui.set_mix_text(format!("{:.0}%", ctx.get_param_plain(P::Mix) * 100.0).into());
                })
            })
            .resizable(false),
        )
    }
}

/// Editor size in logical points; matches `ChorusUi`'s width/height.
const EDITOR_SIZE: (u32, u32) = (440, 190);

/// Knob index used by the Slint callbacks -> param. Order must match
/// `ChorusUi` in `ui/main.slint`.
const KNOBS: [P; 4] = [P::Rate, P::Depth, P::Delay, P::Mix];

/// Connect the knob callbacks to host edit gestures. A drag is one gesture
/// (`begin_edit` on press, `set_param` while moving, `end_edit` on release),
/// so the host records it as a single automation pass. (truce-slint's
/// `bind!` sends begin/set/end for every mouse move instead.)
fn wire_knobs(ui: &ChorusUi, ctx: &PluginContext<CrazyChorusParams>) {
    let params = ctx.params();
    let defaults: [f64; 4] = [
        &params.rate.info,
        &params.depth.info,
        &params.delay.info,
        &params.mix.info,
    ]
    .map(|info| info.range.normalize(info.default_plain));

    let c = ctx.clone();
    ui.on_begin_edit(move |i| {
        if let Some(&id) = KNOBS.get(i as usize) {
            c.begin_edit(id);
        }
    });
    let c = ctx.clone();
    ui.on_edit(move |i, v| {
        if let Some(&id) = KNOBS.get(i as usize) {
            c.set_param(id, f64::from(v));
        }
    });
    let c = ctx.clone();
    ui.on_end_edit(move |i| {
        if let Some(&id) = KNOBS.get(i as usize) {
            c.end_edit(id);
        }
    });
    let c = ctx.clone();
    ui.on_reset(move |i| {
        if let (Some(&id), Some(&default)) = (KNOBS.get(i as usize), defaults.get(i as usize)) {
            c.automate(id, default);
        }
    });
}

/// "0.80 Hz" below 1 Hz, "2.5 Hz" above: slow rates need the extra digit.
fn format_rate(hz: f32) -> String {
    if hz < 1.0 { format!("{hz:.2} Hz") } else { format!("{hz:.1} Hz") }
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
