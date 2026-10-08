use super::delay_line::DelayLine;
use super::lfo::Lfo;

const MIN_DELAY_MS: f32 = 7.0;
const MAX_DELAY_MS: f32 = 25.0;
const MAX_DEPTH_MS: f32 = 5.0;
const STEREO_OFFSET: f32 = 0.25;

pub struct Chorus {
    delay_l: DelayLine,
    delay_r: DelayLine,
    lfo_l: Lfo,
    lfo_r: Lfo,
    sample_rate: f32,
    delay_ms: f32,
    depth_ms: f32,
    mix: f32,
}

impl Chorus {
    pub fn new(sample_rate: f32) -> Self {
        todo!()
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        todo!()
    }

    pub fn set_rate(&mut self, rate_hz: f32) {
        todo!()
    }

    pub fn set_delay_ms(&mut self, delay_ms: f32) {
        todo!()
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
