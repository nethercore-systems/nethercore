//! EPU runtime settings and mip calculation utilities.
//!
//! This module provides configuration knobs for the EPU radiance generation
//! pipeline, including map size and mip pyramid settings.

/// Default output map size in texels (octahedral).
///
/// Override via [`EpuRuntimeSettings`] or `NETHERCORE_EPU_MAP_SIZE`.
pub const EPU_MAP_SIZE: u32 = 128;

/// Minimum mip size for the EPU radiance pyramid.
///
/// Mips smaller than this provide little value for stylized IBL and can be
/// disproportionately expensive to manage (more passes, tiny dispatches).
///
/// Override via [`EpuRuntimeSettings`] or `NETHERCORE_EPU_MIN_MIP_SIZE`.
pub const EPU_MIN_MIP_SIZE: u32 = 4;

/// Target mip size for diffuse irradiance (SH9) extraction.
///
/// The SH9 pass samples many directions; using a coarser mip reduces noise and
/// better matches "diffuse = low frequency".
pub(super) const EPU_IRRAD_TARGET_SIZE: u32 = 16;

/// Maximum number of environment states that can be processed.
pub const MAX_ENV_STATES: u32 = 256;

/// Initial number of texture array layers (grows on demand).
/// Starting small saves VRAM - most games use < 16 environments.
pub(super) const EPU_INITIAL_LAYERS: u32 = 8;

/// Maximum number of active environments per dispatch.
pub const MAX_ACTIVE_ENVS: u32 = 32;

/// Runtime knobs for EPU radiance generation.
///
/// `map_size` and `min_mip_size` are intentionally exposed to make it easy to
/// experiment with quality/perf tradeoffs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpuRuntimeSettings {
    /// Base EnvRadiance resolution (width == height).
    pub map_size: u32,
    /// Smallest mip level to generate (inclusive).
    pub min_mip_size: u32,
}

impl Default for EpuRuntimeSettings {
    fn default() -> Self {
        Self {
            map_size: EPU_MAP_SIZE,
            min_mip_size: EPU_MIN_MIP_SIZE,
        }
    }
}

impl EpuRuntimeSettings {
    /// Read runtime overrides from environment variables:
    /// - `NETHERCORE_EPU_MAP_SIZE`
    /// - `NETHERCORE_EPU_MIN_MIP_SIZE`
    ///
    /// Invalid values fall back to defaults.
    pub fn from_env() -> Self {
        fn parse_u32(var: &str) -> Option<u32> {
            std::env::var(var).ok()?.parse::<u32>().ok()
        }

        let mut settings = Self::default();

        if let Some(v) = parse_u32("NETHERCORE_EPU_MAP_SIZE") {
            settings.map_size = v;
        }
        if let Some(v) = parse_u32("NETHERCORE_EPU_MIN_MIP_SIZE") {
            settings.min_mip_size = v;
        }

        settings.sanitized()
    }

    /// Clamp/repair settings into a valid state (power-of-two, min<=base).
    #[must_use]
    pub fn sanitized(self) -> Self {
        let mut out = self;

        if out.map_size < 1 {
            out.map_size = EPU_MAP_SIZE;
        }
        if out.min_mip_size < 1 {
            out.min_mip_size = EPU_MIN_MIP_SIZE;
        }

        if !out.map_size.is_power_of_two() {
            out.map_size = out.map_size.next_power_of_two().max(1);
        }
        if !out.min_mip_size.is_power_of_two() {
            out.min_mip_size = out.min_mip_size.next_power_of_two().max(1);
        }

        if out.min_mip_size > out.map_size {
            out.min_mip_size = out.map_size;
        }

        out
    }
}

/// Calculate mip sizes for a pyramid from base_size down to min_size.
///
/// Returns a Vec of sizes starting with base_size and halving until min_size.
/// All sizes are power-of-two.
pub(super) fn calc_mip_sizes(base_size: u32, min_size: u32) -> Vec<u32> {
    debug_assert!(base_size >= 1);
    debug_assert!(min_size >= 1);
    debug_assert!(
        base_size.is_power_of_two() && min_size.is_power_of_two(),
        "EPU mip pyramid assumes power-of-two sizing (base={base_size}, min={min_size})"
    );
    debug_assert!(
        min_size <= base_size,
        "min mip size must be <= base size (base={base_size}, min={min_size})"
    );

    let mut sizes = vec![base_size];
    let mut size = base_size;
    while size > min_size {
        size /= 2;
        sizes.push(size);
    }
    sizes
}

/// Choose the mip level to use for SH9 irradiance extraction.
///
/// Returns the first mip index whose size is <= target_size, or the last mip
/// if all are larger than the target.
pub(super) fn choose_irrad_mip_level(mip_sizes: &[u32], target_size: u32) -> u32 {
    debug_assert!(!mip_sizes.is_empty());
    mip_sizes
        .iter()
        .position(|&s| s <= target_size)
        .unwrap_or(mip_sizes.len().saturating_sub(1)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_mip_sizes() {
        let sizes = calc_mip_sizes(128, 4);
        assert_eq!(sizes, vec![128, 64, 32, 16, 8, 4]);
        assert!(sizes.iter().all(|&s| s.is_power_of_two()));
        assert!(sizes.windows(2).all(|w| w[0] > w[1]));
    }

    #[test]
    fn test_choose_irrad_mip_level() {
        let sizes = calc_mip_sizes(128, 4);
        assert_eq!(choose_irrad_mip_level(&sizes, 16), 3);
        assert_eq!(choose_irrad_mip_level(&sizes, 8), 4);
        assert_eq!(choose_irrad_mip_level(&sizes, 4), 5);
        // If target is smaller than the smallest generated mip, clamp to last.
        assert_eq!(choose_irrad_mip_level(&sizes, 2), 5);
        // If target is larger than base, pick mip 0.
        assert_eq!(choose_irrad_mip_level(&sizes, 256), 0);
    }

    #[test]
    fn test_settings_sanitized_power_of_two() {
        let s = EpuRuntimeSettings {
            map_size: 300,
            min_mip_size: 7,
        }
        .sanitized();

        assert!(s.map_size.is_power_of_two());
        assert!(s.min_mip_size.is_power_of_two());
        assert!(s.min_mip_size <= s.map_size);
    }
}
