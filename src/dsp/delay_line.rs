pub struct DelayLine {
    buffer: Vec<f32>,
    write_pos: usize,
    mask: usize,
    max_delay_samples: usize
}

impl DelayLine {
    pub fn new(max_delay_samples: usize) -> Self {
        let len = (max_delay_samples + 3).next_power_of_two();
        DelayLine {
            buffer: vec![0.0; len],
            write_pos: 0,
            mask: len - 1,
            max_delay_samples: max_delay_samples
        }
    }

    pub fn write(&mut self, x: f32) {
        self.buffer[self.write_pos] = x;
        self.write_pos = (self.write_pos + 1) & self.mask;
    }

    /// read(0) is the most recent write
    pub fn read(&self, delay: usize) -> f32 {
        debug_assert!(delay <= self.mask);
        self.buffer[self.write_pos.wrapping_sub(1 + delay) & self.mask]
    }

    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn impulse_comes_out_after_delay() {
        let mut dl = DelayLine::new(16);
        for n in 0..32 {
            dl.write(if n == 0 { 1.0 } else { 0.0 });
            let expected = if n == 5 { 1.0 } else { 0.0 };
            assert_eq!(dl.read(5), expected, "sample {n}");
        }
    }

    #[test]
    fn delay_is_correct_after_wraparound() {
        let mut dl = DelayLine::new(10);
        let len = dl.mask + 1;
        for i in 0..len * 3 + 5 {
            dl.write(i as f32);
            for d in 0..=dl.mask.min(i) {
                assert_eq!(dl.read(d), (i - d) as f32, "write {i}, delay {d}");
            }
        }
    }

    #[test]
    fn reset_clears_history() {
        let mut dl = DelayLine::new(10);
        for i in 0..dl.mask * 2 {
            dl.write(i as f32 + 1.0);
        }
        dl.reset();
        for d in 0..=dl.mask {
            assert_eq!(dl.read(d), 0.0, "delay {d}");
        }

        dl.write(1.0);
        assert_eq!(dl.read(0), 1.0);
        assert_eq!(dl.read(1), 0.0);
    }

    #[test]
    fn max_delay_is_readable() {
        for max in [0, 1, 61, 62, 100, 1000] {
            for delay in [max, max + 1, max + 2] {
                let mut dl = DelayLine::new(max);
                dl.write(1.0);
                for _ in 0..delay {
                    dl.write(0.0);
                }
                assert_eq!(dl.read(delay), 1.0, "max {max}, delay {delay}");
            }
        }
    }
}
