use truce::prelude::*;
use truce_gui_types::layout::{GridLayout, knob, widgets};

#[derive(Params)]
pub struct CrazyChorusParams {
    #[param(
        name = "Gain",
        range = "linear(-60, 6)",
        unit = "dB",
        smooth = "exp(5)"
    )]
    pub gain: FloatParam,
}

use CrazyChorusParamsParamId as P;

// The plugin struct is its own DSP state (`type DspState = Self`). The
// shell owns it and preserves it across a hot-reload, so a code-only
// reload keeps reverb tails and oscillator phase alive.
#[derive(Default)]
pub struct CrazyChorus {
    // Per-instance DSP state - filters, delay lines, phase counters.
    // Fields need `Default`. Add them as your DSP grows.
}

impl PluginLogic for CrazyChorus {
    type Params = CrazyChorusParams;
    type DspState = Self;

    fn process(
        _state: &mut Self::DspState,
        params: &Self::Params,
        buffer: &mut AudioBuffer,
        _events: &EventList,
        _context: &mut ProcessContext,
    ) -> ProcessStatus {
        for i in 0..buffer.num_samples() {
            let gain = db_to_linear(params.gain.read());
            for ch in 0..buffer.channels() {
                let (inp, out) = buffer.io(ch);
                out[i] = inp[i] * gain;
            }
        }
        ProcessStatus::Normal
    }

    fn editor(params: Arc<CrazyChorusParams>) -> Box<dyn Editor> {
        truce_gui::default_editor(
            params,
            GridLayout::build(vec![widgets(vec![knob(P::Gain, "Gain")])]),
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
