//! Texture quality heuristics and validation
//!
//! Provides actual checking code for texture quality metrics,
//! ensuring generated textures meet professional standards.

use super::TextureBuffer;
use std::collections::HashSet;

/// Quality assessment result for a texture
#[derive(Debug, Clone)]
pub struct TextureQualityReport {
    /// Overall quality score (0.0 to 1.0)
    pub score: f32,
    /// Individual metric results
    pub metrics: TextureMetrics,
    /// List of issues found
    pub issues: Vec<QualityIssue>,
    /// List of passed checks
    pub passed: Vec<&'static str>,
}

/// Individual texture quality metrics
#[derive(Debug, Clone)]
pub struct TextureMetrics {
    /// Contrast level (0.0 to 1.0)
    pub contrast: f32,
    /// Number of unique colors
    pub unique_colors: u32,
    /// Histogram balance (0.0 to 1.0, 1.0 = perfectly balanced)
    pub histogram_balance: f32,
    /// Noise coherence (0.0 to 1.0, higher = more coherent patterns)
    pub noise_coherence: f32,
    /// Tileability score (0.0 to 1.0)
    pub tileability: f32,
    /// Has multi-frequency detail
    pub has_detail_layers: bool,
    /// Has color variation (not just brightness)
    pub has_color_variation: bool,
    /// Edge visibility (for tileable textures)
    pub seam_visibility: f32,
}

/// A quality issue found in the texture
#[derive(Debug, Clone)]
pub struct QualityIssue {
    /// Issue severity
    pub severity: IssueSeverity,
    /// Issue description
    pub message: String,
    /// Suggested fix
    pub suggestion: String,
}

/// Severity of a quality issue
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    /// Minor issue, texture is usable
    Warning,
    /// Significant issue that should be addressed
    Error,
    /// Critical issue, texture is unusable
    Critical,
}

/// Quality thresholds for validation
#[derive(Debug, Clone)]
pub struct QualityThresholds {
    /// Minimum contrast (default 0.15)
    pub min_contrast: f32,
    /// Minimum unique colors (default 50)
    pub min_unique_colors: u32,
    /// Minimum histogram balance (default 0.3)
    pub min_histogram_balance: f32,
    /// Maximum seam visibility for tileable (default 0.1)
    pub max_seam_visibility: f32,
    /// Minimum noise coherence (default 0.4)
    pub min_noise_coherence: f32,
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            min_contrast: 0.15,
            min_unique_colors: 50,
            min_histogram_balance: 0.3,
            max_seam_visibility: 0.1,
            min_noise_coherence: 0.4,
        }
    }
}

impl QualityThresholds {
    /// Lenient thresholds for stylized/retro textures
    pub fn lenient() -> Self {
        Self {
            min_contrast: 0.1,
            min_unique_colors: 20,
            min_histogram_balance: 0.2,
            max_seam_visibility: 0.15,
            min_noise_coherence: 0.3,
        }
    }

    /// Strict thresholds for photorealistic textures
    pub fn strict() -> Self {
        Self {
            min_contrast: 0.2,
            min_unique_colors: 100,
            min_histogram_balance: 0.4,
            max_seam_visibility: 0.05,
            min_noise_coherence: 0.5,
        }
    }
}

/// Assess texture quality
pub fn assess_quality(texture: &TextureBuffer, thresholds: &QualityThresholds) -> TextureQualityReport {
    let mut issues = Vec::new();
    let mut passed = Vec::new();

    // Calculate all metrics
    let contrast = calculate_contrast(texture);
    let unique_colors = count_unique_colors(texture);
    let histogram_balance = calculate_histogram_balance(texture);
    let noise_coherence = calculate_noise_coherence(texture);
    let tileability = calculate_tileability(texture);
    let seam_visibility = calculate_seam_visibility(texture);
    let has_detail_layers = detect_multi_frequency_detail(texture);
    let has_color_variation = detect_hue_variation(texture);

    // Check against thresholds
    if contrast < thresholds.min_contrast {
        issues.push(QualityIssue {
            severity: if contrast < 0.05 { IssueSeverity::Critical } else { IssueSeverity::Error },
            message: format!("Low contrast: {:.2} (minimum: {:.2})", contrast, thresholds.min_contrast),
            suggestion: "Add more layers with different brightness levels or increase noise intensity".to_string(),
        });
    } else {
        passed.push("Contrast");
    }

    if unique_colors < thresholds.min_unique_colors {
        issues.push(QualityIssue {
            severity: if unique_colors < 10 { IssueSeverity::Critical } else { IssueSeverity::Warning },
            message: format!("Low color variety: {} colors (minimum: {})", unique_colors, thresholds.min_unique_colors),
            suggestion: "Add color variation layers with hue/saturation shifts".to_string(),
        });
    } else {
        passed.push("Color variety");
    }

    if histogram_balance < thresholds.min_histogram_balance {
        issues.push(QualityIssue {
            severity: IssueSeverity::Warning,
            message: format!("Unbalanced histogram: {:.2} (minimum: {:.2})", histogram_balance, thresholds.min_histogram_balance),
            suggestion: "Texture is too dark or too bright overall; add contrast or adjust base color".to_string(),
        });
    } else {
        passed.push("Histogram balance");
    }

    if noise_coherence < thresholds.min_noise_coherence {
        issues.push(QualityIssue {
            severity: IssueSeverity::Warning,
            message: format!("Low noise coherence: {:.2} (minimum: {:.2})", noise_coherence, thresholds.min_noise_coherence),
            suggestion: "Patterns appear random; use lower-frequency noise or add structured features".to_string(),
        });
    } else {
        passed.push("Noise coherence");
    }

    if seam_visibility > thresholds.max_seam_visibility {
        issues.push(QualityIssue {
            severity: IssueSeverity::Warning,
            message: format!("Visible seams: {:.2} (maximum: {:.2})", seam_visibility, thresholds.max_seam_visibility),
            suggestion: "Seams visible when tiled; use tileable noise or blend edges".to_string(),
        });
    } else {
        passed.push("Tileability");
    }

    if !has_detail_layers {
        issues.push(QualityIssue {
            severity: IssueSeverity::Warning,
            message: "No multi-frequency detail detected".to_string(),
            suggestion: "Add detail layers at different scales (coarse + fine noise)".to_string(),
        });
    } else {
        passed.push("Multi-frequency detail");
    }

    if !has_color_variation {
        issues.push(QualityIssue {
            severity: IssueSeverity::Warning,
            message: "No hue variation detected (grayscale or single-hue)".to_string(),
            suggestion: "Add color variation with hue shifts for more natural look".to_string(),
        });
    } else {
        passed.push("Color variation");
    }

    // Calculate overall score
    let mut score = 0.0;
    let mut weight_sum = 0.0;

    let add_score = |score: &mut f32, weight_sum: &mut f32, value: f32, weight: f32| {
        *score += value.clamp(0.0, 1.0) * weight;
        *weight_sum += weight;
    };

    add_score(&mut score, &mut weight_sum, contrast / 0.3, 2.0);
    add_score(&mut score, &mut weight_sum, (unique_colors as f32) / 200.0, 1.5);
    add_score(&mut score, &mut weight_sum, histogram_balance, 1.0);
    add_score(&mut score, &mut weight_sum, noise_coherence, 1.0);
    add_score(&mut score, &mut weight_sum, tileability, 1.0);
    add_score(&mut score, &mut weight_sum, 1.0 - seam_visibility, 0.5);
    add_score(&mut score, &mut weight_sum, if has_detail_layers { 1.0 } else { 0.0 }, 1.5);
    add_score(&mut score, &mut weight_sum, if has_color_variation { 1.0 } else { 0.0 }, 1.0);

    let final_score = if weight_sum > 0.0 { score / weight_sum } else { 0.0 };

    TextureQualityReport {
        score: final_score,
        metrics: TextureMetrics {
            contrast,
            unique_colors,
            histogram_balance,
            noise_coherence,
            tileability,
            has_detail_layers,
            has_color_variation,
            seam_visibility,
        },
        issues,
        passed,
    }
}

/// Calculate contrast using standard deviation of luminance
fn calculate_contrast(texture: &TextureBuffer) -> f32 {
    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    let count = (texture.width * texture.height) as f32;

    for y in 0..texture.height {
        for x in 0..texture.width {
            let p = texture.get_pixel(x, y);
            let lum = (p[0] as f32 * 0.299 + p[1] as f32 * 0.587 + p[2] as f32 * 0.114) / 255.0;
            sum += lum;
            sum_sq += lum * lum;
        }
    }

    let mean = sum / count;
    let variance = ((sum_sq / count) - (mean * mean)).max(0.0);  // Avoid negative due to float precision
    let std_dev = variance.sqrt();

    // Normalize to 0-1 range (std_dev of 0.5 = max contrast)
    // Handle NaN case (shouldn't happen but be safe)
    let result = (std_dev * 2.0).min(1.0);
    if result.is_nan() { 0.0 } else { result }
}

/// Count unique colors in the texture
fn count_unique_colors(texture: &TextureBuffer) -> u32 {
    let mut colors = HashSet::new();

    for y in 0..texture.height {
        for x in 0..texture.width {
            let p = texture.get_pixel(x, y);
            // Quantize to reduce near-duplicates
            let key = ((p[0] >> 2) as u32) << 16 | ((p[1] >> 2) as u32) << 8 | (p[2] >> 2) as u32;
            colors.insert(key);
        }
    }

    colors.len() as u32
}

/// Calculate histogram balance (how evenly distributed pixel values are)
fn calculate_histogram_balance(texture: &TextureBuffer) -> f32 {
    let mut histogram = [0u32; 256];
    let count = texture.width * texture.height;

    for y in 0..texture.height {
        for x in 0..texture.width {
            let p = texture.get_pixel(x, y);
            let lum = ((p[0] as u32 * 77 + p[1] as u32 * 150 + p[2] as u32 * 29) / 256) as usize;
            histogram[lum] += 1;
        }
    }

    // Calculate how evenly distributed the histogram is
    let expected = count as f32 / 256.0;
    let mut deviation = 0.0;

    for &bin in &histogram {
        let diff = (bin as f32 - expected).abs();
        deviation += diff;
    }

    // Normalize: lower deviation = more balanced
    let max_deviation = count as f32 * 2.0; // Worst case: all in one bin
    1.0 - (deviation / max_deviation).min(1.0)
}

/// Calculate noise coherence (how structured vs random the patterns are)
fn calculate_noise_coherence(texture: &TextureBuffer) -> f32 {
    if texture.width < 3 || texture.height < 3 {
        return 0.5;
    }

    // Measure local correlation: how similar each pixel is to its neighbors
    let mut coherence_sum = 0.0;
    let mut count = 0.0;

    for y in 1..texture.height - 1 {
        for x in 1..texture.width - 1 {
            let center = texture.get_pixel(x, y);
            let center_lum = (center[0] as f32 + center[1] as f32 + center[2] as f32) / 3.0;

            // Sample 4 neighbors
            let neighbors = [
                texture.get_pixel(x - 1, y),
                texture.get_pixel(x + 1, y),
                texture.get_pixel(x, y - 1),
                texture.get_pixel(x, y + 1),
            ];

            let mut neighbor_diff = 0.0;
            for n in &neighbors {
                let n_lum = (n[0] as f32 + n[1] as f32 + n[2] as f32) / 3.0;
                neighbor_diff += (center_lum - n_lum).abs();
            }
            neighbor_diff /= 4.0;

            // Lower difference = more coherent
            coherence_sum += 1.0 - (neighbor_diff / 128.0).min(1.0);
            count += 1.0;
        }
    }

    if count > 0.0 { coherence_sum / count } else { 0.5 }
}

/// Calculate tileability (how well the texture tiles)
fn calculate_tileability(texture: &TextureBuffer) -> f32 {
    1.0 - calculate_seam_visibility(texture)
}

/// Calculate seam visibility when tiling
fn calculate_seam_visibility(texture: &TextureBuffer) -> f32 {
    if texture.width < 2 || texture.height < 2 {
        return 0.0;
    }

    let mut horizontal_diff = 0.0;
    let mut vertical_diff = 0.0;
    let mut internal_diff = 0.0;

    // Check horizontal seam (left edge vs right edge)
    for y in 0..texture.height {
        let left = texture.get_pixel(0, y);
        let right = texture.get_pixel(texture.width - 1, y);
        horizontal_diff += color_distance(left, right);
    }
    horizontal_diff /= texture.height as f32;

    // Check vertical seam (top edge vs bottom edge)
    for x in 0..texture.width {
        let top = texture.get_pixel(x, 0);
        let bottom = texture.get_pixel(x, texture.height - 1);
        vertical_diff += color_distance(top, bottom);
    }
    vertical_diff /= texture.width as f32;

    // Calculate internal variation for comparison
    let mut sample_count = 0;
    for y in 1..texture.height {
        for x in 1..texture.width {
            let current = texture.get_pixel(x, y);
            let prev = texture.get_pixel(x - 1, y);
            internal_diff += color_distance(current, prev);
            sample_count += 1;
        }
    }
    internal_diff /= sample_count as f32;

    // Seam visibility is how much edge difference exceeds internal variation
    let edge_avg = (horizontal_diff + vertical_diff) / 2.0;
    let visibility = if internal_diff > 0.001 {
        (edge_avg / internal_diff - 1.0).max(0.0)
    } else {
        0.0
    };

    visibility.min(1.0)
}

/// Detect if texture has multi-frequency detail
fn detect_multi_frequency_detail(texture: &TextureBuffer) -> bool {
    if texture.width < 16 || texture.height < 16 {
        return false;
    }

    // Sample at different scales
    let fine_variation = sample_variation(texture, 2);
    let medium_variation = sample_variation(texture, 8);
    let coarse_variation = sample_variation(texture, 16);

    // Multi-frequency detail means variation at multiple scales
    let has_fine = fine_variation > 5.0;
    let has_medium = medium_variation > 10.0;
    let has_coarse = coarse_variation > 15.0;

    // At least 2 scales should have significant variation
    (has_fine as u32 + has_medium as u32 + has_coarse as u32) >= 2
}

fn sample_variation(texture: &TextureBuffer, step: u32) -> f32 {
    let mut variation = 0.0;
    let mut count = 0;

    let mut y = 0;
    while y + step < texture.height {
        let mut x = 0;
        while x + step < texture.width {
            let p1 = texture.get_pixel(x, y);
            let p2 = texture.get_pixel(x + step, y + step);
            variation += color_distance(p1, p2);
            count += 1;
            x += step;
        }
        y += step;
    }

    if count > 0 { variation / count as f32 } else { 0.0 }
}

/// Detect if texture has hue variation (not just brightness)
fn detect_hue_variation(texture: &TextureBuffer) -> bool {
    use super::color::rgb_to_hsv;

    let mut hues = Vec::new();

    // Sample hues from the texture
    let step = ((texture.width * texture.height) as f32).sqrt() as u32 / 8;
    let step = step.max(1);

    let mut y = 0;
    while y < texture.height {
        let mut x = 0;
        while x < texture.width {
            let p = texture.get_pixel(x, y);
            let (h, s, _) = rgb_to_hsv(p[0], p[1], p[2]);

            // Only count if saturation is significant
            if s > 0.1 {
                hues.push(h);
            }
            x += step;
        }
        y += step;
    }

    if hues.len() < 10 {
        return false;
    }

    // Calculate hue variance
    let mean_hue: f32 = hues.iter().sum::<f32>() / hues.len() as f32;
    let variance: f32 = hues.iter().map(|h| {
        let diff = (h - mean_hue).abs();
        let diff = if diff > 180.0 { 360.0 - diff } else { diff };
        diff * diff
    }).sum::<f32>() / hues.len() as f32;

    // More than 10 degrees of hue variation
    variance.sqrt() > 10.0
}

/// Calculate color distance between two pixels
fn color_distance(a: [u8; 4], b: [u8; 4]) -> f32 {
    let dr = a[0] as f32 - b[0] as f32;
    let dg = a[1] as f32 - b[1] as f32;
    let db = a[2] as f32 - b[2] as f32;
    (dr * dr + dg * dg + db * db).sqrt()
}

impl TextureQualityReport {
    /// Check if the texture passes quality requirements
    pub fn is_acceptable(&self) -> bool {
        !self.issues.iter().any(|i| i.severity == IssueSeverity::Critical)
    }

    /// Check if the texture passes without any errors
    pub fn is_good(&self) -> bool {
        !self.issues.iter().any(|i| matches!(i.severity, IssueSeverity::Critical | IssueSeverity::Error))
    }

    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        let status = if self.is_good() {
            "PASS"
        } else if self.is_acceptable() {
            "ACCEPTABLE"
        } else {
            "FAIL"
        };

        let mut result = format!("Quality: {} (score: {:.1}%)\n", status, self.score * 100.0);

        if !self.passed.is_empty() {
            result.push_str(&format!("  Passed: {}\n", self.passed.join(", ")));
        }

        for issue in &self.issues {
            let severity = match issue.severity {
                IssueSeverity::Warning => "[WARN]",
                IssueSeverity::Error => "[ERROR]",
                IssueSeverity::Critical => "[CRITICAL]",
            };
            result.push_str(&format!("  {} {}\n", severity, issue.message));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solid_gray_fails_quality() {
        // Solid gray should fail multiple checks
        let texture = TextureBuffer::filled(64, 64, [128, 128, 128, 255]);
        let report = assess_quality(&texture, &QualityThresholds::default());

        assert!(!report.is_good());
        // Solid color should have very low contrast (near 0)
        assert!(report.metrics.contrast < 0.05, "Contrast was: {}", report.metrics.contrast);
        // Solid color has exactly 1 unique color
        assert!(report.metrics.unique_colors <= 1, "Unique colors: {}", report.metrics.unique_colors);
    }

    #[test]
    fn test_high_contrast_passes() {
        // Create a texture with contrast
        let mut texture = TextureBuffer::new(64, 64);
        for y in 0..64 {
            for x in 0..64 {
                let value = if (x + y) % 2 == 0 { 200 } else { 50 };
                texture.set_pixel(x, y, [value, value, value, 255]);
            }
        }

        let report = assess_quality(&texture, &QualityThresholds::default());
        assert!(report.metrics.contrast > 0.15);
    }

    #[test]
    fn test_quality_summary() {
        let texture = TextureBuffer::filled(64, 64, [128, 128, 128, 255]);
        let report = assess_quality(&texture, &QualityThresholds::default());
        let summary = report.summary();

        assert!(summary.contains("FAIL") || summary.contains("ACCEPTABLE"));
        assert!(summary.contains("Quality"));
    }

    #[test]
    fn test_seam_visibility_on_tileable() {
        // A properly tileable texture should have low seam visibility
        let mut texture = TextureBuffer::new(32, 32);
        for y in 0..32 {
            for x in 0..32 {
                let value = ((x + y) % 32 * 8) as u8;
                texture.set_pixel(x, y, [value, value, value, 255]);
            }
        }

        let visibility = calculate_seam_visibility(&texture);
        // This specific pattern tiles well
        assert!(visibility < 0.5);
    }
}
