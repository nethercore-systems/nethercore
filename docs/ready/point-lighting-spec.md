# Point Lighting System - Implementation Specification

This document specifies adding point light support to the Emberware Z lighting system. Point lights have position, range, and attenuation, compared to directional lights which only have direction.

---

## Current System Summary

**PackedLight (8 bytes):**
```rust
pub struct PackedLight {
    pub direction_oct: u32,       // Octahedral snorm16x2
    pub color_and_intensity: u32, // 0xRRGGBBII format
}
```

**Bit Layout (color_and_intensity):**
- Bits 31-24: Red (u8)
- Bits 23-16: Green (u8)
- Bits 15-8: Blue (u8)
- Bits 7-0: Intensity (u8, 0-255 -> 0.0-1.0)

**Key Files:**
- [unified_shading_state.rs](../emberware-z/src/graphics/unified_shading_state.rs) - PackedLight struct
- [common.wgsl](../emberware-z/shaders/common.wgsl) - Shader unpacking
- [blinnphong_common.wgsl](../emberware-z/shaders/blinnphong_common.wgsl) - Lighting loop
- [lighting.rs](../emberware-z/src/ffi/lighting.rs) - FFI functions

---

## Proposed Changes

### 1. Expand PackedLight to 12 bytes

```rust
// emberware-z/src/graphics/unified_shading_state.rs

/// One packed light (12 bytes) - supports directional and point lights
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedLight {
    /// Directional: octahedral direction (snorm16x2)
    /// Point: position XY (f16x2)
    pub data0: u32,

    /// RGB8 (bits 31-8) + type (bit 7) + intensity (bits 6-0)
    /// Format: 0xRRGGBB_TI where T=type(1bit), I=intensity(7bits)
    pub data1: u32,

    /// Directional: unused (0)
    /// Point: position Z (f16, bits 15-0) + range (f16, bits 31-16)
    pub data2: u32,
}
```

**Impact on PackedUnifiedShadingState:**
- lights array: 48 bytes (was 32)
- Total size: 80 bytes (was 64)

### 2. Bit Layout Details

**data1 format (both light types):**
```
Bits 31-24: Red (u8, 0-255 -> 0.0-1.0)
Bits 23-16: Green (u8)
Bits 15-8:  Blue (u8)
Bit 7:      Type (0 = Directional, 1 = Point)
Bits 6-0:   Intensity (u7, 0-127 -> 0.0-8.0 for HDR support)
```

**data0 for directional lights:**
```
Bits 15-0:  U component (snorm16, -32767..32767 -> -1.0..1.0)
Bits 31-16: V component (snorm16)
```

**data0 for point lights:**
```
Bits 15-0:  Position X (f16, IEEE 754 half-precision)
Bits 31-16: Position Y (f16)
```

**data2 for point lights:**
```
Bits 15-0:  Position Z (f16)
Bits 31-16: Range (f16, distance for full attenuation)
```

---

## Rust Implementation

### 3. F16 Packing Utilities

Add to [unified_shading_state.rs](../emberware-z/src/graphics/unified_shading_state.rs):

```rust
/// Pack f32 to IEEE 754 half-precision float (f16) stored as u16
#[inline]
pub fn pack_f16(value: f32) -> u16 {
    half::f16::from_f32(value).to_bits()
}

/// Unpack IEEE 754 half-precision float (f16) from u16 to f32
#[inline]
pub fn unpack_f16(bits: u16) -> f32 {
    half::f16::from_bits(bits).to_f32()
}

/// Pack two f32 values into a u32 as f16x2
#[inline]
pub fn pack_f16x2(x: f32, y: f32) -> u32 {
    let x_bits = pack_f16(x) as u32;
    let y_bits = pack_f16(y) as u32;
    x_bits | (y_bits << 16)
}

/// Unpack u32 to two f32 values from f16x2
#[inline]
pub fn unpack_f16x2(packed: u32) -> (f32, f32) {
    let x = unpack_f16((packed & 0xFFFF) as u16);
    let y = unpack_f16((packed >> 16) as u16);
    (x, y)
}
```

**Add to [Cargo.toml](../emberware-z/Cargo.toml):**
```toml
half = "2.4"
```

### 4. Light Type Enum

Add to [unified_shading_state.rs](../emberware-z/src/graphics/unified_shading_state.rs):

```rust
/// Light type stored in bit 7 of data1
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LightType {
    #[default]
    Directional = 0,
    Point = 1,
}

impl LightType {
    pub fn from_bit(bit: bool) -> Self {
        if bit { LightType::Point } else { LightType::Directional }
    }

    pub fn to_bit(self) -> bool {
        matches!(self, LightType::Point)
    }
}
```

### 5. Updated PackedLight Implementation

Replace the existing `impl PackedLight` block in [unified_shading_state.rs](../emberware-z/src/graphics/unified_shading_state.rs):

```rust
impl PackedLight {
    /// Create a directional light
    pub fn directional(direction: Vec3, color: Vec3, intensity: f32, enabled: bool) -> Self {
        let data0 = pack_octahedral_u32(direction.normalize_or_zero());
        let data1 = Self::pack_color_type_intensity(color, LightType::Directional, intensity, enabled);
        Self { data0, data1, data2: 0 }
    }

    /// Create a point light
    pub fn point(position: Vec3, color: Vec3, intensity: f32, range: f32, enabled: bool) -> Self {
        let data0 = pack_f16x2(position.x, position.y);
        let data1 = Self::pack_color_type_intensity(color, LightType::Point, intensity, enabled);
        let data2 = pack_f16x2(position.z, range);
        Self { data0, data1, data2 }
    }

    /// Pack color, type, and intensity into data1
    /// Format: 0xRRGGBB_TI where T=type(1bit), I=intensity(7bits)
    fn pack_color_type_intensity(color: Vec3, light_type: LightType, intensity: f32, enabled: bool) -> u32 {
        let r = pack_unorm8(color.x);
        let g = pack_unorm8(color.y);
        let b = pack_unorm8(color.z);

        // Intensity: 0.0-8.0 -> 0-127 (7 bits)
        // If disabled, set to 0
        let intensity_7bit = if enabled {
            ((intensity / 8.0).clamp(0.0, 1.0) * 127.0).round() as u8
        } else {
            0
        };

        // Type in bit 7, intensity in bits 0-6
        let type_intensity = ((light_type as u8) << 7) | (intensity_7bit & 0x7F);

        ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (type_intensity as u32)
    }

    /// Create a disabled light (all zeros)
    pub fn disabled() -> Self {
        Self::default()
    }

    /// Get light type (directional or point)
    pub fn get_type(&self) -> LightType {
        LightType::from_bit((self.data1 & 0x80) != 0)
    }

    /// Get color as [f32; 3]
    pub fn get_color(&self) -> [f32; 3] {
        let r = unpack_unorm8(((self.data1 >> 24) & 0xFF) as u8);
        let g = unpack_unorm8(((self.data1 >> 16) & 0xFF) as u8);
        let b = unpack_unorm8(((self.data1 >> 8) & 0xFF) as u8);
        [r, g, b]
    }

    /// Get intensity as f32 (0.0-8.0 range)
    pub fn get_intensity(&self) -> f32 {
        let intensity_7bit = (self.data1 & 0x7F) as f32;
        intensity_7bit / 127.0 * 8.0
    }

    /// Check if light is enabled (intensity > 0)
    pub fn is_enabled(&self) -> bool {
        (self.data1 & 0x7F) != 0
    }

    /// Get direction (only valid for directional lights)
    pub fn get_direction(&self) -> [f32; 3] {
        let dir = unpack_octahedral_u32(self.data0);
        [dir.x, dir.y, dir.z]
    }

    /// Get position (only valid for point lights)
    pub fn get_position(&self) -> [f32; 3] {
        let (x, y) = unpack_f16x2(self.data0);
        let (z, _) = unpack_f16x2(self.data2);
        [x, y, z]
    }

    /// Get range (only valid for point lights)
    pub fn get_range(&self) -> f32 {
        let (_, range) = unpack_f16x2(self.data2);
        range
    }
}
```

### 6. Update existing `from_floats` method

The existing `PackedLight::from_floats()` should delegate to `directional()`:

```rust
impl PackedLight {
    /// Create a PackedLight from f32 parameters (directional light)
    /// If enabled=false, intensity is set to 0 (which indicates disabled light)
    pub fn from_floats(direction: Vec3, color: Vec3, intensity: f32, enabled: bool) -> Self {
        Self::directional(direction, color, intensity, enabled)
    }
}
```

---

## WGSL Shader Implementation

### 7. Updated Shader Structures

Update [common.wgsl](../emberware-z/shaders/common.wgsl) PackedLight struct:

```wgsl
// Packed light data (12 bytes)
struct PackedLight {
    data0: u32,  // Direction (oct) or Position XY (f16x2)
    data1: u32,  // RGB8 + type(1) + intensity(7)
    data2: u32,  // Unused (directional) or Position Z + Range (f16x2)
}
```

Update the LightData struct:

```wgsl
// Unpacked light data (extended for point lights)
struct LightData {
    direction: vec3<f32>,    // For directional lights
    position: vec3<f32>,     // For point lights
    color: vec3<f32>,
    intensity: f32,
    range: f32,              // For point lights
    light_type: u32,         // 0 = directional, 1 = point
    enabled: bool,
}
```

### 8. F16 Unpacking Utilities

Add to [common.wgsl](../emberware-z/shaders/common.wgsl) after existing unpacking functions:

```wgsl
// Unpack two f16 values from u32
// Uses WGSL builtin unpack2x16float
fn unpack_f16x2(packed: u32) -> vec2<f32> {
    return unpack2x16float(packed);
}
```

### 9. Updated Light Unpacking

Replace the `unpack_light` function in [common.wgsl](../emberware-z/shaders/common.wgsl):

```wgsl
// Unpack PackedLight to usable values
fn unpack_light(packed: PackedLight) -> LightData {
    var light: LightData;

    // Extract type from bit 7 of data1
    light.light_type = (packed.data1 >> 7u) & 1u;

    // Extract intensity from bits 0-6, map 0-127 -> 0.0-8.0
    let intensity_7bit = f32(packed.data1 & 0x7Fu);
    light.intensity = intensity_7bit / 127.0 * 8.0;
    light.enabled = intensity_7bit > 0.0;

    // Extract color (bits 31-8)
    light.color = vec3<f32>(
        f32((packed.data1 >> 24u) & 0xFFu) / 255.0,
        f32((packed.data1 >> 16u) & 0xFFu) / 255.0,
        f32((packed.data1 >> 8u) & 0xFFu) / 255.0
    );

    if (light.light_type == 0u) {
        // Directional light: unpack octahedral direction
        light.direction = unpack_octahedral(packed.data0);
        light.position = vec3<f32>(0.0);
        light.range = 0.0;
    } else {
        // Point light: unpack position (f16x2 + f16) and range (f16)
        let xy = unpack_f16x2(packed.data0);
        let z_range = unpack_f16x2(packed.data2);
        light.position = vec3<f32>(xy.x, xy.y, z_range.x);
        light.range = z_range.y;
        light.direction = vec3<f32>(0.0, -1.0, 0.0);  // Default, unused
    }

    return light;
}
```

### 10. Point Light Attenuation

Add to [blinnphong_common.wgsl](../emberware-z/shaders/blinnphong_common.wgsl) after existing lighting functions:

```wgsl
// Smooth distance attenuation for point lights
// Returns 1.0 at distance=0, 0.0 at distance>=range
fn point_light_attenuation(distance: f32, range: f32) -> f32 {
    if (range <= 0.0) {
        return 0.0;
    }
    let t = clamp(distance / range, 0.0, 1.0);
    // Smooth falloff: quadratic ease-out (feels more natural than linear)
    let inv_t = 1.0 - t;
    return inv_t * inv_t;
}
```

### 11. Updated Lighting Loop

Replace the lighting loop in [blinnphong_common.wgsl](../emberware-z/shaders/blinnphong_common.wgsl) fragment shader (around line 164):

```wgsl
    // 4 dynamic lights (direct illumination)
    for (var i = 0u; i < 4u; i++) {
        let light = unpack_light(shading.lights[i]);
        if (light.enabled) {
            var light_dir: vec3<f32>;
            var attenuation: f32 = 1.0;

            if (light.light_type == 0u) {
                // Directional light: use stored direction
                light_dir = light.direction;
            } else {
                // Point light: compute direction and attenuation
                let to_light = light.position - in.world_position;
                let distance = length(to_light);
                light_dir = -normalize(to_light);  // Negate: convention is "ray direction"
                attenuation = point_light_attenuation(distance, light.range);
            }

            let light_color = light.color * light.intensity * attenuation;

            // Diffuse
            final_color += lambert_diffuse(N, light_dir, albedo, light_color)
                           * diffuse_factor * diffuse_fresnel;

            // Specular
            final_color += normalized_blinn_phong_specular(
                N, view_dir, light_dir, shininess, specular_color, light_color
            );
        }
    }
```

---

## FFI Implementation

### 12. New FFI Functions

Add to [lighting.rs](../emberware-z/src/ffi/lighting.rs):

```rust
use crate::graphics::unified_shading_state::LightType;

/// Register lighting FFI functions
pub fn register(linker: &mut Linker<GameStateWithConsole<ZInput, ZFFIState>>) -> Result<()> {
    // Existing directional light functions
    linker.func_wrap("env", "light_set", light_set)?;
    linker.func_wrap("env", "light_color", light_color)?;
    linker.func_wrap("env", "light_intensity", light_intensity)?;
    linker.func_wrap("env", "light_enable", light_enable)?;
    linker.func_wrap("env", "light_disable", light_disable)?;

    // New point light functions
    linker.func_wrap("env", "light_set_point", light_set_point)?;
    linker.func_wrap("env", "light_range", light_range)?;

    Ok(())
}

/// Set light as point light with position
///
/// # Arguments
/// * `index` - Light index (0-3)
/// * `x` - World-space X position
/// * `y` - World-space Y position
/// * `z` - World-space Z position
///
/// Converts the light to a point light and sets its position.
/// Use `light_range()` to set the falloff distance.
/// Use `light_color()` and `light_intensity()` for color/brightness.
fn light_set_point(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    index: u32,
    x: f32,
    y: f32,
    z: f32,
) {
    if index > 3 {
        warn!("light_set_point: invalid light index {} (must be 0-3)", index);
        return;
    }

    let state = &mut caller.data_mut().console;
    let light = &state.current_shading_state.lights[index as usize];
    let color = light.get_color();
    let intensity = light.get_intensity();
    let range = if light.get_type() == LightType::Point {
        light.get_range()
    } else {
        10.0  // Default range for new point lights
    };

    state.update_point_light(index as usize, [x, y, z], color, intensity, range, true);
}

/// Set point light range (falloff distance)
///
/// # Arguments
/// * `index` - Light index (0-3)
/// * `range` - Distance at which light reaches zero intensity
///
/// Only affects point lights. Directional lights ignore this.
fn light_range(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    index: u32,
    range: f32,
) {
    if index > 3 {
        warn!("light_range: invalid light index {} (must be 0-3)", index);
        return;
    }

    let range = range.max(0.0);  // Clamp negative to 0

    let state = &mut caller.data_mut().console;
    let light = &state.current_shading_state.lights[index as usize];

    // Only valid for point lights
    if light.get_type() != LightType::Point {
        warn!("light_range: light {} is directional, not point", index);
        return;
    }

    let position = light.get_position();
    let color = light.get_color();
    let intensity = light.get_intensity();
    let enabled = light.is_enabled();

    state.update_point_light(index as usize, position, color, intensity, range, enabled);
}
```

### 13. State Update Functions

Add to [ffi_state.rs](../emberware-z/src/state/ffi_state.rs) in the `impl ZFFIState` block:

```rust
use crate::graphics::unified_shading_state::PackedLight;
use glam::Vec3;

impl ZFFIState {
    /// Update a directional light
    pub fn update_light(
        &mut self,
        index: usize,
        direction: [f32; 3],
        color: [f32; 3],
        intensity: f32,
        enabled: bool,
    ) {
        self.current_shading_state.lights[index] = PackedLight::directional(
            Vec3::from(direction),
            Vec3::from(color),
            intensity,
            enabled,
        );
        self.shading_state_dirty = true;
    }

    /// Update a point light
    pub fn update_point_light(
        &mut self,
        index: usize,
        position: [f32; 3],
        color: [f32; 3],
        intensity: f32,
        range: f32,
        enabled: bool,
    ) {
        self.current_shading_state.lights[index] = PackedLight::point(
            Vec3::from(position),
            Vec3::from(color),
            intensity,
            range,
            enabled,
        );
        self.shading_state_dirty = true;
    }
}
```

---

## FFI Documentation Update

### 14. Add to docs/ffi.md

Add this section under the existing lighting documentation:

```markdown
### Point Lights

Point lights emit light from a position in world space with distance-based falloff.

#### `light_set_point(index: u32, x: f32, y: f32, z: f32)`

Set a light as a point light at the given world-space position.

**Parameters:**
- `index` - Light slot (0-3)
- `x, y, z` - World-space position

**Example:**
```c
// Place a point light above the player
light_set_point(0, player_x, player_y + 2.0, player_z);
light_color(0, 0xFFAA44FF);  // Warm orange
light_intensity(0, 2.0);
light_range(0, 15.0);
```

#### `light_range(index: u32, range: f32)`

Set the falloff distance for a point light. Light intensity reaches zero at this distance.

**Parameters:**
- `index` - Light slot (0-3)
- `range` - Falloff distance in world units

**Attenuation:** Uses smooth quadratic falloff: `(1 - d/range)^2`

**Note:** Only affects point lights. Call `light_set_point()` first to convert a directional light to a point light.
```

---

## Unit Tests

### 15. Add to unified_shading_state.rs tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packed_light_size() {
        assert_eq!(std::mem::size_of::<PackedLight>(), 12);
    }

    #[test]
    fn test_packed_shading_state_size() {
        assert_eq!(std::mem::size_of::<PackedUnifiedShadingState>(), 80);
    }

    #[test]
    fn test_directional_light_roundtrip() {
        let dir = Vec3::new(0.5, -0.7, 0.3).normalize();
        let color = Vec3::new(1.0, 0.5, 0.25);
        let intensity = 2.5;

        let light = PackedLight::directional(dir, color, intensity, true);

        assert_eq!(light.get_type(), LightType::Directional);
        assert!(light.is_enabled());

        let unpacked_dir = light.get_direction();
        assert!((unpacked_dir[0] - dir.x).abs() < 0.01);
        assert!((unpacked_dir[1] - dir.y).abs() < 0.01);
        assert!((unpacked_dir[2] - dir.z).abs() < 0.01);

        let unpacked_color = light.get_color();
        assert!((unpacked_color[0] - color.x).abs() < 0.01);
        assert!((unpacked_color[1] - color.y).abs() < 0.01);
        assert!((unpacked_color[2] - color.z).abs() < 0.01);

        // Intensity with 7-bit precision in 0-8 range
        let unpacked_intensity = light.get_intensity();
        assert!((unpacked_intensity - intensity).abs() < 0.1);
    }

    #[test]
    fn test_point_light_roundtrip() {
        let pos = Vec3::new(10.5, -5.25, 100.0);
        let color = Vec3::new(0.8, 0.6, 0.4);
        let intensity = 4.0;
        let range = 25.0;

        let light = PackedLight::point(pos, color, intensity, range, true);

        assert_eq!(light.get_type(), LightType::Point);
        assert!(light.is_enabled());

        let unpacked_pos = light.get_position();
        // f16 precision is about 3 decimal digits
        assert!((unpacked_pos[0] - pos.x).abs() < 0.1);
        assert!((unpacked_pos[1] - pos.y).abs() < 0.1);
        assert!((unpacked_pos[2] - pos.z).abs() < 1.0);

        let unpacked_range = light.get_range();
        assert!((unpacked_range - range).abs() < 0.5);
    }

    #[test]
    fn test_f16_packing() {
        let values = [0.0, 1.0, -1.0, 100.0, 0.001, 65504.0];
        for v in values {
            let packed = pack_f16(v);
            let unpacked = unpack_f16(packed);
            let error = (unpacked - v).abs() / v.abs().max(1.0);
            assert!(error < 0.01, "f16 roundtrip failed for {}: got {}", v, unpacked);
        }
    }

    #[test]
    fn test_f16x2_packing() {
        let (x, y) = (42.5, -17.25);
        let packed = pack_f16x2(x, y);
        let (ux, uy) = unpack_f16x2(packed);
        assert!((ux - x).abs() < 0.1);
        assert!((uy - y).abs() < 0.1);
    }

    #[test]
    fn test_intensity_range() {
        // Test intensity at various points in 0-8 range
        for intensity in [0.0, 1.0, 2.0, 4.0, 7.9] {
            let light = PackedLight::directional(
                Vec3::new(0.0, -1.0, 0.0),
                Vec3::ONE,
                intensity,
                true,
            );
            let unpacked = light.get_intensity();
            assert!(
                (unpacked - intensity).abs() < 0.1,
                "intensity {} unpacked to {}",
                intensity,
                unpacked
            );
        }
    }

    #[test]
    fn test_disabled_light() {
        let light = PackedLight::directional(
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::ONE,
            1.0,
            false,  // disabled
        );
        assert!(!light.is_enabled());
        assert_eq!(light.get_intensity(), 0.0);
    }
}
```

---

## Files to Modify

| File | Changes |
|------|---------|
| [emberware-z/Cargo.toml](../emberware-z/Cargo.toml) | Add `half = "2.4"` dependency |
| [unified_shading_state.rs](../emberware-z/src/graphics/unified_shading_state.rs) | Expand PackedLight to 12 bytes, add f16 utils, add LightType enum |
| [common.wgsl](../emberware-z/shaders/common.wgsl) | Update PackedLight struct, add f16 unpacking, update unpack_light |
| [blinnphong_common.wgsl](../emberware-z/shaders/blinnphong_common.wgsl) | Add point_light_attenuation, update lighting loop |
| [lighting.rs](../emberware-z/src/ffi/lighting.rs) | Add light_set_point, light_range functions |
| [ffi_state.rs](../emberware-z/src/state/ffi_state.rs) | Add update_point_light method |
| [ffi.md](../docs/ffi.md) | Document new FFI functions |

---

## Migration Notes

**Backward Compatibility:**
- Existing games using `light_set()` continue to work (directional lights)
- Default light type is Directional (type bit = 0)
- PackedUnifiedShadingState size increase (64->80 bytes) requires buffer reallocation

**Breaking Changes:**
- Intensity range changes from 0-255->0.0-1.0 to 0-127->0.0-8.0
- Existing intensity values will be ~8x brighter if not adjusted
- Consider adding a version flag or migration path

**Recommended Migration:**
```rust
// Old: intensity 0.5 was stored as u8(127), mapped to 0.5
// New: intensity 0.5 is stored as u7(8), mapped from 0.5/8.0*127

// To maintain compatibility, existing code should divide intensity by 8:
// light_intensity(0, old_intensity / 8.0);
```

---

## Implementation Order

1. Add `half` dependency to Cargo.toml
2. Add f16 packing utilities to unified_shading_state.rs
3. Add LightType enum to unified_shading_state.rs
4. Update PackedLight struct (3 fields) and impl
5. Update tests for new struct sizes
6. Update common.wgsl PackedLight and unpack_light
7. Add point_light_attenuation to blinnphong_common.wgsl
8. Update lighting loop in blinnphong_common.wgsl
9. Add new FFI functions to lighting.rs
10. Add update_point_light to ffi_state.rs
11. Update ffi.md documentation
12. Run `cargo test` and `cargo run` to verify
13. Update `examples/lighting` to demonstrate point lights
14. Create `examples/point-lights` as a dedicated point light showcase

---

## Example Updates

### 16. Update examples/lighting/src/lib.rs

Add point light FFI declarations and demonstration. The existing example uses directional lights; we'll add a point light that orbits the sphere.

**Add to extern declarations:**
```rust
#[link(wasm_import_module = "env")]
extern "C" {
    // ... existing declarations ...

    // Point lights (new)
    fn light_set_point(index: u32, x: f32, y: f32, z: f32);
    fn light_range(index: u32, range: f32);
}
```

**Add point light state:**
```rust
/// Point light orbit angle (degrees)
static mut POINT_LIGHT_ORBIT: f32 = 0.0;

/// Point light settings
static mut POINT_LIGHT_RANGE: f32 = 5.0;
static mut POINT_LIGHT_HEIGHT: f32 = 1.5;
static mut POINT_LIGHT_RADIUS: f32 = 2.0;
```

**Update init() to set up light 3 as a point light:**
```rust
// In init(), after initializing directional lights:
// Set light 3 as an orbiting point light
light_set_point(3, POINT_LIGHT_RADIUS, POINT_LIGHT_HEIGHT, 0.0);
light_color(3, 0xFF6600FF);  // Orange
light_intensity(3, 3.0);     // Brighter for point lights
light_range(3, POINT_LIGHT_RANGE);
LIGHT_ENABLED[3] = true;
light_enable(3);
```

**Update update() to orbit the point light:**
```rust
// In update(), add point light orbit:
// Orbit point light around the sphere
POINT_LIGHT_ORBIT += 1.0;
if POINT_LIGHT_ORBIT >= 360.0 {
    POINT_LIGHT_ORBIT -= 360.0;
}
let orbit_rad = POINT_LIGHT_ORBIT * 0.0174533; // deg to rad
let px = fast_sin(orbit_rad) * POINT_LIGHT_RADIUS;
let pz = fast_cos(orbit_rad) * POINT_LIGHT_RADIUS;
light_set_point(3, px, POINT_LIGHT_HEIGHT, pz);

// Adjust range with bumpers (LB/RB)
if button_held(0, BUTTON_LB) != 0 {
    POINT_LIGHT_RANGE = (POINT_LIGHT_RANGE - 0.05).max(1.0);
    light_range(3, POINT_LIGHT_RANGE);
}
if button_held(0, BUTTON_RB) != 0 {
    POINT_LIGHT_RANGE = (POINT_LIGHT_RANGE + 0.05).min(15.0);
    light_range(3, POINT_LIGHT_RANGE);
}
```

**Update render() UI to show point light info:**
```rust
// In render(), add point light status:
let point_label = b"Point Light (LB/RB): Range ";
let len = format_float(POINT_LIGHT_RANGE, &mut buf[point_label.len()..]);
buf[..point_label.len()].copy_from_slice(point_label);
draw_text(buf.as_ptr(), (point_label.len() + len) as u32, 20.0, y + line_h * 8.0, 14.0, 0xFF6600FF);
```

### 17. Create examples/point-lights/src/lib.rs

A dedicated example showcasing point light features: multiple point lights, range visualization, and dynamic positioning.

**File: examples/point-lights/src/lib.rs**
```rust
//! Point Lights Example
//!
//! Demonstrates Emberware Z's point light system.
//!
//! Features demonstrated:
//! - `light_set_point()` to position point lights in world space
//! - `light_range()` to control light falloff distance
//! - Multiple point lights with different colors
//! - Dynamic light positioning following game objects
//!
//! Controls:
//! - Left stick: Move player cube
//! - Right stick: Rotate camera
//! - D-pad Up/Down: Adjust point light range
//! - A/B/X: Toggle lights 0-2
//! - Y: Toggle player-attached light

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "env")]
extern "C" {
    fn set_clear_color(color: u32);
    fn render_mode(mode: u32);
    fn camera_set(x: f32, y: f32, z: f32, tx: f32, ty: f32, tz: f32);
    fn camera_fov(fov: f32);
    fn left_stick_x(player: u32) -> f32;
    fn left_stick_y(player: u32) -> f32;
    fn button_pressed(player: u32, button: u32) -> u32;
    fn button_held(player: u32, button: u32) -> u32;

    // Point light functions
    fn light_set_point(index: u32, x: f32, y: f32, z: f32);
    fn light_color(index: u32, color: u32);
    fn light_intensity(index: u32, intensity: f32);
    fn light_range(index: u32, range: f32);
    fn light_enable(index: u32);
    fn light_disable(index: u32);

    // Sky and materials
    fn sky_set_colors(horizon: u32, zenith: u32);
    fn sky_set_sun(dx: f32, dy: f32, dz: f32, color: u32, sharpness: f32);
    fn draw_sky();
    fn material_metallic(v: f32);
    fn material_roughness(v: f32);

    // Procedural meshes
    fn cube(size: f32) -> u32;
    fn sphere(radius: f32, segments: u32, rings: u32) -> u32;

    // Drawing
    fn draw_mesh(handle: u32);
    fn push_identity();
    fn translate(x: f32, y: f32, z: f32);
    fn scale(x: f32, y: f32, z: f32);
    fn set_color(color: u32);
    fn depth_test(enabled: u32);
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
}

// Button constants
const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;
const BUTTON_A: u32 = 4;
const BUTTON_B: u32 = 5;
const BUTTON_X: u32 = 6;
const BUTTON_Y: u32 = 7;

// Mesh handles
static mut FLOOR_MESH: u32 = 0;
static mut PLAYER_MESH: u32 = 0;
static mut LIGHT_SPHERE: u32 = 0;

// Player position
static mut PLAYER_X: f32 = 0.0;
static mut PLAYER_Z: f32 = 0.0;

// Point light positions (3 static + 1 player-attached)
static mut LIGHT_POSITIONS: [[f32; 3]; 4] = [
    [-3.0, 1.5, -3.0],  // Red corner light
    [3.0, 1.5, -3.0],   // Green corner light
    [0.0, 1.5, 3.0],    // Blue back light
    [0.0, 2.0, 0.0],    // Player-attached (updated in update)
];

// Light colors
static LIGHT_COLORS: [u32; 4] = [
    0xFF3333FF,  // Red
    0x33FF33FF,  // Green
    0x3333FFFF,  // Blue
    0xFFFF66FF,  // Yellow (player light)
];

// Light enabled states
static mut LIGHT_ENABLED: [bool; 4] = [true, true, true, true];

// Point light range (shared for simplicity)
static mut LIGHT_RANGE: f32 = 6.0;

fn fast_sin(x: f32) -> f32 {
    const PI: f32 = 3.14159265359;
    let mut x = x;
    while x > PI { x -= 2.0 * PI; }
    while x < -PI { x += 2.0 * PI; }
    let abs_x = if x < 0.0 { -x } else { x };
    16.0 * x * (PI - abs_x) / (5.0 * PI * PI - 4.0 * abs_x * (PI - abs_x))
}

fn fast_cos(x: f32) -> f32 {
    fast_sin(x + 1.57079632679)
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x0A0A14FF);  // Very dark blue
        render_mode(2);  // PBR mode

        camera_set(0.0, 8.0, 10.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);
        depth_test(1);

        // Create meshes
        FLOOR_MESH = cube(1.0);   // Will be scaled to a flat floor
        PLAYER_MESH = cube(1.0);  // Player cube
        LIGHT_SPHERE = sphere(0.2, 16, 8);  // Small spheres to visualize light positions

        // Set up all 4 point lights
        for i in 0..4u32 {
            let pos = LIGHT_POSITIONS[i as usize];
            light_set_point(i, pos[0], pos[1], pos[2]);
            light_color(i, LIGHT_COLORS[i as usize]);
            light_intensity(i, 2.0);
            light_range(i, LIGHT_RANGE);
            light_enable(i);
        }

        // Default material
        material_metallic(0.1);
        material_roughness(0.6);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Move player with left stick
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);
        PLAYER_X += stick_x * 0.1;
        PLAYER_Z -= stick_y * 0.1;  // Invert for intuitive forward/back

        // Clamp to floor bounds
        PLAYER_X = PLAYER_X.clamp(-4.5, 4.5);
        PLAYER_Z = PLAYER_Z.clamp(-4.5, 4.5);

        // Update player-attached light position (light 3)
        LIGHT_POSITIONS[3][0] = PLAYER_X;
        LIGHT_POSITIONS[3][2] = PLAYER_Z;
        if LIGHT_ENABLED[3] {
            light_set_point(3, PLAYER_X, 2.0, PLAYER_Z);
        }

        // Adjust range with D-pad
        if button_held(0, BUTTON_UP) != 0 {
            LIGHT_RANGE = (LIGHT_RANGE + 0.05).min(15.0);
            for i in 0..4u32 {
                if LIGHT_ENABLED[i as usize] {
                    light_range(i, LIGHT_RANGE);
                }
            }
        }
        if button_held(0, BUTTON_DOWN) != 0 {
            LIGHT_RANGE = (LIGHT_RANGE - 0.05).max(1.0);
            for i in 0..4u32 {
                if LIGHT_ENABLED[i as usize] {
                    light_range(i, LIGHT_RANGE);
                }
            }
        }

        // Toggle lights with face buttons
        let buttons = [BUTTON_A, BUTTON_B, BUTTON_X, BUTTON_Y];
        for i in 0..4 {
            if button_pressed(0, buttons[i]) != 0 {
                LIGHT_ENABLED[i] = !LIGHT_ENABLED[i];
                if LIGHT_ENABLED[i] {
                    light_enable(i as u32);
                } else {
                    light_disable(i as u32);
                }
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Dim sky (point lights are the main illumination)
        sky_set_colors(0x1A1A2EFF, 0x0D0D1AFF);
        sky_set_sun(-0.5, -0.3, -0.8, 0x333344FF, 0.5);  // Very dim sun
        draw_sky();

        // Draw floor (scaled flat cube)
        push_identity();
        translate(0.0, -0.5, 0.0);
        scale(10.0, 0.1, 10.0);
        material_roughness(0.8);
        set_color(0x404040FF);
        draw_mesh(FLOOR_MESH);

        // Draw player cube
        push_identity();
        translate(PLAYER_X, 0.5, PLAYER_Z);
        material_roughness(0.3);
        material_metallic(0.8);
        set_color(0xCCCCCCFF);
        draw_mesh(PLAYER_MESH);

        // Draw light position indicators (small bright spheres)
        material_metallic(0.0);
        material_roughness(0.1);
        for i in 0..4 {
            if LIGHT_ENABLED[i] {
                let pos = LIGHT_POSITIONS[i];
                push_identity();
                translate(pos[0], pos[1], pos[2]);
                set_color(LIGHT_COLORS[i]);
                draw_mesh(LIGHT_SPHERE);
            }
        }

        // UI
        let title = b"Point Lights Demo";
        draw_text(title.as_ptr(), title.len() as u32, 20.0, 20.0, 20.0, 0xFFFFFFFF);

        let controls = b"L-Stick: Move  D-pad: Range  A/B/X/Y: Toggle Lights";
        draw_text(controls.as_ptr(), controls.len() as u32, 20.0, 50.0, 14.0, 0x888888FF);

        // Show range value
        let range_label = b"Range: ";
        let mut buf = [0u8; 16];
        buf[..range_label.len()].copy_from_slice(range_label);
        let whole = LIGHT_RANGE as i32;
        let frac = ((LIGHT_RANGE - whole as f32) * 10.0) as i32;
        buf[range_label.len()] = b'0' + whole as u8;
        buf[range_label.len() + 1] = b'.';
        buf[range_label.len() + 2] = b'0' + frac as u8;
        draw_text(buf.as_ptr(), (range_label.len() + 3) as u32, 20.0, 75.0, 14.0, 0xFFFF66FF);
    }
}
```

### 18. Create examples/point-lights/Cargo.toml

```toml
[package]
name = "point-lights"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[profile.release]
opt-level = "s"
lto = true
```

### 19. Add to examples manifest (if exists)

If there's a workspace or examples registry, add the new example:
```
point-lights - Demonstrates point light positioning, range, and attenuation
```
