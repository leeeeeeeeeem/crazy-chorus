pub struct Lfo {
    phase: f32,
    phase_inc: f32,
    sample_rate: f32,
}

impl Lfo {
    pub fn new(sample_rate: f32) -> Self {
        assert!(sample_rate > 0.0);
        Lfo {
            phase: 0.0,
            phase_inc: 0.0,
            sample_rate,
        }
    }

    pub fn set_rate(&mut self, rate_hz: f32) {
        debug_assert!(!rate_hz.is_nan());
        self.phase_inc = (rate_hz / self.sample_rate).max(0.0).min(0.5)
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        assert!(sample_rate > 0.0);
        let rate_hz = self.sample_rate * self.phase_inc;
        self.sample_rate = sample_rate;
        self.set_rate(rate_hz);
    }

    pub fn set_phase(&mut self, phase: f32) {
        debug_assert!(!phase.is_nan());
        self.phase = phase.rem_euclid(1.0);
    }

    /// Returns the current value in `-1.0..=1.0`, then advances one sample.
    pub fn next(&mut self) -> f32 {
        todo!()
    }

    pub fn reset(&mut self) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f32, expected: f32, msg: &str) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "{msg}: expected {expected}, got {actual}"
        );
    }

    #[test]
    fn set_rate_computes_phase_inc() {
        let mut lfo = Lfo::new(44100.0);
        for rate in [0.1, 1.0, 5.0] {
            lfo.set_rate(rate);
            assert_close(lfo.phase_inc, rate / 44100.0, &format!("rate {rate}"));
        }
    }

    #[test]
    fn set_rate_clamps_to_zero_and_nyquist() {
        let mut lfo = Lfo::new(44100.0);
        lfo.set_rate(-3.0);
        assert_eq!(lfo.phase_inc, 0.0, "negative rate");
        lfo.set_rate(22050.0);
        assert_eq!(lfo.phase_inc, 0.5, "exactly nyquist");
        lfo.set_rate(1e9);
        assert_eq!(lfo.phase_inc, 0.5, "above nyquist");
        lfo.set_rate(f32::INFINITY);
        assert_eq!(lfo.phase_inc, 0.5, "infinite rate");
    }

    // The debug_assert in set_rate fires on NaN in debug builds, so this only
    // runs under `cargo test --release`, where the release fallback applies.
    #[test]
    #[cfg(not(debug_assertions))]
    fn set_rate_nan_stops_lfo_in_release() {
        let mut lfo = Lfo::new(44100.0);
        lfo.set_rate(1.0);
        lfo.set_rate(f32::NAN);
        assert_eq!(lfo.phase_inc, 0.0);
    }

    #[test]
    fn set_sample_rate_keeps_rate_in_hz() {
        let mut lfo = Lfo::new(44100.0);
        lfo.set_rate(1.0);
        lfo.set_sample_rate(96000.0);
        assert_eq!(lfo.sample_rate, 96000.0);
        assert_close(lfo.phase_inc, 1.0 / 96000.0, "44.1k -> 96k");
        lfo.set_sample_rate(48000.0);
        assert_close(lfo.phase_inc, 1.0 / 48000.0, "96k -> 48k");
    }

    #[test]
    fn set_sample_rate_lowering_keeps_nyquist_bound() {
        let mut lfo = Lfo::new(96000.0);
        lfo.set_rate(48000.0);
        assert_eq!(lfo.phase_inc, 0.5);
        lfo.set_sample_rate(44100.0);
        assert!(lfo.phase_inc <= 0.5, "phase_inc {}", lfo.phase_inc);
    }

    #[test]
    fn set_phase_wraps_into_unit_range() {
        let mut lfo = Lfo::new(44100.0);
        for (input, expected) in [(0.0, 0.0), (0.25, 0.25), (1.25, 0.25), (-0.25, 0.75), (3.0, 0.0)] {
            lfo.set_phase(input);
            assert_close(lfo.phase, expected, &format!("set_phase({input})"));
        }
    }
}
