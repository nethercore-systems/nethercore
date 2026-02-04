//! GPU-side types for EPU runtime.
//!
//! This module contains the uniform buffer structures and GPU-compatible
//! representations used by the EPU compute shaders.

use super::layer::EpuConfig;

/// Frame uniforms structure matching the WGSL `FrameUniforms` struct.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct FrameUniforms {
    pub active_count: u32,
    pub map_size: u32,
    pub _pad0: u32,
    pub _pad1: u32,
}

/// Irradiance uniforms structure matching the WGSL `IrradUniforms` struct.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct IrradUniforms {
    pub active_count: u32,
    pub _pad0: u32,
    pub _pad1: u32,
    pub _pad2: u32,
}

/// SH9 (L2) diffuse irradiance coefficients.
///
/// These are Lambertian-convolved coefficients in the real SH basis, stored in
/// the following order:
/// `[Y00, Y1-1, Y10, Y11, Y2-2, Y2-1, Y20, Y21, Y22]`.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EpuSh9 {
    pub c0: [f32; 3],
    _pad0: f32,
    pub c1: [f32; 3],
    _pad1: f32,
    pub c2: [f32; 3],
    _pad2: f32,
    pub c3: [f32; 3],
    _pad3: f32,
    pub c4: [f32; 3],
    _pad4: f32,
    pub c5: [f32; 3],
    _pad5: f32,
    pub c6: [f32; 3],
    _pad6: f32,
    pub c7: [f32; 3],
    _pad7: f32,
    pub c8: [f32; 3],
    _pad8: f32,
}

/// GPU representation of an EPU environment state (128-bit format).
///
/// Each layer is 128 bits = 4 x u32 for GPU compatibility.
/// The shader expects `array<vec4u, 8>` where each vec4u represents a 128-bit instruction.
///
/// WGSL vec4u layout: [w0, w1, w2, w3] where:
/// - w0 = bits 31..0   (lo.lo)
/// - w1 = bits 63..32  (lo.hi)
/// - w2 = bits 95..64  (hi.lo)
/// - w3 = bits 127..96 (hi.hi)
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct GpuEnvironmentState {
    /// 8 layers stored as [lo_lo, lo_hi, hi_lo, hi_hi] quadruplets
    /// Total: 8 layers x 4 u32 = 32 u32 = 128 bytes
    pub layers: [[u32; 4]; 8],
}

impl From<&EpuConfig> for GpuEnvironmentState {
    fn from(config: &EpuConfig) -> Self {
        // EpuConfig stores [hi, lo] where hi=bits 127..64, lo=bits 63..0
        // WGSL vec4u needs [w0, w1, w2, w3] = [lo_lo, lo_hi, hi_lo, hi_hi]
        let layers = config.layers.map(|[hi, lo]| {
            [
                (lo & 0xFFFF_FFFF) as u32, // w0 = lo bits 31..0
                (lo >> 32) as u32,         // w1 = lo bits 63..32
                (hi & 0xFFFF_FFFF) as u32, // w2 = hi bits 31..0 (overall bits 95..64)
                (hi >> 32) as u32,         // w3 = hi bits 63..32 (overall bits 127..96)
            ]
        });
        Self { layers }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_uniforms_size() {
        // FrameUniforms must be exactly 16 bytes (4 x u32/f32)
        assert_eq!(
            std::mem::size_of::<FrameUniforms>(),
            16,
            "FrameUniforms must be 16 bytes"
        );
    }

    #[test]
    fn test_irrad_uniforms_size() {
        // IrradUniforms must be exactly 16 bytes (4 x u32)
        assert_eq!(
            std::mem::size_of::<IrradUniforms>(),
            16,
            "IrradUniforms must be 16 bytes"
        );
    }

    #[test]
    fn test_sh9_size() {
        // EpuSh9 must be exactly 144 bytes (9 coefficients x 16 bytes each)
        assert_eq!(
            std::mem::size_of::<EpuSh9>(),
            144,
            "EpuSh9 must be 144 bytes"
        );
    }

    #[test]
    fn test_gpu_environment_state_size() {
        // GpuEnvironmentState must be exactly 128 bytes (8 layers x 16 bytes)
        assert_eq!(
            std::mem::size_of::<GpuEnvironmentState>(),
            128,
            "GpuEnvironmentState must be 128 bytes"
        );
    }

    #[test]
    fn test_gpu_environment_state_conversion() {
        let config = EpuConfig {
            layers: [
                [0x1234_5678_9ABC_DEF0, 0xFEDC_BA98_7654_3210], // layer 0: [hi, lo]
                [0xAAAA_BBBB_CCCC_DDDD, 0x1111_2222_3333_4444], // layer 1: [hi, lo]
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
            ],
        };

        let gpu_state = GpuEnvironmentState::from(&config);

        // Check first layer [hi, lo] -> [lo_lo, lo_hi, hi_lo, hi_hi]
        // hi = 0x1234_5678_9ABC_DEF0 -> hi_hi=0x1234_5678, hi_lo=0x9ABC_DEF0
        // lo = 0xFEDC_BA98_7654_3210 -> lo_hi=0xFEDC_BA98, lo_lo=0x7654_3210
        assert_eq!(gpu_state.layers[0][0], 0x7654_3210); // w0 = lo_lo
        assert_eq!(gpu_state.layers[0][1], 0xFEDC_BA98); // w1 = lo_hi
        assert_eq!(gpu_state.layers[0][2], 0x9ABC_DEF0); // w2 = hi_lo
        assert_eq!(gpu_state.layers[0][3], 0x1234_5678); // w3 = hi_hi

        // Check second layer
        // hi = 0xAAAA_BBBB_CCCC_DDDD -> hi_hi=0xAAAA_BBBB, hi_lo=0xCCCC_DDDD
        // lo = 0x1111_2222_3333_4444 -> lo_hi=0x1111_2222, lo_lo=0x3333_4444
        assert_eq!(gpu_state.layers[1][0], 0x3333_4444); // w0 = lo_lo
        assert_eq!(gpu_state.layers[1][1], 0x1111_2222); // w1 = lo_hi
        assert_eq!(gpu_state.layers[1][2], 0xCCCC_DDDD); // w2 = hi_lo
        assert_eq!(gpu_state.layers[1][3], 0xAAAA_BBBB); // w3 = hi_hi

        // Rest should be zero
        for i in 2..8 {
            assert_eq!(gpu_state.layers[i], [0, 0, 0, 0]);
        }
    }
}
