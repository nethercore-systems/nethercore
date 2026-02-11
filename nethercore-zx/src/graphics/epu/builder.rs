//! EPU builder API for constructing environment configurations.
//!
//! This module provides the builder pattern for creating EPU configurations
//! with semantic methods for adding bounds and feature layers.

use super::{
    ApertureParams, AtmosphereParams, BandRadianceParams, CellParams, DecalParams, EpuBlend,
    EpuConfig, EpuLayer, EpuOpcode, FlowParams, GridParams, LobeRadianceParams, PatchesParams,
    REGION_ALL, RampParams, ScatterParams, SectorParams, SilhouetteParams, SplitParams,
    encode_direction_u16, pack_meta5, pack_thresholds,
};

// =============================================================================
// Builder API
// =============================================================================

/// Begin building an EPU configuration.
///
/// Returns an `EpuBuilder` that can be used to add bounds and feature layers.
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
/// Automatically manages layer slot allocation:
/// - Bounds (RAMP + bounds ops) goes to slots 0-3
/// - Radiance (feature ops) goes to slots 4-7
pub struct EpuBuilder {
    cfg: EpuConfig,
    next_bounds: usize,
    next_feature: usize,
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
            next_bounds: 0,
            next_feature: 4,
        }
    }

    /// Finish building and return the packed configuration.
    #[inline]
    pub fn finish(self) -> EpuConfig {
        self.cfg
    }

    /// Push a bounds layer (slots 0-3). Silently ignored if full.
    fn push_bounds(&mut self, layer: EpuLayer) {
        if self.next_bounds >= 4 {
            return;
        }
        self.cfg.layers[self.next_bounds] = layer.encode();
        self.next_bounds += 1;
    }

    /// Push a feature layer (slots 4-7). Silently ignored if full.
    fn push_feature(&mut self, layer: EpuLayer) {
        if self.next_feature >= 8 {
            return;
        }
        self.cfg.layers[self.next_feature] = layer.encode();
        self.next_feature += 1;
    }

    // =========================================================================
    // Bounds Helpers
    // =========================================================================

    /// Set the bounds gradient (RAMP) - always goes to slot 0.
    ///
    /// This establishes the base colors and region weights used by all other layers.
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
        // RAMP always goes to slot 0
        self.cfg.layers[0] = layer.encode();
        self.next_bounds = self.next_bounds.max(1);
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

    /// Apply a SILHOUETTE bounds modifier.
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

    /// Add a horizon band (BAND_RADIANCE).
    pub fn band_radiance(&mut self, p: BandRadianceParams) {
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::BandRadiance,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            meta5: 0,
            color_a: p.color,
            color_b: p.edge_color,
            alpha_a: p.alpha,
            alpha_b: 0,
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
}
