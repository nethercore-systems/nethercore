//! Material Functions (Mode 2/3)

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Bind an MRE texture (Metallic-Roughness-Emissive) to slot 1.
    pub fn material_mre(texture: u32);

    /// Bind an albedo texture to slot 0.
    pub fn material_albedo(texture: u32);

    /// Bind a normal map texture to slot 3.
    ///
    /// # Arguments
    /// * `texture` — Handle to a BC5 or RGBA normal map texture
    ///
    /// Normal maps perturb surface normals for detailed lighting without extra geometry.
    /// Requires mesh with tangent data (FORMAT_TANGENT) and UVs.
    /// Works in all lit modes (0=Lambert, 2=PBR, 3=Hybrid) and Mode 1 (Matcap).
    pub fn material_normal(texture: u32);

    /// Skip normal map sampling (use vertex normal instead).
    ///
    /// # Arguments
    /// * `skip` — 1 to skip normal map, 0 to use normal map (default)
    ///
    /// When a mesh has tangent data, normal mapping is enabled by default.
    /// Use this flag to opt out temporarily for debugging or artistic control.
    pub fn skip_normal_map(skip: u32);

    /// Set material metallic value (0.0 = dielectric, 1.0 = metal).
    pub fn material_metallic(value: f32);

    /// Set material roughness value (0.0 = smooth, 1.0 = rough).
    pub fn material_roughness(value: f32);

    /// Set material emissive intensity (0.0 = no emission, >1.0 for HDR).
    pub fn material_emissive(value: f32);

    /// Set rim lighting parameters.
    ///
    /// # Arguments
    /// * `intensity` — Rim brightness (0.0-1.0)
    /// * `power` — Falloff sharpness (0.0-32.0, higher = tighter)
    pub fn material_rim(intensity: f32, power: f32);

    /// Enable/disable uniform color override.
    ///
    /// When enabled, uses the last set_color() value for all subsequent draws,
    /// overriding vertex colors and material albedo.
    ///
    /// # Arguments
    /// * `enabled` — 1 to enable, 0 to disable
    pub fn use_uniform_color(enabled: u32);

    /// Enable/disable uniform metallic override.
    ///
    /// When enabled, uses the last material_metallic() value for all subsequent draws,
    /// overriding per-vertex or per-material metallic values.
    ///
    /// # Arguments
    /// * `enabled` — 1 to enable, 0 to disable
    pub fn use_uniform_metallic(enabled: u32);

    /// Enable/disable uniform roughness override.
    ///
    /// When enabled, uses the last material_roughness() value for all subsequent draws,
    /// overriding per-vertex or per-material roughness values.
    ///
    /// # Arguments
    /// * `enabled` — 1 to enable, 0 to disable
    pub fn use_uniform_roughness(enabled: u32);

    /// Enable/disable uniform emissive override.
    ///
    /// When enabled, uses the last material_emissive() value for all subsequent draws,
    /// overriding per-vertex or per-material emissive values.
    ///
    /// # Arguments
    /// * `enabled` — 1 to enable, 0 to disable
    pub fn use_uniform_emissive(enabled: u32);

    /// Set shininess (Mode 3 alias for roughness).
    pub fn material_shininess(value: f32);

    /// Set specular color (Mode 3 only).
    ///
    /// # Arguments
    /// * `color` — Specular color (0xRRGGBBAA, alpha ignored)
    pub fn material_specular(color: u32);
}
