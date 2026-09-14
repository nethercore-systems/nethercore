//! EPU builder API for constructing environment configurations.
//!
//! This module provides the builder pattern for creating EPU configurations
//! with semantic methods for adding bounds and feature layers.

use super::{
    AdvectParams, ApertureParams, AtmosphereParams, BandRadianceParams, CellParams, DecalParams,
    EpuBlend, EpuConfig, EpuLayer, EpuOpcode, FlowParams, GridParams, LobeRadianceParams,
    MassParams, PatchesParams, REGION_ALL, RampParams, ScatterParams, SectorParams,
    SilhouetteParams, SplitParams, SurfaceParams, encode_direction_u16, pack_meta5,
    pack_thresholds,
};

// =============================================================================
// Builder API
// =============================================================================

/// Begin building an EPU configuration.
///
/// Layers are appended in authored order across all 8 instruction slots.
#[inline]
pub fn epu_begin() -> EpuBuilder {
    EpuBuilder::new()
}

/// Finish building and return the packed `EpuConfig`.
#[inline]
pub fn epu_finish(builder: EpuBuilder) -> EpuConfig {
    builder.finish()
}

/// Builder for constructing EPU configurations with semantic methods.
///
/// Layers are appended in authored order across all 8 instruction slots.
/// This matches the runtime shader model directly.
pub struct EpuBuilder {
    cfg: EpuConfig,
    next_slot: usize,
}

impl Default for EpuBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EpuBuilder {
    /// Create a new builder with all layers initialized to NOP.
    #[inline]
    pub fn new() -> Self {
        Self {
            cfg: EpuConfig::default(),
            next_slot: 0,
        }
    }

    /// Finish building and return the packed configuration.
    #[inline]
    pub fn finish(self) -> EpuConfig {
        self.cfg
    }

    /// Push a bounds layer. Silently ignored if no slots remain.
    fn push_bounds(&mut self, layer: EpuLayer) {
        self.push_layer(layer);
    }

    /// Push a feature layer. Silently ignored if no slots remain.
    fn push_feature(&mut self, layer: EpuLayer) {
        self.push_layer(layer);
    }

    #[inline]
    fn push_layer(&mut self, layer: EpuLayer) {
        if self.next_slot >= 8 {
            return;
        }
        self.cfg.layers[self.next_slot] = layer.encode();
        self.next_slot += 1;
    }

    // =========================================================================
    // Bounds Helpers
    // =========================================================================

    /// Set the bounds gradient (RAMP).
    pub fn ramp_bounds(&mut self, p: RampParams) {
        let layer = EpuLayer {
            opcode: EpuOpcode::Ramp,
            region_mask: REGION_ALL,
            blend: EpuBlend::Add,
            meta5: 0,
            color_a: p.sky_color,
            color_b: p.floor_color,
            alpha_a: 15,
            alpha_b: 15,
            intensity: p.softness,
            param_a: p.wall_color[0], // Wall R (for gradient mixing)
            param_b: p.wall_color[1], // Wall G
            param_c: p.wall_color[2], // Wall B
            param_d: pack_thresholds(p.ceil_q, p.floor_q),
            direction: encode_direction_u16(p.up),
        };
        self.push_layer(layer);
    }

    /// Apply a SECTOR bounds modifier.
    pub fn sector_bounds(&mut self, p: SectorParams) {
        self.push_bounds(EpuLayer {
            opcode: EpuOpcode::Sector,
            region_mask: REGION_ALL,
            blend: EpuBlend::Add,
            meta5: pack_meta5(0, p.variant_id),
            color_a: p.sky_color,
            color_b: p.wall_color,
            alpha_a: 15,
            alpha_b: 15,
            intensity: p.strength,
            param_a: p.center_u01,
            param_b: p.width,
            param_c: 0,
            param_d: 0,
            direction: encode_direction_u16(p.up),
        });
    }

    /// Apply static SILHOUETTE bounds. See [`SilhouetteParams`] for current semantics:
    /// historical `drift_speed` is wall depth; `drift_amount_q` is reserved/inactive.
    /// Packs both fields unchanged within their existing byte/nibble widths.
    pub fn silhouette_bounds(&mut self, p: SilhouetteParams) {
        let param_c = ((p.octaves_q & 0x0F) << 4) | (p.drift_amount_q & 0x0F);
        self.push_bounds(EpuLayer {
            opcode: EpuOpcode::Silhouette,
            region_mask: REGION_ALL,
            blend: EpuBlend::Add,
            meta5: pack_meta5(0, p.variant_id),
            color_a: p.silhouette_color,
            color_b: p.background_color,
            alpha_a: p.strength,
            alpha_b: 0,
            intensity: p.edge_softness,
            param_a: p.horizon_bias,
            param_b: p.roughness,
            param_c,
            param_d: p.drift_speed,
            direction: encode_direction_u16(p.up),
        });
    }

    /// Apply a SPLIT bounds source.
    pub fn split_bounds(&mut self, p: SplitParams) {
        self.push_bounds(EpuLayer {
            opcode: EpuOpcode::Split,
            region_mask: REGION_ALL,
            blend: EpuBlend::Add,
            meta5: pack_meta5(0, p.variant_id),
            color_a: p.sky_color,
            color_b: p.wall_color,
            alpha_a: 15,
            alpha_b: 15,
            intensity: 0,
            param_a: p.blend_width,
            param_b: p.wedge_angle,
            param_c: p.count,
            param_d: p.offset,
            direction: encode_direction_u16(p.axis),
        });
    }

    /// Apply a CELL bounds source.
    pub fn cell_bounds(&mut self, p: CellParams) {
        self.push_bounds(EpuLayer {
            opcode: EpuOpcode::Cell,
            region_mask: REGION_ALL,
            blend: EpuBlend::Add,
            meta5: pack_meta5(0, p.variant_id),
            color_a: p.gap_color,
            color_b: p.wall_color,
            alpha_a: p.gap_alpha,
            alpha_b: p.outline_alpha,
            intensity: p.outline_brightness,
            param_a: p.density,
            param_b: p.fill_ratio,
            param_c: p.gap_width,
            param_d: p.seed,
            direction: encode_direction_u16(p.axis),
        });
    }

    /// Apply a PATCHES bounds source.
    pub fn patches_bounds(&mut self, p: PatchesParams) {
        self.push_bounds(EpuLayer {
            opcode: EpuOpcode::Patches,
            region_mask: REGION_ALL,
            blend: EpuBlend::Add,
            meta5: pack_meta5(p.domain_id, p.variant_id),
            color_a: p.sky_color,
            color_b: p.wall_color,
            alpha_a: p.sky_alpha,
            alpha_b: p.wall_alpha,
            intensity: 0,
            param_a: p.scale,
            param_b: p.coverage,
            param_c: p.sharpness,
            param_d: p.seed,
            direction: encode_direction_u16(p.axis),
        });
    }

    /// Apply an APERTURE bounds modifier.
    pub fn aperture_bounds(&mut self, p: ApertureParams) {
        self.push_bounds(EpuLayer {
            opcode: EpuOpcode::Aperture,
            region_mask: REGION_ALL,
            blend: EpuBlend::Add,
            meta5: pack_meta5(0, p.variant_id),
            color_a: p.opening_color,
            color_b: p.frame_color,
            alpha_a: 0,
            alpha_b: 0,
            intensity: p.edge_softness,
            param_a: p.half_width,
            param_b: p.half_height,
            param_c: p.frame_thickness,
            param_d: p.variant_param,
            direction: encode_direction_u16(p.dir),
        });
    }

    // =========================================================================
    // Feature Helpers
    // =========================================================================

    /// Add a decal shape (DECAL).
    pub fn decal(&mut self, p: DecalParams) {
        let param_a = ((p.shape as u8) << 4) | (p.softness_q & 0x0F);
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::Decal,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            meta5: 0,
            color_a: p.color,
            color_b: p.color_b,
            alpha_a: p.alpha,
            alpha_b: 15,
            intensity: p.intensity,
            param_a,
            param_b: p.size,
            param_c: p.glow_softness,
            param_d: p.phase,
            direction: encode_direction_u16(p.dir),
        });
    }

    /// Add scattered points (SCATTER).
    pub fn scatter(&mut self, p: ScatterParams) {
        let param_c = (p.twinkle_q & 0x0F) << 4;
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::Scatter,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            meta5: 0,
            color_a: p.color,
            color_b: [0, 0, 0],
            alpha_a: 15,
            alpha_b: 15,
            intensity: p.intensity,
            param_a: p.density,
            param_b: p.size,
            param_c,
            param_d: p.seed,
            direction: 0,
        });
    }

    /// Add a grid pattern (GRID).
    pub fn grid(&mut self, p: GridParams) {
        let param_c = ((p.pattern as u8) << 4) | (p.scroll_q & 0x0F);
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::Grid,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            meta5: 0,
            color_a: p.color,
            color_b: [0, 0, 0],
            alpha_a: 15,
            alpha_b: 15,
            intensity: p.intensity,
            param_a: p.scale,
            param_b: p.thickness,
            param_c,
            param_d: p.phase,
            direction: 0,
        });
    }

    /// Add animated flow (FLOW).
    pub fn flow(&mut self, p: FlowParams) {
        let param_c = ((p.octaves & 0x0F) << 4) | ((p.pattern as u8) & 0x0F);
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::Flow,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            meta5: 0,
            color_a: p.color,
            color_b: [0, 0, 0],
            alpha_a: 15,
            alpha_b: 15,
            intensity: p.intensity,
            param_a: p.scale,
            param_b: p.turbulence,
            param_c,
            param_d: p.phase,
            direction: encode_direction_u16(p.dir),
        });
    }

    /// Add a directional glow (LOBE_RADIANCE).
    pub fn lobe_radiance(&mut self, p: LobeRadianceParams) {
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::LobeRadiance,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            meta5: 0,
            color_a: p.color,
            color_b: p.edge_color,
            alpha_a: p.alpha,
            alpha_b: 0,
            intensity: p.intensity,
            param_a: p.exponent,
            param_b: p.falloff,
            param_c: p.waveform as u8,
            param_d: p.phase,
            direction: encode_direction_u16(p.dir),
        });
    }

    /// Add a plain horizon band (BAND_RADIANCE, zero modulation depth).
    /// Use [`Self::band_radiance_with_depth`] for cyclic azimuthal modulation.
    pub fn band_radiance(&mut self, p: BandRadianceParams) {
        self.band_radiance_with_depth(p, 0);
    }

    /// Add a band with explicit modulation depth (clamped to 0..15).
    /// Zero is plain; 15 retains the four-wave 40..100% profile. Phase is cyclic.
    /// Depth uses existing Color B alpha; no packed-layout or parameter-struct change.
    pub fn band_radiance_with_depth(&mut self, p: BandRadianceParams, depth: u8) {
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::BandRadiance,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            meta5: 0,
            color_a: p.color,
            color_b: p.edge_color,
            alpha_a: p.alpha,
            alpha_b: depth.min(15),
            intensity: p.intensity,
            param_a: p.width,
            param_b: p.offset,
            param_c: p.softness,
            param_d: p.phase,
            direction: encode_direction_u16(p.axis),
        });
    }

    /// Add atmospheric absorption/scattering (ATMOSPHERE).
    pub fn atmosphere(&mut self, p: AtmosphereParams) {
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::Atmosphere,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            meta5: pack_meta5(0, p.variant_id),
            color_a: p.zenith_color,
            color_b: p.horizon_color,
            alpha_a: p.alpha,
            alpha_b: 0,
            intensity: p.intensity,
            param_a: p.falloff_exponent,
            param_b: p.horizon_y,
            param_c: p.mie_concentration,
            param_d: p.mie_exponent,
            direction: encode_direction_u16(p.sun_dir),
        });
    }

    /// Add broad transport / mass motion (ADVECT).
    pub fn advect(&mut self, p: AdvectParams) {
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::Advect,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            meta5: pack_meta5(p.domain_id, p.variant as u8),
            color_a: p.color,
            color_b: p.color_b,
            alpha_a: p.alpha,
            alpha_b: 0,
            intensity: p.intensity,
            param_a: p.scale,
            param_b: p.coverage,
            param_c: p.breakup,
            param_d: p.phase,
            direction: encode_direction_u16(p.dir),
        });
    }

    /// Add broad material / surface response (SURFACE).
    pub fn surface(&mut self, p: SurfaceParams) {
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::Surface,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            meta5: pack_meta5(0, p.variant as u8),
            color_a: p.color,
            color_b: p.color_b,
            alpha_a: p.alpha,
            alpha_b: 0,
            intensity: p.intensity,
            param_a: p.scale,
            param_b: p.fracture,
            param_c: p.sheen,
            param_d: p.phase,
            direction: encode_direction_u16(p.dir),
        });
    }

    /// Add a broad scene-owning body (MASS).
    pub fn mass(&mut self, p: MassParams) {
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::Mass,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            meta5: pack_meta5(p.domain_id, p.variant as u8),
            color_a: p.color,
            color_b: p.color_b,
            alpha_a: p.alpha,
            alpha_b: 0,
            intensity: p.intensity,
            param_a: p.scale,
            param_b: p.coverage,
            param_c: p.breakup,
            param_d: p.phase,
            direction: encode_direction_u16(p.dir),
        });
    }
}

#[cfg(test)]
mod mass_tests {
    use super::super::{
        EpuBlend, EpuLayer, EpuOpcode, EpuRegion, MassParams, MassVariant, epu_begin, epu_finish,
    };
    use glam::Vec3;

    #[test]
    fn mass_builder_packs_nondefault_and_default() {
        let mut builder = epu_begin();
        builder.mass(MassParams {
            region: EpuRegion::Walls,
            blend: EpuBlend::Overlay,
            dir: Vec3::X,
            color: [0x12, 0x34, 0x56],
            color_b: [0xA1, 0xB2, 0xC3],
            intensity: 0xD4,
            scale: 0xE5,
            coverage: 0xF6,
            breakup: 0x17,
            phase: 0x28,
            alpha: 0x09,
            domain_id: 2,
            variant: MassVariant::Plume,
        });
        let config = epu_finish(builder);
        let expected = EpuLayer {
            opcode: EpuOpcode::Mass,
            region_mask: EpuRegion::Walls.to_mask(),
            blend: EpuBlend::Overlay,
            meta5: 18,
            color_a: [0x12, 0x34, 0x56],
            color_b: [0xA1, 0xB2, 0xC3],
            alpha_a: 0x09,
            alpha_b: 0,
            intensity: 0xD4,
            param_a: 0xE5,
            param_b: 0xF6,
            param_c: 0x17,
            param_d: 0x28,
            direction: 0x80FF,
        };
        assert_eq!(config.layers[0], expected.encode());
        assert_eq!(config.layers[0], [0xBAF2123456A1B2C3, 0xD4E5F6172880FF90]);
        assert_eq!((config.layers[0][0] >> 59) & 0x1F, EpuOpcode::Mass as u64);

        let mut default_builder = epu_begin();
        default_builder.mass(MassParams::default());
        let default_config = epu_finish(default_builder);
        assert_eq!(
            (default_config.layers[0][0] >> 59) & 0x1F,
            EpuOpcode::Mass as u64
        );
    }
}
