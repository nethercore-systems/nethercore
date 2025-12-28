//! Material database with PBR parameters
//!
//! Maps semantic material descriptors to physically-based rendering parameters.

/// Material definition with PBR parameters
#[derive(Clone, Copy, Debug)]
pub struct Material {
    /// Base RGB color (0-255)
    pub base_color: [u8; 3],
    /// Metallic factor (0.0 = dielectric, 1.0 = metal)
    pub metallic: f32,
    /// Roughness factor (0.0 = mirror, 1.0 = rough)
    pub roughness: f32,
    /// Normal map strength (0.0 - 2.0)
    pub normal_strength: f32,
    /// Ambient occlusion strength (0.0 - 1.0)
    pub ao_strength: f32,
    /// Emission intensity (0.0 = none, >0 = emissive)
    pub emission: f32,
    /// Index of refraction (1.0 - 3.0)
    pub ior: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: [128, 128, 128],
            metallic: 0.0,
            roughness: 0.5,
            normal_strength: 1.0,
            ao_strength: 1.0,
            emission: 0.0,
            ior: 1.5,
        }
    }
}

impl Material {
    /// Look up material by semantic descriptor (e.g., "metal.polished")
    pub fn lookup(descriptor: &str) -> Option<Self> {
        let parts: Vec<&str> = descriptor.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        let category = parts[0].to_lowercase();
        let variant = parts.get(1).map(|s| s.to_lowercase()).unwrap_or_default();

        match category.as_str() {
            "metal" => Some(Self::metal(&variant)),
            "wood" => Some(Self::wood(&variant)),
            "stone" => Some(Self::stone(&variant)),
            "fabric" => Some(Self::fabric(&variant)),
            "leather" => Some(Self::leather(&variant)),
            "plastic" => Some(Self::plastic(&variant)),
            "organic" => Some(Self::organic(&variant)),
            "crystal" => Some(Self::crystal(&variant)),
            "tech" => Some(Self::tech(&variant)),
            "concrete" => Some(Self::concrete(&variant)),
            _ => None,
        }
    }

    /// Metal materials
    pub fn metal(variant: &str) -> Self {
        match variant {
            "polished" => Self {
                base_color: [200, 200, 205],
                metallic: 1.0,
                roughness: 0.1,
                normal_strength: 0.5,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 2.5,
            },
            "brushed" => Self {
                base_color: [180, 180, 185],
                metallic: 1.0,
                roughness: 0.35,
                normal_strength: 0.8,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 2.5,
            },
            "chrome" => Self {
                base_color: [220, 220, 225],
                metallic: 1.0,
                roughness: 0.05,
                normal_strength: 0.3,
                ao_strength: 0.8,
                emission: 0.0,
                ior: 2.9,
            },
            "iron" => Self {
                base_color: [120, 115, 110],
                metallic: 0.95,
                roughness: 0.5,
                normal_strength: 1.0,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 2.2,
            },
            "rusted" => Self {
                base_color: [140, 80, 50],
                metallic: 0.3,
                roughness: 0.8,
                normal_strength: 1.5,
                ao_strength: 1.2,
                emission: 0.0,
                ior: 1.8,
            },
            "oxidized" => Self {
                base_color: [70, 120, 100],
                metallic: 0.4,
                roughness: 0.6,
                normal_strength: 1.2,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.9,
            },
            "gold" => Self {
                base_color: [255, 200, 100],
                metallic: 1.0,
                roughness: 0.2,
                normal_strength: 0.6,
                ao_strength: 0.9,
                emission: 0.0,
                ior: 2.5,
            },
            "copper" => Self {
                base_color: [200, 120, 80],
                metallic: 1.0,
                roughness: 0.3,
                normal_strength: 0.7,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 2.4,
            },
            "brass" => Self {
                base_color: [180, 160, 80],
                metallic: 1.0,
                roughness: 0.25,
                normal_strength: 0.6,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 2.3,
            },
            "bronze" => Self {
                base_color: [140, 100, 60],
                metallic: 1.0,
                roughness: 0.4,
                normal_strength: 0.8,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 2.2,
            },
            "painted" => Self {
                base_color: [100, 100, 100],
                metallic: 0.1,
                roughness: 0.4,
                normal_strength: 0.8,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.5,
            },
            "riveted" => Self {
                base_color: [140, 140, 145],
                metallic: 0.9,
                roughness: 0.5,
                normal_strength: 1.2,
                ao_strength: 1.2,
                emission: 0.0,
                ior: 2.3,
            },
            _ => Self {
                base_color: [160, 160, 165],
                metallic: 1.0,
                roughness: 0.4,
                normal_strength: 0.8,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 2.3,
            },
        }
    }

    /// Wood materials
    pub fn wood(variant: &str) -> Self {
        match variant {
            "fresh" => Self {
                base_color: [180, 140, 90],
                metallic: 0.0,
                roughness: 0.5,
                normal_strength: 0.8,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.5,
            },
            "weathered" => Self {
                base_color: [120, 100, 70],
                metallic: 0.0,
                roughness: 0.7,
                normal_strength: 1.2,
                ao_strength: 1.1,
                emission: 0.0,
                ior: 1.5,
            },
            "polished" => Self {
                base_color: [140, 90, 50],
                metallic: 0.0,
                roughness: 0.2,
                normal_strength: 0.6,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.5,
            },
            "painted" => Self {
                base_color: [100, 100, 100],
                metallic: 0.0,
                roughness: 0.4,
                normal_strength: 0.5,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.5,
            },
            "charred" => Self {
                base_color: [40, 35, 30],
                metallic: 0.0,
                roughness: 0.9,
                normal_strength: 1.5,
                ao_strength: 1.3,
                emission: 0.0,
                ior: 1.4,
            },
            "mossy" => Self {
                base_color: [80, 100, 60],
                metallic: 0.0,
                roughness: 0.8,
                normal_strength: 1.3,
                ao_strength: 1.2,
                emission: 0.0,
                ior: 1.5,
            },
            "oak" => Self {
                base_color: [160, 120, 70],
                metallic: 0.0,
                roughness: 0.5,
                normal_strength: 0.9,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.5,
            },
            "mahogany" => Self {
                base_color: [100, 50, 30],
                metallic: 0.0,
                roughness: 0.3,
                normal_strength: 0.7,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.5,
            },
            "ebony" => Self {
                base_color: [30, 25, 20],
                metallic: 0.0,
                roughness: 0.25,
                normal_strength: 0.5,
                ao_strength: 0.9,
                emission: 0.0,
                ior: 1.5,
            },
            "rotting" => Self {
                base_color: [70, 60, 40],
                metallic: 0.0,
                roughness: 0.9,
                normal_strength: 1.8,
                ao_strength: 1.4,
                emission: 0.0,
                ior: 1.4,
            },
            _ => Self {
                base_color: [150, 110, 70],
                metallic: 0.0,
                roughness: 0.55,
                normal_strength: 0.9,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.5,
            },
        }
    }

    /// Stone materials
    pub fn stone(variant: &str) -> Self {
        match variant {
            "rough" => Self {
                base_color: [140, 130, 120],
                metallic: 0.0,
                roughness: 0.85,
                normal_strength: 1.3,
                ao_strength: 1.2,
                emission: 0.0,
                ior: 1.5,
            },
            "polished" => Self {
                base_color: [100, 95, 90],
                metallic: 0.0,
                roughness: 0.15,
                normal_strength: 0.4,
                ao_strength: 0.9,
                emission: 0.0,
                ior: 1.5,
            },
            "mossy" => Self {
                base_color: [90, 110, 80],
                metallic: 0.0,
                roughness: 0.75,
                normal_strength: 1.4,
                ao_strength: 1.2,
                emission: 0.0,
                ior: 1.5,
            },
            "cracked" => Self {
                base_color: [120, 115, 105],
                metallic: 0.0,
                roughness: 0.8,
                normal_strength: 1.6,
                ao_strength: 1.3,
                emission: 0.0,
                ior: 1.5,
            },
            "marble" => Self {
                base_color: [235, 230, 225],
                metallic: 0.0,
                roughness: 0.2,
                normal_strength: 0.5,
                ao_strength: 0.9,
                emission: 0.0,
                ior: 1.6,
            },
            "dark" => Self {
                base_color: [50, 45, 40],
                metallic: 0.0,
                roughness: 0.6,
                normal_strength: 1.0,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.5,
            },
            "sandstone" => Self {
                base_color: [200, 170, 130],
                metallic: 0.0,
                roughness: 0.7,
                normal_strength: 1.1,
                ao_strength: 1.1,
                emission: 0.0,
                ior: 1.5,
            },
            _ => Self {
                base_color: [130, 125, 115],
                metallic: 0.0,
                roughness: 0.7,
                normal_strength: 1.1,
                ao_strength: 1.1,
                emission: 0.0,
                ior: 1.5,
            },
        }
    }

    /// Fabric materials
    pub fn fabric(variant: &str) -> Self {
        match variant {
            "cotton" => Self {
                base_color: [230, 225, 220],
                metallic: 0.0,
                roughness: 0.8,
                normal_strength: 0.6,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.3,
            },
            "silk" => Self {
                base_color: [200, 180, 160],
                metallic: 0.0,
                roughness: 0.3,
                normal_strength: 0.4,
                ao_strength: 0.9,
                emission: 0.0,
                ior: 1.4,
            },
            "velvet" => Self {
                base_color: [80, 40, 60],
                metallic: 0.0,
                roughness: 0.9,
                normal_strength: 0.8,
                ao_strength: 1.1,
                emission: 0.0,
                ior: 1.4,
            },
            "wool" => Self {
                base_color: [180, 170, 150],
                metallic: 0.0,
                roughness: 0.95,
                normal_strength: 0.9,
                ao_strength: 1.1,
                emission: 0.0,
                ior: 1.3,
            },
            "burlap" => Self {
                base_color: [160, 140, 100],
                metallic: 0.0,
                roughness: 0.95,
                normal_strength: 1.2,
                ao_strength: 1.2,
                emission: 0.0,
                ior: 1.3,
            },
            "synthetic" => Self {
                base_color: [150, 150, 160],
                metallic: 0.0,
                roughness: 0.4,
                normal_strength: 0.5,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.4,
            },
            _ => Self {
                base_color: [200, 190, 180],
                metallic: 0.0,
                roughness: 0.7,
                normal_strength: 0.6,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.35,
            },
        }
    }

    /// Leather materials
    pub fn leather(variant: &str) -> Self {
        match variant {
            "brown" => Self {
                base_color: [80, 50, 30],
                metallic: 0.0,
                roughness: 0.5,
                normal_strength: 0.8,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.4,
            },
            "worn" => Self {
                base_color: [100, 70, 50],
                metallic: 0.0,
                roughness: 0.7,
                normal_strength: 1.1,
                ao_strength: 1.1,
                emission: 0.0,
                ior: 1.4,
            },
            "fine" => Self {
                base_color: [60, 30, 20],
                metallic: 0.0,
                roughness: 0.3,
                normal_strength: 0.5,
                ao_strength: 0.9,
                emission: 0.0,
                ior: 1.5,
            },
            _ => Self {
                base_color: [90, 55, 35],
                metallic: 0.0,
                roughness: 0.5,
                normal_strength: 0.8,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.4,
            },
        }
    }

    /// Plastic materials
    pub fn plastic(variant: &str) -> Self {
        match variant {
            "glossy" => Self {
                base_color: [200, 50, 50],
                metallic: 0.0,
                roughness: 0.1,
                normal_strength: 0.3,
                ao_strength: 0.8,
                emission: 0.0,
                ior: 1.5,
            },
            "matte" => Self {
                base_color: [180, 180, 180],
                metallic: 0.0,
                roughness: 0.6,
                normal_strength: 0.5,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.5,
            },
            "rubber" => Self {
                base_color: [40, 40, 45],
                metallic: 0.0,
                roughness: 0.85,
                normal_strength: 0.7,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.5,
            },
            "translucent" => Self {
                base_color: [220, 220, 230],
                metallic: 0.0,
                roughness: 0.2,
                normal_strength: 0.3,
                ao_strength: 0.7,
                emission: 0.0,
                ior: 1.6,
            },
            _ => Self {
                base_color: [180, 180, 190],
                metallic: 0.0,
                roughness: 0.4,
                normal_strength: 0.4,
                ao_strength: 0.9,
                emission: 0.0,
                ior: 1.5,
            },
        }
    }

    /// Organic materials
    pub fn organic(variant: &str) -> Self {
        match variant {
            "skin" => Self {
                base_color: [200, 160, 140],
                metallic: 0.0,
                roughness: 0.5,
                normal_strength: 0.6,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.4,
            },
            "bark" => Self {
                base_color: [80, 60, 40],
                metallic: 0.0,
                roughness: 0.9,
                normal_strength: 1.5,
                ao_strength: 1.3,
                emission: 0.0,
                ior: 1.5,
            },
            "bone" => Self {
                base_color: [230, 220, 200],
                metallic: 0.0,
                roughness: 0.4,
                normal_strength: 0.7,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.5,
            },
            "chitin" => Self {
                base_color: [50, 45, 30],
                metallic: 0.1,
                roughness: 0.3,
                normal_strength: 0.8,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.6,
            },
            "coral" => Self {
                base_color: [230, 150, 140],
                metallic: 0.0,
                roughness: 0.6,
                normal_strength: 1.0,
                ao_strength: 1.1,
                emission: 0.0,
                ior: 1.5,
            },
            "moss" => Self {
                base_color: [60, 90, 40],
                metallic: 0.0,
                roughness: 0.95,
                normal_strength: 1.2,
                ao_strength: 1.2,
                emission: 0.0,
                ior: 1.3,
            },
            "corrupted" => Self {
                base_color: [60, 40, 70],
                metallic: 0.1,
                roughness: 0.6,
                normal_strength: 1.3,
                ao_strength: 1.1,
                emission: 0.2,
                ior: 1.5,
            },
            _ => Self {
                base_color: [150, 130, 110],
                metallic: 0.0,
                roughness: 0.6,
                normal_strength: 0.9,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.4,
            },
        }
    }

    /// Crystal materials
    pub fn crystal(variant: &str) -> Self {
        match variant {
            "clear" => Self {
                base_color: [240, 245, 250],
                metallic: 0.0,
                roughness: 0.05,
                normal_strength: 0.3,
                ao_strength: 0.6,
                emission: 0.0,
                ior: 2.0,
            },
            "colored" => Self {
                base_color: [100, 150, 200],
                metallic: 0.0,
                roughness: 0.1,
                normal_strength: 0.4,
                ao_strength: 0.7,
                emission: 0.0,
                ior: 1.9,
            },
            "magical" => Self {
                base_color: [150, 100, 200],
                metallic: 0.0,
                roughness: 0.15,
                normal_strength: 0.5,
                ao_strength: 0.6,
                emission: 0.5,
                ior: 2.0,
            },
            "corrupted" => Self {
                base_color: [80, 50, 100],
                metallic: 0.1,
                roughness: 0.25,
                normal_strength: 0.8,
                ao_strength: 0.8,
                emission: 0.3,
                ior: 1.8,
            },
            _ => Self {
                base_color: [220, 230, 240],
                metallic: 0.0,
                roughness: 0.1,
                normal_strength: 0.4,
                ao_strength: 0.7,
                emission: 0.0,
                ior: 1.9,
            },
        }
    }

    /// Tech materials
    pub fn tech(variant: &str) -> Self {
        match variant {
            "screen" => Self {
                base_color: [20, 30, 40],
                metallic: 0.0,
                roughness: 0.1,
                normal_strength: 0.2,
                ao_strength: 0.5,
                emission: 0.8,
                ior: 1.5,
            },
            "panel" => Self {
                base_color: [60, 65, 70],
                metallic: 0.3,
                roughness: 0.4,
                normal_strength: 0.6,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.5,
            },
            "circuit" => Self {
                base_color: [30, 80, 30],
                metallic: 0.4,
                roughness: 0.3,
                normal_strength: 0.8,
                ao_strength: 1.0,
                emission: 0.1,
                ior: 1.6,
            },
            "hologram" => Self {
                base_color: [100, 200, 255],
                metallic: 0.0,
                roughness: 0.0,
                normal_strength: 0.1,
                ao_strength: 0.3,
                emission: 1.0,
                ior: 1.0,
            },
            _ => Self {
                base_color: [80, 85, 90],
                metallic: 0.2,
                roughness: 0.35,
                normal_strength: 0.6,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.5,
            },
        }
    }

    /// Concrete materials
    pub fn concrete(variant: &str) -> Self {
        match variant {
            "fresh" => Self {
                base_color: [180, 175, 170],
                metallic: 0.0,
                roughness: 0.7,
                normal_strength: 0.8,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.5,
            },
            "stained" => Self {
                base_color: [140, 130, 120],
                metallic: 0.0,
                roughness: 0.75,
                normal_strength: 1.0,
                ao_strength: 1.1,
                emission: 0.0,
                ior: 1.5,
            },
            "cracked" => Self {
                base_color: [150, 145, 140],
                metallic: 0.0,
                roughness: 0.8,
                normal_strength: 1.4,
                ao_strength: 1.2,
                emission: 0.0,
                ior: 1.5,
            },
            _ => Self {
                base_color: [160, 155, 150],
                metallic: 0.0,
                roughness: 0.72,
                normal_strength: 0.9,
                ao_strength: 1.0,
                emission: 0.0,
                ior: 1.5,
            },
        }
    }

    /// Apply style modifiers to this material
    pub fn with_style(&self, modifiers: &super::StyleModifiers) -> Self {
        let mut result = *self;

        // Apply roughness offset
        result.roughness = (result.roughness + modifiers.roughness_offset).clamp(0.0, 1.0);

        // Apply damage
        if modifiers.damage_amount > 0.0 {
            result.roughness = (result.roughness + modifiers.damage_amount * 0.3).min(1.0);
            result.normal_strength *= 1.0 + modifiers.damage_amount * 0.5;
            result.ao_strength *= 1.0 + modifiers.damage_amount * 0.2;

            // Desaturate base color for damage
            let avg = (result.base_color[0] as f32 + result.base_color[1] as f32 + result.base_color[2] as f32) / 3.0;
            let blend = modifiers.damage_amount * 0.3;
            result.base_color[0] = lerp_u8(result.base_color[0], avg as u8, blend);
            result.base_color[1] = lerp_u8(result.base_color[1], avg as u8, blend);
            result.base_color[2] = lerp_u8(result.base_color[2], avg as u8, blend);
        }

        // Apply temperature shift
        if modifiers.color_temperature != 0.0 {
            let rgb = crate::texture::apply_temperature(
                result.base_color[0],
                result.base_color[1],
                result.base_color[2],
                modifiers.color_temperature * 0.5,
            );
            result.base_color = [rgb.0, rgb.1, rgb.2];
        }

        result
    }

    /// Get all available material categories
    pub fn categories() -> &'static [&'static str] {
        &["metal", "wood", "stone", "fabric", "leather", "plastic", "organic", "crystal", "tech", "concrete"]
    }
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_lookup() {
        let mat = Material::lookup("metal.polished").unwrap();
        assert!(mat.metallic > 0.9);
        assert!(mat.roughness < 0.2);

        let mat = Material::lookup("wood.weathered").unwrap();
        assert!(mat.metallic < 0.1);
        assert!(mat.roughness > 0.5);
    }

    #[test]
    fn test_material_lookup_invalid() {
        assert!(Material::lookup("invalid.material").is_none());
        assert!(Material::lookup("").is_none());
    }

    #[test]
    fn test_material_with_style() {
        let mat = Material::metal("polished");
        let style = super::super::StyleToken::PostApoc.modifiers();
        let damaged = mat.with_style(&style);

        // Damaged should be rougher
        assert!(damaged.roughness > mat.roughness);
    }
}
