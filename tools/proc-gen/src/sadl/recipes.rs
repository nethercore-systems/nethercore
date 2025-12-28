//! Generation recipes combining style, palette, and material
//!
//! Recipes are pre-configured bundles of generation parameters
//! for creating consistent, high-quality assets.

use super::{StyleToken, StyleModifiers, ColorPalette, ColorSet, Material, SimpleRng};
use crate::texture::{TextureBuffer, LayeredTextureBuilder, Scratches};
use crate::mesh::{UnpackedMesh, MeshApply, NoiseDisplace, BakeVertexAO, BakeVertexCurvature, BakeDirectionalLight};
use nethercore_zx::procedural::{generate_cube_uv, generate_sphere_uv, generate_cylinder_uv};

/// A generation recipe combining all SADL parameters
#[derive(Clone)]
pub struct GenerationRecipe {
    /// Recipe name
    pub name: String,
    /// Description
    pub description: String,
    /// Base style token
    pub style: StyleToken,
    /// Color palette
    pub palette: ColorPalette,
    /// Material descriptor
    pub material: String,
    /// Shape hints (e.g., "barrel", "crate", "wall")
    pub shape_hints: Vec<String>,
    /// Constraints
    pub constraints: RecipeConstraints,
}

/// Constraints for asset generation
#[derive(Clone, Copy, Debug)]
pub struct RecipeConstraints {
    /// Minimum scale
    pub scale_min: f32,
    /// Maximum scale
    pub scale_max: f32,
    /// Minimum noise amplitude
    pub noise_amplitude_min: f32,
    /// Maximum noise amplitude
    pub noise_amplitude_max: f32,
    /// Minimum polygon budget
    pub poly_budget_min: u32,
    /// Maximum polygon budget
    pub poly_budget_max: u32,
    /// Texture resolution
    pub texture_resolution: u32,
    /// Target texel density (texels per world unit)
    pub texel_density: f32,
}

impl Default for RecipeConstraints {
    fn default() -> Self {
        Self {
            scale_min: 0.5,
            scale_max: 2.0,
            noise_amplitude_min: 0.01,
            noise_amplitude_max: 0.05,
            poly_budget_min: 100,
            poly_budget_max: 500,
            texture_resolution: 256,
            texel_density: 256.0,
        }
    }
}

impl RecipeConstraints {
    /// Constraints for hero/important assets (higher budget)
    pub fn hero() -> Self {
        Self {
            scale_min: 1.0,
            scale_max: 4.0,
            noise_amplitude_min: 0.02,
            noise_amplitude_max: 0.08,
            poly_budget_min: 500,
            poly_budget_max: 2000,
            texture_resolution: 512,
            texel_density: 512.0,
        }
    }

    /// Constraints for background/distant assets (lower budget)
    pub fn background() -> Self {
        Self {
            scale_min: 0.5,
            scale_max: 1.5,
            noise_amplitude_min: 0.005,
            noise_amplitude_max: 0.02,
            poly_budget_min: 50,
            poly_budget_max: 200,
            texture_resolution: 128,
            texel_density: 128.0,
        }
    }

    /// Constraints based on expected game size
    pub fn for_game_size(game_size: GameSize) -> Self {
        match game_size {
            GameSize::Tiny => Self {
                poly_budget_min: 200,
                poly_budget_max: 1000,
                texture_resolution: 128,
                ..Default::default()
            },
            GameSize::Small => Self {
                poly_budget_min: 300,
                poly_budget_max: 1500,
                texture_resolution: 256,
                ..Default::default()
            },
            GameSize::Medium => Self {
                poly_budget_min: 400,
                poly_budget_max: 2000,
                texture_resolution: 256,
                ..Default::default()
            },
            GameSize::Large => Self {
                poly_budget_min: 500,
                poly_budget_max: 3000,
                texture_resolution: 512,
                texel_density: 512.0,
                ..Default::default()
            },
            GameSize::Massive => Self {
                poly_budget_min: 100,
                poly_budget_max: 500,
                texture_resolution: 128,
                texel_density: 128.0,
                ..Default::default()
            },
        }
    }
}

/// Expected game size for dynamic budgeting
#[derive(Clone, Copy, Debug, Default)]
pub enum GameSize {
    /// Very small game, few assets (can use high budgets)
    Tiny,
    /// Small game with moderate assets
    Small,
    /// Medium sized game
    #[default]
    Medium,
    /// Large game with many assets
    Large,
    /// Massive game requiring aggressive budgets
    Massive,
}

impl GenerationRecipe {
    /// Create a recipe from a semantic description
    pub fn from_description(description: &str, constraints: RecipeConstraints) -> Self {
        let lower = description.to_lowercase();

        // Extract style token
        let style = extract_style_token(&lower);

        // Extract palette
        let palette = extract_palette(&lower, &style);

        // Extract material
        let material = extract_material(&lower);

        // Extract shape hints
        let shape_hints = extract_shape_hints(&lower);

        Self {
            name: description.to_string(),
            description: description.to_string(),
            style,
            palette,
            material,
            shape_hints,
            constraints,
        }
    }

    /// Get style modifiers
    pub fn style_modifiers(&self) -> StyleModifiers {
        self.style.modifiers()
    }

    /// Get material with style applied
    pub fn styled_material(&self) -> Material {
        Material::lookup(&self.material)
            .unwrap_or_default()
            .with_style(&self.style_modifiers())
    }

    /// Sample colors from the palette with style temperature applied
    pub fn sample_colors(&self, seed: u32) -> ColorSet {
        let mut rng = SimpleRng::new(seed);
        self.palette.sample_full(&mut rng)
            .with_temperature(self.style_modifiers().color_temperature)
    }

    /// Generate a texture using this recipe
    pub fn generate_texture(&self, width: u32, height: u32, seed: u32) -> TextureBuffer {
        let colors = self.sample_colors(seed);
        let modifiers = self.style_modifiers();
        let material = self.styled_material();

        // Use layered builder with recipe parameters
        let mut builder = LayeredTextureBuilder::new(width, height)
            .base_with_noise(
                colors.primary,
                0.02 * modifiers.pattern_scale,
                0.1 + modifiers.damage_amount * 0.1,
                seed,
            );

        // Add weathering based on damage
        if modifiers.damage_amount > 0.1 {
            builder = builder.weathering_pass(modifiers.damage_amount, colors.dark, seed + 100);
        }

        // Add detail based on detail level
        match modifiers.detail_level {
            super::DetailLevel::High | super::DetailLevel::Extreme => {
                builder = builder.scratches(Scratches::light());
            }
            _ => {}
        }

        // Final pass with contrast based on material roughness
        builder = builder.final_pass(
            0.02,
            colors.secondary,
            1.0 + (1.0 - material.roughness) * 0.2,
            seed + 200,
        );

        builder.build()
    }

    /// Generate a mesh using this recipe
    pub fn generate_mesh(&self, seed: u32) -> UnpackedMesh {
        let mut rng = SimpleRng::new(seed);
        let modifiers = self.style_modifiers();

        // Determine base shape from hints
        let mut mesh: UnpackedMesh = match self.primary_shape() {
            PrimaryShape::Cube => generate_cube_uv(1.0, 1.0, 1.0),
            PrimaryShape::Sphere => generate_sphere_uv(0.5, 16, 8),
            PrimaryShape::Cylinder => generate_cylinder_uv(0.5, 0.5, 1.0, 12),
        };

        // Apply noise displacement based on style
        let amplitude = lerp(
            self.constraints.noise_amplitude_min,
            self.constraints.noise_amplitude_max,
            modifiers.damage_amount + rng.next_f32() * 0.3,
        );

        if amplitude > 0.001 {
            mesh.apply(NoiseDisplace {
                amplitude,
                scale: 1.0 / modifiers.pattern_scale,
                octaves: modifiers.effective_noise_octaves(2),
                persistence: 0.5,
                seed,
                recalculate_normals: true,
            });
        }

        mesh
    }

    /// Generate a mesh with baked vertex colors
    pub fn generate_mesh_with_colors(&self, seed: u32) -> UnpackedMesh {
        let mut mesh = self.generate_mesh(seed);
        let colors = self.sample_colors(seed);
        let modifiers = self.style_modifiers();

        // Bake curvature for edge wear
        mesh.apply(BakeVertexCurvature::default());

        // Bake AO
        mesh.apply(BakeVertexAO::quick());

        // Apply directional light based on style
        let light_temp = if modifiers.color_temperature > 0.0 {
            [255, 245, 230, 255] // Warm light
        } else {
            [230, 240, 255, 255] // Cool light
        };

        mesh.apply(BakeDirectionalLight {
            direction: [0.5, 1.0, 0.3],
            light_color: light_temp,
            shadow_color: colors.dark,
            ambient: 0.3,
        });

        mesh
    }

    fn primary_shape(&self) -> PrimaryShape {
        for hint in &self.shape_hints {
            match hint.to_lowercase().as_str() {
                "barrel" | "cylinder" | "pipe" | "column" | "pillar" | "pole" | "tube" => {
                    return PrimaryShape::Cylinder;
                }
                "sphere" | "ball" | "orb" | "dome" | "boulder" | "rock" => {
                    return PrimaryShape::Sphere;
                }
                _ => {}
            }
        }
        PrimaryShape::Cube
    }
}

#[derive(Clone, Copy)]
enum PrimaryShape {
    Cube,
    Sphere,
    Cylinder,
}

// Preset recipes

impl GenerationRecipe {
    /// Medieval fantasy prop preset
    pub fn medieval_prop() -> Self {
        Self {
            name: "medieval_prop".to_string(),
            description: "Weathered medieval wooden or metal prop".to_string(),
            style: StyleToken::Medieval,
            palette: ColorPalette::WarmEarthy,
            material: "wood.weathered".to_string(),
            shape_hints: vec!["crate".to_string(), "barrel".to_string(), "chest".to_string()],
            constraints: RecipeConstraints::default(),
        }
    }

    /// Sci-fi panel preset
    pub fn scifi_panel() -> Self {
        Self {
            name: "scifi_panel".to_string(),
            description: "Clean sci-fi wall panel or console".to_string(),
            style: StyleToken::Scifi,
            palette: ColorPalette::CoolMetal,
            material: "metal.brushed".to_string(),
            shape_hints: vec!["panel".to_string(), "console".to_string()],
            constraints: RecipeConstraints {
                noise_amplitude_min: 0.0,
                noise_amplitude_max: 0.01,
                ..Default::default()
            },
        }
    }

    /// Post-apocalyptic debris preset
    pub fn postapoc_debris() -> Self {
        Self {
            name: "postapoc_debris".to_string(),
            description: "Rusted, damaged post-apocalyptic debris".to_string(),
            style: StyleToken::PostApoc,
            palette: ColorPalette::Dusty,
            material: "metal.rusted".to_string(),
            shape_hints: vec!["debris".to_string(), "wreck".to_string()],
            constraints: RecipeConstraints::default(),
        }
    }

    /// Fantasy crystal preset
    pub fn fantasy_crystal() -> Self {
        Self {
            name: "fantasy_crystal".to_string(),
            description: "Magical glowing crystal".to_string(),
            style: StyleToken::Fantasy,
            palette: ColorPalette::Vibrant,
            material: "crystal.magical".to_string(),
            shape_hints: vec!["crystal".to_string(), "gem".to_string()],
            constraints: RecipeConstraints::default(),
        }
    }

    /// Industrial machinery preset
    pub fn industrial_machine() -> Self {
        Self {
            name: "industrial_machine".to_string(),
            description: "Heavy industrial machinery".to_string(),
            style: StyleToken::Industrial,
            palette: ColorPalette::Grayscale,
            material: "metal.iron".to_string(),
            shape_hints: vec!["machine".to_string(), "engine".to_string()],
            constraints: RecipeConstraints::hero(),
        }
    }

    /// Organic growth preset
    pub fn organic_growth() -> Self {
        Self {
            name: "organic_growth".to_string(),
            description: "Natural organic growth (vines, moss, etc)".to_string(),
            style: StyleToken::Organic,
            palette: ColorPalette::ForestGreen,
            material: "organic.moss".to_string(),
            shape_hints: vec!["plant".to_string(), "growth".to_string()],
            constraints: RecipeConstraints::background(),
        }
    }
}

// Helper functions

fn extract_style_token(text: &str) -> StyleToken {
    // Check for explicit style keywords
    let keywords = [
        ("rustic", StyleToken::Rustic),
        ("medieval", StyleToken::Medieval),
        ("ancient", StyleToken::Ancient),
        ("victorian", StyleToken::Victorian),
        ("cyberpunk", StyleToken::Cyberpunk),
        ("sci-fi", StyleToken::Scifi),
        ("scifi", StyleToken::Scifi),
        ("futuristic", StyleToken::Scifi),
        ("industrial", StyleToken::Industrial),
        ("minimalist", StyleToken::Minimalist),
        ("organic", StyleToken::Organic),
        ("overgrown", StyleToken::Overgrown),
        ("crystal", StyleToken::Crystalline),
        ("elemental", StyleToken::Elemental),
        ("fantasy", StyleToken::Fantasy),
        ("gothic", StyleToken::Gothic),
        ("steampunk", StyleToken::Steampunk),
        ("dieselpunk", StyleToken::Dieselpunk),
        ("geometric", StyleToken::Geometric),
        ("abstract", StyleToken::Abstract),
        ("baroque", StyleToken::Baroque),
        ("art deco", StyleToken::ArtDeco),
        ("post-apoc", StyleToken::PostApoc),
        ("postapoc", StyleToken::PostApoc),
        ("apocalyptic", StyleToken::PostApoc),
        ("pristine", StyleToken::Pristine),
        ("new", StyleToken::Pristine),
        ("clean", StyleToken::Pristine),
        ("corrupted", StyleToken::Corrupted),
        ("ethereal", StyleToken::Ethereal),
        ("ghostly", StyleToken::Ethereal),
    ];

    for (keyword, token) in keywords {
        if text.contains(keyword) {
            return token;
        }
    }

    // Infer from adjectives
    if text.contains("weathered") || text.contains("old") || text.contains("worn") {
        return StyleToken::Rustic;
    }
    if text.contains("rusted") || text.contains("damaged") || text.contains("broken") {
        return StyleToken::PostApoc;
    }
    if text.contains("magical") || text.contains("enchanted") || text.contains("glowing") {
        return StyleToken::Fantasy;
    }
    if text.contains("metallic") || text.contains("chrome") || text.contains("sleek") {
        return StyleToken::Scifi;
    }

    StyleToken::default()
}

fn extract_palette(text: &str, style: &StyleToken) -> ColorPalette {
    // Check explicit color keywords
    if text.contains("red") || text.contains("blood") {
        return ColorPalette::BloodRed;
    }
    if text.contains("green") || text.contains("forest") || text.contains("nature") {
        return ColorPalette::ForestGreen;
    }
    if text.contains("blue") || text.contains("ocean") || text.contains("water") {
        return ColorPalette::Ocean;
    }
    if text.contains("gold") || text.contains("golden") {
        return ColorPalette::Gold;
    }
    if text.contains("purple") || text.contains("violet") {
        return ColorPalette::Violet;
    }
    if text.contains("copper") || text.contains("bronze") {
        return ColorPalette::Copper;
    }
    if text.contains("neon") || text.contains("glowing") {
        return ColorPalette::Neon;
    }
    if text.contains("dark") || text.contains("shadow") {
        return ColorPalette::Night;
    }
    if text.contains("ice") || text.contains("frozen") || text.contains("arctic") {
        return ColorPalette::Arctic;
    }

    // Default based on style
    match style {
        StyleToken::Rustic | StyleToken::Medieval => ColorPalette::WarmEarthy,
        StyleToken::Cyberpunk => ColorPalette::Neon,
        StyleToken::Scifi | StyleToken::Industrial => ColorPalette::CoolMetal,
        StyleToken::PostApoc => ColorPalette::Dusty,
        StyleToken::Fantasy | StyleToken::Crystalline => ColorPalette::Vibrant,
        StyleToken::Gothic => ColorPalette::Night,
        StyleToken::Organic | StyleToken::Overgrown => ColorPalette::ForestGreen,
        StyleToken::Pristine | StyleToken::Minimalist => ColorPalette::Grayscale,
        StyleToken::Ethereal => ColorPalette::Pastel,
        _ => ColorPalette::default(),
    }
}

fn extract_material(text: &str) -> String {
    // Direct material mentions
    let materials = [
        ("rusted metal", "metal.rusted"),
        ("rusty", "metal.rusted"),
        ("chrome", "metal.chrome"),
        ("polished metal", "metal.polished"),
        ("brushed metal", "metal.brushed"),
        ("gold", "metal.gold"),
        ("copper", "metal.copper"),
        ("brass", "metal.brass"),
        ("bronze", "metal.bronze"),
        ("iron", "metal.iron"),
        ("steel", "metal.polished"),
        ("weathered wood", "wood.weathered"),
        ("fresh wood", "wood.fresh"),
        ("polished wood", "wood.polished"),
        ("charred", "wood.charred"),
        ("oak", "wood.oak"),
        ("mahogany", "wood.mahogany"),
        ("rough stone", "stone.rough"),
        ("polished stone", "stone.polished"),
        ("marble", "stone.marble"),
        ("sandstone", "stone.sandstone"),
        ("mossy stone", "stone.mossy"),
        ("cracked stone", "stone.cracked"),
        ("leather", "leather.brown"),
        ("worn leather", "leather.worn"),
        ("fabric", "fabric.cotton"),
        ("silk", "fabric.silk"),
        ("velvet", "fabric.velvet"),
        ("plastic", "plastic.matte"),
        ("glossy plastic", "plastic.glossy"),
        ("rubber", "plastic.rubber"),
        ("crystal", "crystal.clear"),
        ("magical crystal", "crystal.magical"),
        ("corrupted", "crystal.corrupted"),
        ("bone", "organic.bone"),
        ("chitin", "organic.chitin"),
        ("bark", "organic.bark"),
        ("moss", "organic.moss"),
        ("concrete", "concrete.stained"),
        ("cracked concrete", "concrete.cracked"),
        ("screen", "tech.screen"),
        ("circuit", "tech.circuit"),
        ("hologram", "tech.hologram"),
    ];

    for (keyword, mat) in materials {
        if text.contains(keyword) {
            return mat.to_string();
        }
    }

    // Infer from context
    if text.contains("barrel") || text.contains("crate") || text.contains("chest") {
        return "wood.weathered".to_string();
    }
    if text.contains("panel") || text.contains("door") && text.contains("metal") {
        return "metal.brushed".to_string();
    }
    if text.contains("wall") || text.contains("floor") {
        return "stone.rough".to_string();
    }
    if text.contains("machine") || text.contains("robot") {
        return "metal.iron".to_string();
    }

    // Default based on other hints
    "metal.brushed".to_string()
}

fn extract_shape_hints(text: &str) -> Vec<String> {
    let shapes = [
        "barrel", "crate", "chest", "box", "cube",
        "sphere", "ball", "orb", "boulder", "rock",
        "cylinder", "pipe", "column", "pillar", "pole",
        "panel", "wall", "floor", "platform",
        "door", "gate", "fence",
        "machine", "engine", "console", "terminal",
        "crystal", "gem", "shard",
        "plant", "tree", "bush", "vine",
        "debris", "wreck", "ruin",
    ];

    let mut hints = Vec::new();
    for shape in shapes {
        if text.contains(shape) {
            hints.push(shape.to_string());
        }
    }

    hints
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_from_description() {
        let recipe = GenerationRecipe::from_description(
            "weathered medieval barrel",
            RecipeConstraints::default(),
        );

        assert!(matches!(recipe.style, StyleToken::Medieval | StyleToken::Rustic));
        assert!(recipe.material.contains("wood"));
        assert!(recipe.shape_hints.contains(&"barrel".to_string()));
    }

    #[test]
    fn test_recipe_generate_texture() {
        let recipe = GenerationRecipe::medieval_prop();
        let texture = recipe.generate_texture(64, 64, 42);

        assert_eq!(texture.width, 64);
        assert_eq!(texture.height, 64);
    }

    #[test]
    fn test_recipe_generate_mesh() {
        let recipe = GenerationRecipe::scifi_panel();
        let mesh = recipe.generate_mesh(42);

        assert!(!mesh.positions.is_empty());
    }

    #[test]
    fn test_recipe_generate_mesh_with_colors() {
        let recipe = GenerationRecipe::postapoc_debris();
        let mesh = recipe.generate_mesh_with_colors(42);

        assert!(!mesh.positions.is_empty());
        assert!(!mesh.colors.is_empty());
        assert_eq!(mesh.colors.len(), mesh.positions.len());
    }

    #[test]
    fn test_constraints_for_game_size() {
        let tiny = RecipeConstraints::for_game_size(GameSize::Tiny);
        let massive = RecipeConstraints::for_game_size(GameSize::Massive);

        // Massive games should have lower budgets
        assert!(massive.poly_budget_max < tiny.poly_budget_max);
    }
}
