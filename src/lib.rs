//! Frequency detection made easy
//!
//! ```
//! use freq_det::FreqDetector;
//!
//! let sample_count = 4096;
//!
//! let sinusoid_440hz = (0..sample_count)
//!     .map(|i| {
//!         use std::f32::consts::TAU;
//!         (i as f32 / 44100.0 * 440.0 * TAU).sin()
//!         // noise
//!         + 0.5 * (i as f32 / 44100.0 * 100.0 * TAU).sin()
//!         + 0.5 * (i as f32 / 44100.0 * 120.0 * TAU).sin()
//!     })
//!     .collect::<Vec<_>>();
//!
//! let freq_detector = FreqDetector::new(44100, sample_count);
//! assert_eq!(freq_detector.detect(&sinusoid_440hz).round(), 440.0);
//! ```

use std::sync::Arc;

use rustfft::{
    num_complex::{Complex, ComplexFloat},
    Fft, FftPlanner,
};

/// Frequency detector
pub struct FreqDetector {
    fft: Arc<dyn Fft<f32>>,
    sample_count: usize,
    sample_rate: usize,
}

impl FreqDetector {
    /// `sample_rate` is `44100` for most modern applications
    ///
    /// `sample_count` numbers between `2048` and `8192` work well.
    /// More samples usually means more accuracy, but requires more audio,
    /// which also means more latency for realtime application.
    pub fn new(sample_rate: usize, sample_count: usize) -> Self {
        let mut planner = FftPlanner::new();
        Self {
            fft: planner.plan_fft_forward(sample_count),

            sample_count,
            sample_rate,
        }
    }

    /// # Panics
    ///
    /// - if `samples.len()` does not match the `sample_count` passed to [Self::new]
    /// - if there are `NaN`s in the sample slice
    pub fn detect(&self, samples: &[f32]) -> f32 {
        let mut buf = samples
            .iter()
            .copied()
            .map(|s| Complex { re: s, im: 0.0 })
            .collect::<Vec<_>>();

        self.fft.process(&mut buf);

        let peak = buf
            .iter()
            .copied()
            .enumerate()
            .take(self.sample_count / 2)
            .max_by_key(|(_, s)| (s.abs() * 1000.0) as u32)
            .expect("to have at least 1 sample");
        if peak.1.abs() < 0.00001 {
            return 0.0;
        }

        // use neighbors for anti-aliasing
        let mut neighbors = Vec::with_capacity(3);
        neighbors.push(peak);
        if peak.0 > 1 {
            neighbors.push((peak.0 - 1, buf[peak.0 - 1]));
        }
        if peak.0 < (self.sample_count / 2 - 1) {
            neighbors.push((peak.0 + 1, buf[peak.0 + 1]));
        }

        neighbors.sort_unstable_by(|c1, c2| c1.1.abs().total_cmp(&c2.1.abs()).reverse());
        // only take the two top values
        neighbors.truncate(2);
        // take weighted average of the two biggest
        // not sure what is the math behind why this works, but it does
        let res = (self.fft_bucket_to_freq(neighbors[0].0) * neighbors[0].1.abs()
            + self.fft_bucket_to_freq(neighbors[1].0) * neighbors[1].1.abs())
            / (neighbors[0].1.abs() + neighbors[1].1.abs());
        if res.is_nan() {
            panic!("Nan values found {neighbors:?}");
        } else {
            res
        }
    }

    fn fft_bucket_to_freq(&self, bucket: usize) -> f32 {
        bucket as f32 * self.sample_rate as f32 / self.sample_count as f32
    }
}

#[cfg(test)]
mod tests {
    use super::FreqDetector;

    #[test]
    fn freq_detector_smoke_test() {
        use std::f32::consts::TAU;
        let sample_count = 4096 * 4;
        let freq_detector = FreqDetector::new(44100, sample_count);

        for freq in [10, 20, 30, 100, 1000, 2000] {
            let sin_samples = (0..sample_count)
                .map(|i| {
                    (i as f32 / 44100.0 * freq as f32 * TAU).sin()
                        // noise
                        + 0.5 * (i as f32 / 44100.0 * 101.0 * TAU).sin()
                        + 0.5 * (i as f32 / 44100.0 * 120.0 * TAU).sin()
                })
                .collect::<Vec<_>>();

            let detected_freq = freq_detector.detect(&sin_samples);
            dbg!(detected_freq, freq);
            assert!(
                (detected_freq - freq as f32).abs() < 0.1,
                "detected {detected_freq} expected {freq}"
            );
        }
    }
}
