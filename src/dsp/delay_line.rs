pub struct DelayLine {
    buffer: Vec<f32>,
    write_pos: usize,
}

impl DelayLine {
    pub fn new(max_delay_samples: usize) -> Self {
        DelayLine { 
            buffer: vec![0.0; max_delay_samples],
            write_pos: 0,
        }
    }

    pub fn write(&mut self, x: f32) {
        self.buffer[self.write_pos] = x;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
    }

    /// Returns the sample written `delay` samples ago.
    /// Decide (and document here) whether `read(0)` is the most recent write.
    pub fn read(&self, delay: usize) -> f32 {
        todo!()
    }

    /// Clears the stored history to silence.
    pub fn reset(&mut self) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn impulse_comes_out_after_delay() {
        // Write 1.0 then zeros; read(5) is 1.0 exactly once, 5 samples later.
        todo!()
    }

    #[test]
    fn delay_is_correct_after_wraparound() {
        // Write more samples than the buffer holds, then check read() still lines up.
        todo!()
    }

    #[test]
    fn reset_clears_history() {
        // After writing non-zero samples and calling reset(), every read is 0.0.
        todo!()
    }

    #[test]
    fn max_delay_is_readable() {
        // read(max_delay_samples) returns the oldest stored sample.
        todo!()
    }
}
