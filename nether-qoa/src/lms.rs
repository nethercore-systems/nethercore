//! LMS (Least Mean Squares) predictor for QOA codec

/// LMS (Least Mean Squares) predictor state
///
/// QOA uses a 4-tap LMS predictor to predict the next sample based on
/// the history of reconstructed samples. The weights are adapted after
/// each sample to minimize prediction error.
#[derive(Clone, Copy, Debug)]
pub struct QoaLms {
    /// History of last 4 reconstructed samples
    pub history: [i32; 4],

    /// Adaptive filter weights
    pub weights: [i32; 4],
}

impl Default for QoaLms {
    fn default() -> Self {
        Self::new()
    }
}

impl QoaLms {
    /// Create new LMS state with default weights
    ///
    /// The initial weights are tuned for typical audio signals:
    /// `[0, 0, -(1 << 13), 1 << 14]` which corresponds to approximately
    /// `[0, 0, -1, 2]` after the shift in prediction.
    #[must_use]
    pub fn new() -> Self {
        Self {
            history: [0; 4],
            weights: [0, 0, -(1 << 13), 1 << 14],
        }
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        self.history = [0; 4];
        self.weights = [0, 0, -(1 << 13), 1 << 14];
    }

    /// Predict next sample based on history
    ///
    /// Returns the predicted sample value (before adding the dequantized residual).
    #[inline]
    #[must_use]
    pub fn predict(&self) -> i32 {
        let mut prediction = 0i32;
        for i in 0..4 {
            prediction = prediction.wrapping_add(self.weights[i].wrapping_mul(self.history[i]));
        }
        prediction >> 13
    }

    /// Update weights and history after decoding a sample
    ///
    /// # Arguments
    /// * `sample` - The reconstructed sample (predicted + dequantized residual, clamped)
    /// * `residual` - The dequantized residual value
    #[inline]
    pub fn update(&mut self, sample: i32, residual: i32) {
        let delta = residual >> 4;
        for i in 0..4 {
            self.weights[i] += if self.history[i] < 0 { -delta } else { delta };
        }

        // Shift history, add new sample
        self.history[0] = self.history[1];
        self.history[1] = self.history[2];
        self.history[2] = self.history[3];
        self.history[3] = sample;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lms_default() {
        let lms = QoaLms::new();
        assert_eq!(lms.history, [0, 0, 0, 0]);
        assert_eq!(lms.weights, [0, 0, -(1 << 13), 1 << 14]);
    }

    #[test]
    fn test_lms_predict_zeros() {
        let lms = QoaLms::new();
        // With zero history, prediction should be 0
        assert_eq!(lms.predict(), 0);
    }

    #[test]
    fn test_lms_predict_known_values() {
        let lms = QoaLms {
            history: [1000, 2000, 3000, 4000],
            weights: [4096, 4096, 4096, 4096], // All equal weights
        };

        // prediction = (1000*4096 + 2000*4096 + 3000*4096 + 4000*4096) >> 13
        //            = (4096000 + 8192000 + 12288000 + 16384000) >> 13
        //            = 40960000 >> 13
        //            = 5000
        assert_eq!(lms.predict(), 5000);
    }

    #[test]
    fn test_lms_predict_default_weights() {
        // Default weights: [0, 0, -(1<<13), 1<<14]
        //                = [0, 0, -8192, 16384]
        let mut lms = QoaLms::new();
        lms.history = [100, 200, 300, 400];

        // prediction = (100*0 + 200*0 + 300*(-8192) + 400*16384) >> 13
        //            = (0 + 0 - 2457600 + 6553600) >> 13
        //            = 4096000 >> 13
        //            = 500
        assert_eq!(lms.predict(), 500);
    }

    #[test]
    fn test_lms_update() {
        let mut lms = QoaLms::new();
        lms.history = [100, 200, 300, 400];
        lms.weights = [1000, 2000, 3000, 4000];

        // Update with sample=500, residual=160
        // delta = 160 >> 4 = 10
        // All history values are positive, so all weights increase by 10
        lms.update(500, 160);

        assert_eq!(lms.weights, [1010, 2010, 3010, 4010]);
        assert_eq!(lms.history, [200, 300, 400, 500]); // Shifted, new sample added
    }

    #[test]
    fn test_lms_update_negative_history() {
        let mut lms = QoaLms::new();
        lms.history = [-100, 200, -300, 400];
        lms.weights = [1000, 2000, 3000, 4000];

        // delta = 160 >> 4 = 10
        // history[0] < 0: weight[0] -= 10 -> 990
        // history[1] >= 0: weight[1] += 10 -> 2010
        // history[2] < 0: weight[2] -= 10 -> 2990
        // history[3] >= 0: weight[3] += 10 -> 4010
        lms.update(500, 160);

        assert_eq!(lms.weights, [990, 2010, 2990, 4010]);
    }

    #[test]
    fn test_lms_reset() {
        let mut lms = QoaLms {
            history: [1, 2, 3, 4],
            weights: [5, 6, 7, 8],
        };
        lms.reset();
        assert_eq!(lms.history, [0, 0, 0, 0]);
        assert_eq!(lms.weights, [0, 0, -(1 << 13), 1 << 14]);
    }
}
