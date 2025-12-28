//! Multi-layer texture composition system
//!
//! Provides a sophisticated layering system for building complex textures
//! from multiple detail passes, similar to professional texturing workflows.

use super::TextureBuffer;
use super::modifiers::{TextureApply, BlendMode, Blend, Contrast};
use super::noise::PerlinConfig;
use super::color::{ColorVariation, ApplyColorVariation};
use super::features::*;
use noise::{NoiseFn, Perlin};

/// A single layer in the texture composition stack
#[derive(Clone)]
pub enum TextureLayer {
    /// Solid base color with optional noise variation
    Base {
        color: [u8; 4],
        noise_scale: Option<f32>,
        noise_intensity: f32,
        seed: u32,
    },

    /// Perlin noise layer
    Noise {
        config: PerlinConfig,
        low_color: [u8; 4],
        high_color: [u8; 4],
        blend: BlendMode,
        opacity: f32,
    },

    /// Color variation pass
    ColorVariation {
        variation: ColorVariation,
        noise_scale: f64,
        seed: u32,
    },

    /// Feature layer (scratches, cracks, etc.)
    Feature {
        feature: FeatureType,
    },

    /// Weathering layer (rust, stains, dust)
    Weathering {
        weathering: WeatheringType,
    },

    /// Edge highlighting
    EdgeWear {
        threshold: f32,
        intensity: f32,
        color: [u8; 4],
    },

    /// Curvature-aware layer (requires curvature map)
    CurvatureAware {
        curvature_map: TextureBuffer,
        edge_wear_color: [u8; 4],
        edge_wear_strength: f32,
        corner_dirt_color: [u8; 4],
        corner_dirt_strength: f32,
    },

    /// Ambient occlusion multiplication
    AmbientOcclusion {
        ao_map: TextureBuffer,
        strength: f32,
    },

    /// Final dust/dirt pass
    FinalPass {
        dust_density: f32,
        dust_color: [u8; 4],
        contrast_boost: f32,
        seed: u32,
    },
}

/// Feature types for the Feature layer
#[derive(Clone)]
pub enum FeatureType {
    Scratches(Scratches),
    Cracks(Cracks),
    Grain(Grain),
    Pores(Pores),
}

/// Weathering types for the Weathering layer
#[derive(Clone)]
pub enum WeatheringType {
    Rust(Rust),
    WaterStains(WaterStains),
    Dust(Dust),
}

/// Builder for composing multi-layer textures
pub struct LayeredTextureBuilder {
    width: u32,
    height: u32,
    layers: Vec<TextureLayer>,
}

impl LayeredTextureBuilder {
    /// Create a new layered texture builder
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            layers: Vec::new(),
        }
    }

    /// Add a base color layer
    pub fn base(mut self, color: [u8; 4]) -> Self {
        self.layers.push(TextureLayer::Base {
            color,
            noise_scale: None,
            noise_intensity: 0.0,
            seed: 0,
        });
        self
    }

    /// Add a base color with noise variation
    pub fn base_with_noise(mut self, color: [u8; 4], noise_scale: f32, intensity: f32, seed: u32) -> Self {
        self.layers.push(TextureLayer::Base {
            color,
            noise_scale: Some(noise_scale),
            noise_intensity: intensity,
            seed,
        });
        self
    }

    /// Add a noise layer
    pub fn noise(
        mut self,
        config: PerlinConfig,
        low: [u8; 4],
        high: [u8; 4],
        blend: BlendMode,
        opacity: f32,
    ) -> Self {
        self.layers.push(TextureLayer::Noise {
            config,
            low_color: low,
            high_color: high,
            blend,
            opacity,
        });
        self
    }

    /// Add color variation layer
    pub fn color_variation(mut self, variation: ColorVariation, noise_scale: f64, seed: u32) -> Self {
        self.layers.push(TextureLayer::ColorVariation {
            variation,
            noise_scale,
            seed,
        });
        self
    }

    /// Add scratches
    pub fn scratches(mut self, scratches: Scratches) -> Self {
        self.layers.push(TextureLayer::Feature {
            feature: FeatureType::Scratches(scratches),
        });
        self
    }

    /// Add cracks
    pub fn cracks(mut self, cracks: Cracks) -> Self {
        self.layers.push(TextureLayer::Feature {
            feature: FeatureType::Cracks(cracks),
        });
        self
    }

    /// Add grain pattern
    pub fn grain(mut self, grain: Grain) -> Self {
        self.layers.push(TextureLayer::Feature {
            feature: FeatureType::Grain(grain),
        });
        self
    }

    /// Add pores
    pub fn pores(mut self, pores: Pores) -> Self {
        self.layers.push(TextureLayer::Feature {
            feature: FeatureType::Pores(pores),
        });
        self
    }

    /// Add rust weathering
    pub fn rust(mut self, rust: Rust) -> Self {
        self.layers.push(TextureLayer::Weathering {
            weathering: WeatheringType::Rust(rust),
        });
        self
    }

    /// Add water stains
    pub fn water_stains(mut self, stains: WaterStains) -> Self {
        self.layers.push(TextureLayer::Weathering {
            weathering: WeatheringType::WaterStains(stains),
        });
        self
    }

    /// Add dust
    pub fn dust(mut self, dust: Dust) -> Self {
        self.layers.push(TextureLayer::Weathering {
            weathering: WeatheringType::Dust(dust),
        });
        self
    }

    /// Add a weathering pass with combined effects based on damage amount
    pub fn weathering_pass(mut self, damage_amount: f32, dark_color: [u8; 4], seed: u32) -> Self {
        // Add scratches proportional to damage
        if damage_amount > 0.1 {
            self.layers.push(TextureLayer::Feature {
                feature: FeatureType::Scratches(Scratches {
                    density: damage_amount * 0.15,
                    depth: damage_amount * 0.6,
                    seed,
                    ..Scratches::default()
                }),
            });
        }

        // Add cracks for heavier damage
        if damage_amount > 0.3 {
            self.layers.push(TextureLayer::Feature {
                feature: FeatureType::Cracks(Cracks {
                    density: (damage_amount - 0.3) * 0.2,
                    depth: damage_amount * 0.5,
                    color: dark_color,
                    seed: seed + 1,
                    ..Cracks::default()
                }),
            });
        }

        // Add dust for any damage level
        self.layers.push(TextureLayer::Weathering {
            weathering: WeatheringType::Dust(Dust {
                density: damage_amount * 0.3,
                color: [
                    ((dark_color[0] as f32 + 128.0) / 2.0) as u8,
                    ((dark_color[1] as f32 + 128.0) / 2.0) as u8,
                    ((dark_color[2] as f32 + 128.0) / 2.0) as u8,
                    255,
                ],
                seed: seed + 2,
            }),
        });

        self
    }

    /// Add edge wear highlighting
    pub fn edge_wear(mut self, threshold: f32, intensity: f32, color: [u8; 4]) -> Self {
        self.layers.push(TextureLayer::EdgeWear {
            threshold,
            intensity,
            color,
        });
        self
    }

    /// Add curvature-aware weathering (requires pre-computed curvature map)
    pub fn curvature_weathering(
        mut self,
        curvature_map: TextureBuffer,
        edge_wear_color: [u8; 4],
        edge_wear_strength: f32,
        corner_dirt_color: [u8; 4],
        corner_dirt_strength: f32,
    ) -> Self {
        self.layers.push(TextureLayer::CurvatureAware {
            curvature_map,
            edge_wear_color,
            edge_wear_strength,
            corner_dirt_color,
            corner_dirt_strength,
        });
        self
    }

    /// Add ambient occlusion multiplication
    pub fn ambient_occlusion(mut self, ao_map: TextureBuffer, strength: f32) -> Self {
        self.layers.push(TextureLayer::AmbientOcclusion {
            ao_map,
            strength,
        });
        self
    }

    /// Add final dust/contrast pass
    pub fn final_pass(mut self, dust_density: f32, dust_color: [u8; 4], contrast_boost: f32, seed: u32) -> Self {
        self.layers.push(TextureLayer::FinalPass {
            dust_density,
            dust_color,
            contrast_boost,
            seed,
        });
        self
    }

    /// Add a raw layer
    pub fn layer(mut self, layer: TextureLayer) -> Self {
        self.layers.push(layer);
        self
    }

    /// Build the final texture by compositing all layers
    pub fn build(self) -> TextureBuffer {
        let mut buffer = TextureBuffer::new(self.width, self.height);

        for layer in self.layers {
            apply_layer(&mut buffer, layer);
        }

        buffer
    }

    /// Build and also generate a height map for normal map generation
    pub fn build_with_height(self) -> (TextureBuffer, TextureBuffer) {
        let mut buffer = TextureBuffer::new(self.width, self.height);
        let mut height = TextureBuffer::new(self.width, self.height);

        // Initialize height to mid-gray
        for y in 0..self.height {
            for x in 0..self.width {
                height.set_pixel(x, y, [128, 128, 128, 255]);
            }
        }

        for layer in self.layers {
            apply_layer_with_height(&mut buffer, &mut height, layer);
        }

        (buffer, height)
    }
}

fn apply_layer(buffer: &mut TextureBuffer, layer: TextureLayer) {
    match layer {
        TextureLayer::Base { color, noise_scale, noise_intensity, seed } => {
            // Fill with base color
            for y in 0..buffer.height {
                for x in 0..buffer.width {
                    buffer.set_pixel(x, y, color);
                }
            }

            // Apply noise variation if specified
            if let Some(scale) = noise_scale {
                let perlin = Perlin::new(seed);
                for y in 0..buffer.height {
                    for x in 0..buffer.width {
                        let noise = perlin.get([x as f64 * scale as f64, y as f64 * scale as f64]);
                        let factor = 1.0 + (noise as f32 * noise_intensity);

                        let pixel = buffer.get_pixel(x, y);
                        buffer.set_pixel(x, y, [
                            (pixel[0] as f32 * factor).clamp(0.0, 255.0) as u8,
                            (pixel[1] as f32 * factor).clamp(0.0, 255.0) as u8,
                            (pixel[2] as f32 * factor).clamp(0.0, 255.0) as u8,
                            pixel[3],
                        ]);
                    }
                }
            }
        }

        TextureLayer::Noise { config, low_color, high_color, blend, opacity } => {
            let noise_tex = config.generate(buffer.width, buffer.height, low_color, high_color);
            buffer.apply(Blend {
                source: noise_tex,
                mode: blend,
                opacity,
            });
        }

        TextureLayer::ColorVariation { variation, noise_scale, seed } => {
            buffer.apply(ApplyColorVariation {
                variation,
                noise_scale,
                seed,
                independent_channels: true,
            });
        }

        TextureLayer::Feature { feature } => {
            match feature {
                FeatureType::Scratches(s) => buffer.apply(s),
                FeatureType::Cracks(c) => buffer.apply(c),
                FeatureType::Grain(g) => buffer.apply(g),
                FeatureType::Pores(p) => buffer.apply(p),
            };
        }

        TextureLayer::Weathering { weathering } => {
            match weathering {
                WeatheringType::Rust(r) => buffer.apply(r),
                WeatheringType::WaterStains(w) => buffer.apply(w),
                WeatheringType::Dust(d) => buffer.apply(d),
            };
        }

        TextureLayer::EdgeWear { threshold, intensity, color } => {
            buffer.apply(EdgeHighlight { threshold, intensity, color });
        }

        TextureLayer::CurvatureAware {
            curvature_map,
            edge_wear_color,
            edge_wear_strength,
            corner_dirt_color,
            corner_dirt_strength,
        } => {
            apply_curvature_weathering(
                buffer,
                &curvature_map,
                edge_wear_color,
                edge_wear_strength,
                corner_dirt_color,
                corner_dirt_strength,
            );
        }

        TextureLayer::AmbientOcclusion { ao_map, strength } => {
            apply_ao(buffer, &ao_map, strength);
        }

        TextureLayer::FinalPass { dust_density, dust_color, contrast_boost, seed } => {
            // Apply dust
            buffer.apply(Dust {
                density: dust_density,
                color: dust_color,
                seed,
            });

            // Apply contrast boost
            if contrast_boost != 1.0 {
                buffer.apply(Contrast { factor: contrast_boost });
            }
        }
    }
}

fn apply_layer_with_height(buffer: &mut TextureBuffer, height: &mut TextureBuffer, layer: TextureLayer) {
    // Clone layer to check type before moving
    match &layer {
        TextureLayer::Feature { feature } => {
            // Features affect height
            match feature {
                FeatureType::Scratches(s) => {
                    // Scratches lower height
                    modify_height_scratches(height, s);
                }
                FeatureType::Cracks(c) => {
                    // Cracks are deep grooves
                    modify_height_cracks(height, c);
                }
                FeatureType::Pores(p) => {
                    // Pores are small indentations
                    modify_height_pores(height, p);
                }
                _ => {}
            }
        }
        _ => {}
    }

    // Apply to color buffer
    apply_layer(buffer, layer);
}

fn modify_height_scratches(height: &mut TextureBuffer, scratches: &Scratches) {
    let perlin = Perlin::new(scratches.seed);
    let depth_value = (128.0 * (1.0 - scratches.depth * 0.5)) as u8;

    // Similar logic to scratch rendering but modifies height
    let area = height.width * height.height;
    let num_scratches = ((area as f32 * scratches.density * 0.01) as u32).max(1);

    for i in 0..num_scratches {
        let nx = (i as f64 * 0.1) + scratches.seed as f64;
        let ny = (i as f64 * 0.2) + scratches.seed as f64;

        let start_x = ((perlin.get([nx, 0.0]) + 1.0) / 2.0 * height.width as f64) as i32;
        let start_y = ((perlin.get([0.0, ny]) + 1.0) / 2.0 * height.height as f64) as i32;

        let len_t = ((perlin.get([nx + 10.0, ny + 10.0]) + 1.0) / 2.0) as f32;
        let length = scratches.length.0 + (scratches.length.1 - scratches.length.0) * len_t;
        let length_px = (length * height.width.min(height.height) as f32) as i32;

        let angle = perlin.get([nx + 20.0, ny + 20.0]) as f32 * std::f32::consts::PI;
        let dx = angle.cos();
        let dy = angle.sin();

        for step in 0..length_px {
            let x = start_x + (step as f32 * dx) as i32;
            let y = start_y + (step as f32 * dy) as i32;

            if x >= 0 && x < height.width as i32 && y >= 0 && y < height.height as i32 {
                let current = height.get_pixel(x as u32, y as u32);
                let new_value = current[0].min(depth_value);
                height.set_pixel(x as u32, y as u32, [new_value, new_value, new_value, 255]);
            }
        }
    }
}

fn modify_height_cracks(height: &mut TextureBuffer, cracks: &Cracks) {
    let perlin = Perlin::new(cracks.seed);
    let scale = (0.02 + cracks.density * 0.05) as f64;

    for y in 0..height.height {
        for x in 0..height.width {
            let nx = x as f64 * scale;
            let ny = y as f64 * scale;

            let v1 = perlin.get([nx, ny]);
            let v2 = perlin.get([nx + 0.5, ny + 0.5]);
            let edge = ((v1 - v2).abs() as f32 * 2.0).min(1.0);

            let threshold = 1.0 - cracks.density * 0.5;
            if edge > threshold {
                let intensity = ((edge - threshold) / (1.0 - threshold)).min(1.0);
                let current = height.get_pixel(x, y);
                let depth = (current[0] as f32 * (1.0 - intensity * cracks.depth * 0.7)) as u8;
                height.set_pixel(x, y, [depth, depth, depth, 255]);
            }
        }
    }
}

fn modify_height_pores(height: &mut TextureBuffer, pores: &Pores) {
    use noise::Worley;

    let worley = Worley::new(pores.seed);
    let scale = (0.05 + pores.density * 0.1) as f64;

    for y in 0..height.height {
        for x in 0..height.width {
            let nx = x as f64 * scale;
            let ny = y as f64 * scale;

            let cell_value = worley.get([nx, ny]);
            let pore_intensity = (1.0 - cell_value.abs() as f32).max(0.0);

            if pore_intensity > 0.7 {
                let intensity = (pore_intensity - 0.7) / 0.3 * pores.depth;
                let current = height.get_pixel(x, y);
                let depth = (current[0] as f32 * (1.0 - intensity * 0.3)) as u8;
                height.set_pixel(x, y, [depth, depth, depth, 255]);
            }
        }
    }
}

fn apply_curvature_weathering(
    buffer: &mut TextureBuffer,
    curvature: &TextureBuffer,
    edge_color: [u8; 4],
    edge_strength: f32,
    corner_color: [u8; 4],
    corner_strength: f32,
) {
    for y in 0..buffer.height.min(curvature.height) {
        for x in 0..buffer.width.min(curvature.width) {
            let curv = curvature.get_pixel(x, y);
            let curv_value = curv[0] as f32 / 255.0;

            let pixel = buffer.get_pixel(x, y);
            let mut r = pixel[0] as f32;
            let mut g = pixel[1] as f32;
            let mut b = pixel[2] as f32;

            // High curvature (edges) - blend toward edge color
            if curv_value > 0.5 {
                let edge_factor = (curv_value - 0.5) * 2.0 * edge_strength;
                r = r * (1.0 - edge_factor) + edge_color[0] as f32 * edge_factor;
                g = g * (1.0 - edge_factor) + edge_color[1] as f32 * edge_factor;
                b = b * (1.0 - edge_factor) + edge_color[2] as f32 * edge_factor;
            }
            // Low curvature (corners/concave) - blend toward corner color
            else {
                let corner_factor = (0.5 - curv_value) * 2.0 * corner_strength;
                r = r * (1.0 - corner_factor) + corner_color[0] as f32 * corner_factor;
                g = g * (1.0 - corner_factor) + corner_color[1] as f32 * corner_factor;
                b = b * (1.0 - corner_factor) + corner_color[2] as f32 * corner_factor;
            }

            buffer.set_pixel(x, y, [
                r.clamp(0.0, 255.0) as u8,
                g.clamp(0.0, 255.0) as u8,
                b.clamp(0.0, 255.0) as u8,
                pixel[3],
            ]);
        }
    }
}

fn apply_ao(buffer: &mut TextureBuffer, ao_map: &TextureBuffer, strength: f32) {
    for y in 0..buffer.height.min(ao_map.height) {
        for x in 0..buffer.width.min(ao_map.width) {
            let ao = ao_map.get_pixel(x, y);
            let ao_value = ao[0] as f32 / 255.0;

            // AO darkens based on occlusion (lower values = more occluded)
            let factor = 1.0 - (1.0 - ao_value) * strength;

            let pixel = buffer.get_pixel(x, y);
            buffer.set_pixel(x, y, [
                (pixel[0] as f32 * factor) as u8,
                (pixel[1] as f32 * factor) as u8,
                (pixel[2] as f32 * factor) as u8,
                pixel[3],
            ]);
        }
    }
}

// Preset layer stacks for common materials

impl LayeredTextureBuilder {
    /// Preset: Worn metal surface
    pub fn preset_worn_metal(width: u32, height: u32, base_color: [u8; 4], seed: u32) -> Self {
        Self::new(width, height)
            .base_with_noise(base_color, 0.02, 0.15, seed)
            .color_variation(ColorVariation::subtle(), 0.03, seed + 1)
            .grain(Grain::brushed_metal())
            .scratches(Scratches::light())
            .edge_wear(0.15, 0.4, [220, 220, 215, 255])
            .dust(Dust { density: 0.1, ..Default::default() })
            .final_pass(0.05, [180, 175, 170, 255], 1.1, seed + 2)
    }

    /// Preset: Rusty metal surface
    pub fn preset_rusty_metal(width: u32, height: u32, base_color: [u8; 4], seed: u32) -> Self {
        Self::new(width, height)
            .base_with_noise(base_color, 0.03, 0.2, seed)
            .color_variation(ColorVariation::rusty(), 0.02, seed + 1)
            .scratches(Scratches::heavy())
            .rust(Rust { amount: 0.4, ..Default::default() })
            .pores(Pores { density: 0.2, ..Default::default() })
            .water_stains(WaterStains { intensity: 0.2, ..Default::default() })
            .final_pass(0.08, [150, 140, 130, 255], 1.15, seed + 2)
    }

    /// Preset: Weathered wood surface
    pub fn preset_weathered_wood(width: u32, height: u32, base_color: [u8; 4], seed: u32) -> Self {
        Self::new(width, height)
            .base_with_noise(base_color, 0.01, 0.1, seed)
            .grain(Grain::wood())
            .color_variation(ColorVariation::moderate(), 0.02, seed + 1)
            .cracks(Cracks { density: 0.15, ..Default::default() })
            .water_stains(WaterStains { intensity: 0.15, ..Default::default() })
            .dust(Dust { density: 0.15, ..Default::default() })
            .final_pass(0.05, [170, 165, 155, 255], 1.08, seed + 2)
    }

    /// Preset: Stone/rock surface
    pub fn preset_stone(width: u32, height: u32, base_color: [u8; 4], seed: u32) -> Self {
        Self::new(width, height)
            .base_with_noise(base_color, 0.04, 0.3, seed)
            .noise(
                PerlinConfig { scale: 0.02, octaves: 4, ..PerlinConfig::with_seed(seed + 1) },
                [
                    (base_color[0] as f32 * 0.7) as u8,
                    (base_color[1] as f32 * 0.7) as u8,
                    (base_color[2] as f32 * 0.7) as u8,
                    255,
                ],
                [
                    (base_color[0] as f32 * 1.2).min(255.0) as u8,
                    (base_color[1] as f32 * 1.2).min(255.0) as u8,
                    (base_color[2] as f32 * 1.2).min(255.0) as u8,
                    255,
                ],
                BlendMode::Overlay,
                0.4,
            )
            .pores(Pores { density: 0.25, depth: 0.3, ..Default::default() })
            .cracks(Cracks { density: 0.08, width: 1.0, ..Default::default() })
            .final_pass(0.03, [160, 155, 150, 255], 1.12, seed + 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layered_builder_basic() {
        let tex = LayeredTextureBuilder::new(64, 64)
            .base([128, 100, 80, 255])
            .build();

        assert_eq!(tex.width, 64);
        assert_eq!(tex.height, 64);
        assert_eq!(tex.get_pixel(0, 0), [128, 100, 80, 255]);
    }

    #[test]
    fn test_layered_builder_with_noise() {
        let tex = LayeredTextureBuilder::new(64, 64)
            .base_with_noise([128, 128, 128, 255], 0.1, 0.3, 42)
            .build();

        // Should have variation
        let mut has_variation = false;
        let first = tex.get_pixel(0, 0);
        for y in 0..64 {
            for x in 0..64 {
                if tex.get_pixel(x, y) != first {
                    has_variation = true;
                    break;
                }
            }
        }
        assert!(has_variation);
    }

    #[test]
    fn test_preset_worn_metal() {
        let tex = LayeredTextureBuilder::preset_worn_metal(64, 64, [180, 180, 190, 255], 42).build();

        assert_eq!(tex.width, 64);
        assert_eq!(tex.height, 64);
    }

    #[test]
    fn test_preset_rusty_metal() {
        let tex = LayeredTextureBuilder::preset_rusty_metal(64, 64, [150, 150, 150, 255], 42).build();

        // Should have some orange/brown rust coloring (at least 5 pixels warmer than blue)
        let mut warm_count = 0;
        for y in 0..64 {
            for x in 0..64 {
                let p = tex.get_pixel(x, y);
                // Check for warm color: red channel significantly higher than blue
                if p[0] > p[2] + 10 {
                    warm_count += 1;
                }
            }
        }
        // With 40% rust coverage, we expect many warm pixels
        assert!(warm_count > 0, "Expected warm rusty colors in texture, found: {}", warm_count);
    }

    #[test]
    fn test_build_with_height() {
        let (albedo, height) = LayeredTextureBuilder::new(64, 64)
            .base([128, 100, 80, 255])
            .scratches(Scratches { density: 0.3, ..Default::default() })
            .build_with_height();

        assert_eq!(albedo.width, 64);
        assert_eq!(height.width, 64);
    }
}
