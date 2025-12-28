//! Style tokens for modifying generation parameters
//!
//! Style tokens adjust base generation parameters to achieve consistent
//! visual styles across assets.

/// Style token representing a visual aesthetic
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StyleToken {
    // Natural/Historical
    Rustic,
    Medieval,
    Ancient,
    Victorian,

    // Modern/Futuristic
    Cyberpunk,
    #[default]
    Scifi,
    Industrial,
    Minimalist,

    // Organic/Natural
    Organic,
    Overgrown,
    Crystalline,
    Elemental,

    // Stylized
    Fantasy,
    Gothic,
    Steampunk,
    Dieselpunk,

    // Abstract/Artistic
    Geometric,
    Abstract,
    Baroque,
    ArtDeco,

    // Condition-based
    PostApoc,
    Pristine,
    Corrupted,
    Ethereal,
}

impl StyleToken {
    /// Get style modifiers for this token
    pub fn modifiers(&self) -> StyleModifiers {
        match self {
            // Natural/Historical
            StyleToken::Rustic => StyleModifiers {
                roughness_offset: 0.2,
                saturation_scale: 0.8,
                detail_level: DetailLevel::Medium,
                edge_hardness: 0.3,
                noise_octaves_offset: 1,
                damage_amount: 0.3,
                color_temperature: 0.2,
                pattern_scale: 1.0,
                emission_tendency: 0.0,
            },
            StyleToken::Medieval => StyleModifiers {
                roughness_offset: 0.15,
                saturation_scale: 0.7,
                detail_level: DetailLevel::High,
                edge_hardness: 0.5,
                noise_octaves_offset: 1,
                damage_amount: 0.2,
                color_temperature: 0.1,
                pattern_scale: 0.8,
                emission_tendency: 0.0,
            },
            StyleToken::Ancient => StyleModifiers {
                roughness_offset: 0.4,
                saturation_scale: 0.5,
                detail_level: DetailLevel::Medium,
                edge_hardness: 0.2,
                noise_octaves_offset: 2,
                damage_amount: 0.6,
                color_temperature: -0.1,
                pattern_scale: 1.2,
                emission_tendency: 0.0,
            },
            StyleToken::Victorian => StyleModifiers {
                roughness_offset: 0.1,
                saturation_scale: 0.6,
                detail_level: DetailLevel::Extreme,
                edge_hardness: 0.7,
                noise_octaves_offset: 0,
                damage_amount: 0.1,
                color_temperature: -0.2,
                pattern_scale: 0.6,
                emission_tendency: 0.0,
            },

            // Modern/Futuristic
            StyleToken::Cyberpunk => StyleModifiers {
                roughness_offset: -0.1,
                saturation_scale: 1.3,
                detail_level: DetailLevel::High,
                edge_hardness: 0.8,
                noise_octaves_offset: 0,
                damage_amount: 0.3,
                color_temperature: -0.3,
                pattern_scale: 0.5,
                emission_tendency: 0.6,
            },
            StyleToken::Scifi => StyleModifiers {
                roughness_offset: -0.2,
                saturation_scale: 0.9,
                detail_level: DetailLevel::High,
                edge_hardness: 0.9,
                noise_octaves_offset: -1,
                damage_amount: 0.0,
                color_temperature: -0.2,
                pattern_scale: 0.7,
                emission_tendency: 0.3,
            },
            StyleToken::Industrial => StyleModifiers {
                roughness_offset: 0.2,
                saturation_scale: 0.6,
                detail_level: DetailLevel::Medium,
                edge_hardness: 0.6,
                noise_octaves_offset: 1,
                damage_amount: 0.4,
                color_temperature: 0.0,
                pattern_scale: 1.0,
                emission_tendency: 0.1,
            },
            StyleToken::Minimalist => StyleModifiers {
                roughness_offset: -0.1,
                saturation_scale: 0.5,
                detail_level: DetailLevel::Low,
                edge_hardness: 0.95,
                noise_octaves_offset: -2,
                damage_amount: 0.0,
                color_temperature: 0.0,
                pattern_scale: 2.0,
                emission_tendency: 0.0,
            },

            // Organic/Natural
            StyleToken::Organic => StyleModifiers {
                roughness_offset: 0.1,
                saturation_scale: 0.9,
                detail_level: DetailLevel::High,
                edge_hardness: 0.2,
                noise_octaves_offset: 2,
                damage_amount: 0.1,
                color_temperature: 0.1,
                pattern_scale: 1.0,
                emission_tendency: 0.0,
            },
            StyleToken::Overgrown => StyleModifiers {
                roughness_offset: 0.3,
                saturation_scale: 1.1,
                detail_level: DetailLevel::Extreme,
                edge_hardness: 0.1,
                noise_octaves_offset: 3,
                damage_amount: 0.4,
                color_temperature: 0.1,
                pattern_scale: 0.8,
                emission_tendency: 0.0,
            },
            StyleToken::Crystalline => StyleModifiers {
                roughness_offset: -0.3,
                saturation_scale: 1.2,
                detail_level: DetailLevel::High,
                edge_hardness: 1.0,
                noise_octaves_offset: 0,
                damage_amount: 0.0,
                color_temperature: -0.2,
                pattern_scale: 0.5,
                emission_tendency: 0.4,
            },
            StyleToken::Elemental => StyleModifiers {
                roughness_offset: 0.0,
                saturation_scale: 1.4,
                detail_level: DetailLevel::Medium,
                edge_hardness: 0.5,
                noise_octaves_offset: 2,
                damage_amount: 0.2,
                color_temperature: 0.0,
                pattern_scale: 1.2,
                emission_tendency: 0.5,
            },

            // Stylized
            StyleToken::Fantasy => StyleModifiers {
                roughness_offset: 0.0,
                saturation_scale: 1.2,
                detail_level: DetailLevel::High,
                edge_hardness: 0.4,
                noise_octaves_offset: 1,
                damage_amount: 0.1,
                color_temperature: 0.1,
                pattern_scale: 0.9,
                emission_tendency: 0.2,
            },
            StyleToken::Gothic => StyleModifiers {
                roughness_offset: 0.1,
                saturation_scale: 0.6,
                detail_level: DetailLevel::Extreme,
                edge_hardness: 0.7,
                noise_octaves_offset: 1,
                damage_amount: 0.3,
                color_temperature: -0.3,
                pattern_scale: 0.7,
                emission_tendency: 0.0,
            },
            StyleToken::Steampunk => StyleModifiers {
                roughness_offset: 0.15,
                saturation_scale: 0.8,
                detail_level: DetailLevel::Extreme,
                edge_hardness: 0.6,
                noise_octaves_offset: 1,
                damage_amount: 0.25,
                color_temperature: 0.2,
                pattern_scale: 0.6,
                emission_tendency: 0.1,
            },
            StyleToken::Dieselpunk => StyleModifiers {
                roughness_offset: 0.2,
                saturation_scale: 0.7,
                detail_level: DetailLevel::High,
                edge_hardness: 0.5,
                noise_octaves_offset: 1,
                damage_amount: 0.35,
                color_temperature: 0.1,
                pattern_scale: 0.8,
                emission_tendency: 0.05,
            },

            // Abstract/Artistic
            StyleToken::Geometric => StyleModifiers {
                roughness_offset: -0.1,
                saturation_scale: 1.0,
                detail_level: DetailLevel::Low,
                edge_hardness: 1.0,
                noise_octaves_offset: -2,
                damage_amount: 0.0,
                color_temperature: 0.0,
                pattern_scale: 1.0,
                emission_tendency: 0.0,
            },
            StyleToken::Abstract => StyleModifiers {
                roughness_offset: 0.0,
                saturation_scale: 1.3,
                detail_level: DetailLevel::Medium,
                edge_hardness: 0.5,
                noise_octaves_offset: 1,
                damage_amount: 0.1,
                color_temperature: 0.0,
                pattern_scale: 1.5,
                emission_tendency: 0.2,
            },
            StyleToken::Baroque => StyleModifiers {
                roughness_offset: 0.05,
                saturation_scale: 0.9,
                detail_level: DetailLevel::Extreme,
                edge_hardness: 0.4,
                noise_octaves_offset: 2,
                damage_amount: 0.15,
                color_temperature: 0.2,
                pattern_scale: 0.5,
                emission_tendency: 0.0,
            },
            StyleToken::ArtDeco => StyleModifiers {
                roughness_offset: -0.1,
                saturation_scale: 0.8,
                detail_level: DetailLevel::High,
                edge_hardness: 0.9,
                noise_octaves_offset: -1,
                damage_amount: 0.0,
                color_temperature: 0.1,
                pattern_scale: 0.7,
                emission_tendency: 0.15,
            },

            // Condition-based
            StyleToken::PostApoc => StyleModifiers {
                roughness_offset: 0.4,
                saturation_scale: 0.5,
                detail_level: DetailLevel::High,
                edge_hardness: 0.3,
                noise_octaves_offset: 2,
                damage_amount: 0.7,
                color_temperature: -0.1,
                pattern_scale: 1.2,
                emission_tendency: 0.0,
            },
            StyleToken::Pristine => StyleModifiers {
                roughness_offset: -0.2,
                saturation_scale: 1.0,
                detail_level: DetailLevel::Medium,
                edge_hardness: 0.8,
                noise_octaves_offset: -1,
                damage_amount: 0.0,
                color_temperature: 0.0,
                pattern_scale: 1.0,
                emission_tendency: 0.0,
            },
            StyleToken::Corrupted => StyleModifiers {
                roughness_offset: 0.2,
                saturation_scale: 0.7,
                detail_level: DetailLevel::High,
                edge_hardness: 0.4,
                noise_octaves_offset: 2,
                damage_amount: 0.5,
                color_temperature: -0.4,
                pattern_scale: 1.3,
                emission_tendency: 0.3,
            },
            StyleToken::Ethereal => StyleModifiers {
                roughness_offset: -0.2,
                saturation_scale: 0.6,
                detail_level: DetailLevel::Low,
                edge_hardness: 0.2,
                noise_octaves_offset: 0,
                damage_amount: 0.0,
                color_temperature: -0.1,
                pattern_scale: 1.5,
                emission_tendency: 0.5,
            },
        }
    }

    /// Parse style token from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rustic" => Some(StyleToken::Rustic),
            "medieval" => Some(StyleToken::Medieval),
            "ancient" => Some(StyleToken::Ancient),
            "victorian" => Some(StyleToken::Victorian),
            "cyberpunk" => Some(StyleToken::Cyberpunk),
            "scifi" | "sci-fi" => Some(StyleToken::Scifi),
            "industrial" => Some(StyleToken::Industrial),
            "minimalist" => Some(StyleToken::Minimalist),
            "organic" => Some(StyleToken::Organic),
            "overgrown" => Some(StyleToken::Overgrown),
            "crystalline" | "crystal" => Some(StyleToken::Crystalline),
            "elemental" => Some(StyleToken::Elemental),
            "fantasy" => Some(StyleToken::Fantasy),
            "gothic" => Some(StyleToken::Gothic),
            "steampunk" => Some(StyleToken::Steampunk),
            "dieselpunk" => Some(StyleToken::Dieselpunk),
            "geometric" => Some(StyleToken::Geometric),
            "abstract" => Some(StyleToken::Abstract),
            "baroque" => Some(StyleToken::Baroque),
            "artdeco" | "art-deco" => Some(StyleToken::ArtDeco),
            "postapoc" | "post-apocalyptic" | "post-apoc" => Some(StyleToken::PostApoc),
            "pristine" | "new" | "clean" => Some(StyleToken::Pristine),
            "corrupted" | "corrupt" => Some(StyleToken::Corrupted),
            "ethereal" | "ghost" | "ghostly" => Some(StyleToken::Ethereal),
            _ => None,
        }
    }
}

/// Detail level for generation
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DetailLevel {
    /// Minimal detail, smooth surfaces
    Low,
    /// Balanced detail
    #[default]
    Medium,
    /// Rich detail
    High,
    /// Maximum detail, ornate
    Extreme,
}

impl DetailLevel {
    /// Get recommended noise octaves for this detail level
    pub fn noise_octaves(&self) -> u32 {
        match self {
            DetailLevel::Low => 1,
            DetailLevel::Medium => 2,
            DetailLevel::High => 3,
            DetailLevel::Extreme => 4,
        }
    }

    /// Get recommended subdivision level
    pub fn subdivision_level(&self) -> u32 {
        match self {
            DetailLevel::Low => 0,
            DetailLevel::Medium => 1,
            DetailLevel::High => 2,
            DetailLevel::Extreme => 3,
        }
    }

    /// Get recommended texture resolution multiplier
    pub fn texture_resolution_multiplier(&self) -> f32 {
        match self {
            DetailLevel::Low => 0.5,
            DetailLevel::Medium => 1.0,
            DetailLevel::High => 1.5,
            DetailLevel::Extreme => 2.0,
        }
    }
}

/// Style modifiers that affect generation parameters
#[derive(Clone, Copy, Debug)]
pub struct StyleModifiers {
    /// Roughness offset (-0.3 to +0.5)
    pub roughness_offset: f32,
    /// Saturation multiplier (0.5 to 1.5)
    pub saturation_scale: f32,
    /// Detail level
    pub detail_level: DetailLevel,
    /// Edge hardness (0.0 soft to 1.0 sharp)
    pub edge_hardness: f32,
    /// Noise octaves adjustment (-2 to +3)
    pub noise_octaves_offset: i32,
    /// Damage/weathering amount (0.0 pristine to 1.0 destroyed)
    pub damage_amount: f32,
    /// Color temperature shift (-1.0 cool to +1.0 warm)
    pub color_temperature: f32,
    /// Pattern scale multiplier
    pub pattern_scale: f32,
    /// Likelihood of emissive elements (0.0 to 1.0)
    pub emission_tendency: f32,
}

impl Default for StyleModifiers {
    fn default() -> Self {
        Self {
            roughness_offset: 0.0,
            saturation_scale: 1.0,
            detail_level: DetailLevel::Medium,
            edge_hardness: 0.5,
            noise_octaves_offset: 0,
            damage_amount: 0.0,
            color_temperature: 0.0,
            pattern_scale: 1.0,
            emission_tendency: 0.0,
        }
    }
}

impl StyleModifiers {
    /// Blend two style modifiers
    pub fn blend(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let inv_t = 1.0 - t;

        Self {
            roughness_offset: self.roughness_offset * inv_t + other.roughness_offset * t,
            saturation_scale: self.saturation_scale * inv_t + other.saturation_scale * t,
            detail_level: if t < 0.5 { self.detail_level } else { other.detail_level },
            edge_hardness: self.edge_hardness * inv_t + other.edge_hardness * t,
            noise_octaves_offset: if t < 0.5 { self.noise_octaves_offset } else { other.noise_octaves_offset },
            damage_amount: self.damage_amount * inv_t + other.damage_amount * t,
            color_temperature: self.color_temperature * inv_t + other.color_temperature * t,
            pattern_scale: self.pattern_scale * inv_t + other.pattern_scale * t,
            emission_tendency: self.emission_tendency * inv_t + other.emission_tendency * t,
        }
    }

    /// Apply damage modifier
    pub fn with_damage(mut self, amount: f32) -> Self {
        self.damage_amount = amount.clamp(0.0, 1.0);
        // Damage increases roughness and decreases saturation
        self.roughness_offset += amount * 0.2;
        self.saturation_scale *= 1.0 - amount * 0.3;
        self
    }

    /// Get effective noise octaves
    pub fn effective_noise_octaves(&self, base: u32) -> u32 {
        let adjusted = base as i32 + self.noise_octaves_offset;
        adjusted.max(1) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_token_modifiers() {
        let mods = StyleToken::Rustic.modifiers();
        assert!(mods.roughness_offset > 0.0);
        assert!(mods.damage_amount > 0.0);

        let mods = StyleToken::Pristine.modifiers();
        assert!(mods.damage_amount == 0.0);
        assert!(mods.roughness_offset < 0.0);
    }

    #[test]
    fn test_style_token_from_str() {
        assert_eq!(StyleToken::from_str("rustic"), Some(StyleToken::Rustic));
        assert_eq!(StyleToken::from_str("CYBERPUNK"), Some(StyleToken::Cyberpunk));
        assert_eq!(StyleToken::from_str("sci-fi"), Some(StyleToken::Scifi));
        assert_eq!(StyleToken::from_str("unknown"), None);
    }

    #[test]
    fn test_modifier_blend() {
        let a = StyleToken::Pristine.modifiers();
        let b = StyleToken::PostApoc.modifiers();
        let blended = a.blend(&b, 0.5);

        // Should be between the two
        assert!(blended.damage_amount > a.damage_amount);
        assert!(blended.damage_amount < b.damage_amount);
    }
}
