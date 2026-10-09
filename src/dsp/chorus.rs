use super::delay_line::DelayLine;
use super::lfo::Lfo;

const MIN_DELAY_MS: f32 = 7.0;
const MAX_DELAY_MS: f32 = 25.0;
const MAX_DEPTH_MS: f32 = 5.0;
const STEREO_OFFSET: f32 = 0.25;

const DEFAULT_RATE: f32 = 0.7;
const DEFAULT_DELAY: f32 = 15.;
const DEFAULT_DEPTH: f32 = 2.;
const DEFAULT_MIX: f32 = 50.;

pub struct Chorus {
    delay_l: DelayLine,
    delay_r: DelayLine,
    lfo_l: Lfo,
    lfo_r: Lfo,
    sample_rate: f32,
    delay_ms: f32,
    delay_samples: usize,
    depth_ms: f32,
    depth_samples: usize,
    mix: f32,
}

fn ms_to_samples(ms: f32, sample_rate: f32) -> usize {
    let samples = f32::ceil(sample_rate * (ms / 1000.));
    samples as usize
}

impl Chorus {
    pub fn new(sample_rate: f32) -> Self {
        assert!(sample_rate > 0.);
        let samples = ms_to_samples(MAX_DELAY_MS + MAX_DEPTH_MS, sample_rate);
        Chorus { 
            delay_l: DelayLine::new(samples),
            delay_r: DelayLine::new(samples),
            lfo_l: Lfo::new(sample_rate),
            lfo_r: Lfo::new(sample_rate),
            sample_rate,
            delay_ms: DEFAULT_DELAY,
            delay_samples: ms_to_samples(DEFAULT_DELAY, sample_rate),
            depth_ms: DEFAULT_DEPTH,
            depth_samples: ms_to_samples(DEFAULT_DEPTH, sample_rate),
            mix: DEFAULT_MIX,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        let samples = ms_to_samples(MAX_DELAY_MS + MAX_DEPTH_MS, sample_rate);
        self.delay_l = DelayLine::new(samples);
        self.delay_r = DelayLine::new(samples);
        self.lfo_l.set_sample_rate(sample_rate);
        self.lfo_r.set_sample_rate(sample_rate);
    }

    pub fn set_rate(&mut self, rate_hz: f32) {
        self.lfo_l.set_rate(rate_hz);
        self.lfo_r.set_rate(rate_hz);
    }

    pub fn set_delay_ms(&mut self, delay_ms: f32) {
        debug_assert!(!delay_ms.is_nan());
        let delay_ms = delay_ms.clamp(MIN_DELAY_MS, MAX_DELAY_MS);
        let delay_samples = ms_to_samples(delay_ms, self.sample_rate);
        self.delay_ms = delay_ms;
        self.delay_samples = delay_samples;
    }

    pub fn set_depth_ms(&mut self, depth_ms: f32) {
        todo!()
    }

    pub fn set_mix(&mut self, mix: f32) {
        todo!()
    }

    pub fn reset(&mut self) {
        todo!()
    }

    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
