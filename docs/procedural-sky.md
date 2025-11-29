# Procedural Sky System - Implementation Reference

## Overview

A simple, performant procedural sky system using hemispherical gradient + analytical sun. Used for:
1. Background rendering (skybox)
2. Environment reflections (metallic surfaces)
3. Specular highlights (glossy surfaces)
4. Diffuse ambient lighting (IBL approximation)

## Core Algorithm

Sample the sky in any direction to get a color:

```
sky_gradient = lerp(horizon_color, zenith_color, direction.y * 0.5 + 0.5)
sun_amount = max(0, dot(direction, sun_direction))
sun_contribution = sun_color * pow(sun_amount, sun_sharpness)
final_color = sky_gradient + sun_contribution
```

## WGSL Implementation

### Uniforms

```wgsl
struct SkyUniforms {
    horizon_color: vec3<f32>,
    zenith_color: vec3<f32>,
    sun_direction: vec3<f32>,
    sun_sharpness: f32,
    sun_color: vec3<f32>,
    _pad: f32,
}

@group(0) @binding(0) var<uniform> sky: SkyUniforms;
```

### Core Function

```wgsl
fn sample_sky(direction: vec3<f32>) -> vec3<f32> {
    // Gradient based on vertical component
    let up_factor = direction.y * 0.5 + 0.5;
    let sky_color = mix(sky.horizon_color, sky.zenith_color, up_factor);
    
    // Analytical sun
    let sun_dot = max(0.0, dot(direction, sky.sun_direction));
    let sun_contribution = sky.sun_color * pow(sun_dot, sky.sun_sharpness);
    
    return sky_color + sun_contribution;
}
```

## Use Case 1: Background Rendering

Render fullscreen quad as first pass with depth = 1.0 (far plane).

```wgsl
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) view_ray: vec3<f32>,
}

@vertex
fn vs_skybox(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Fullscreen triangle
    let x = f32((vertex_index << 1u) & 2u) * 2.0 - 1.0;
    let y = f32(vertex_index & 2u) * 2.0 - 1.0;
    
    var output: VertexOutput;
    output.position = vec4<f32>(x, y, 1.0, 1.0); // At far plane
    
    // Calculate view ray from inverse projection
    let clip_pos = vec4<f32>(x, y, 1.0, 1.0);
    let view_pos = inverse_projection * clip_pos;
    output.view_ray = (inverse_view * vec4<f32>(view_pos.xyz, 0.0)).xyz;
    
    return output;
}

@fragment
fn fs_skybox(in: VertexOutput) -> @location(0) vec4<f32> {
    let direction = normalize(in.view_ray);
    let color = sample_sky(direction);
    return vec4<f32>(color, 1.0);
}
```

## Use Case 2: Environment Reflections (Metallic)

For metallic/mirror-like surfaces, reflect view direction and sample sky.

**Important:** Reflections are always sharp. Roughness does NOT affect reflection sharpness (no mipmaps/blurring).

```wgsl
// In your PBR fragment shader
let view_dir = normalize(camera_pos - world_pos);
let reflection_dir = reflect(-view_dir, normal);
let env_reflection = sample_sky(reflection_dir);

// Apply based on metallic factor only
let reflection_strength = material.metallic;
final_color += env_reflection * reflection_strength;
```

## Use Case 3: Specular Highlights

Combine direct light specular with environment specular.

**Roughness behavior:** Only affects specular highlight size (shininess), NOT reflection sharpness.

```wgsl
// Direct specular from directional light
let light_dir = sky.sun_direction; // Same as sky sun direction
let half_dir = normalize(view_dir + light_dir);
let ndoth = max(dot(normal, half_dir), 0.0);

// Roughness controls shininess
let shininess = mix(4.0, 128.0, 1.0 - material.roughness);
let direct_spec = pow(ndoth, shininess) * sky.sun_color;

// Environment specular (always sharp sample)
let reflection_dir = reflect(-view_dir, normal);
let env_spec = sample_sky(reflection_dir);

// Combine
let total_specular = (direct_spec + env_spec) * material.specular_strength;
```

## Use Case 4: Diffuse Ambient (IBL Approximation)

Sample sky in the **normal direction** for diffuse ambient/IBL contribution.

**Important:** Diffuse samples at the surface normal (N), NOT the reflection direction.

```wgsl
// Sample sky at surface normal (N) for diffuse ambient
let ambient_sky = sample_sky(normal);

// Apply as ambient term
let ambient = ambient_sky * material.ambient_strength;
final_color += base_color * ambient;
```

**Why normal vs reflection:**
- **Diffuse (ambient):** Uses `sample_sky(N)` - represents light coming from the hemisphere around the surface
- **Specular/Reflections:** Uses `sample_sky(R)` - represents light reflecting off the surface

## Complete PBR-Lite Fragment Shader

```wgsl
struct FragmentInput {
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct Material {
    base_color: vec3<f32>,
    metallic: f32,           // 0 = dielectric, 1 = metal
    roughness: f32,          // Only affects specular highlight size
    ambient_strength: f32,   // Ambient light intensity (0.0-1.0 typical)
    specular_strength: f32,  // Specular intensity (0.0-1.0 typical)
}

@fragment
fn fs_main(in: FragmentInput) -> @location(0) vec4<f32> {
    let N = normalize(in.normal);
    let V = normalize(camera_pos - in.world_pos);
    let L = sky.sun_direction;
    let H = normalize(V + L);
    let R = reflect(-V, N);  // Reflection direction
    
    let base_color = material.base_color;
    
    // 1. DIFFUSE (Lambert + ambient from sky)
    let ndotl = max(dot(N, L), 0.0);
    let direct_diffuse = base_color * sky.sun_color * ndotl;
    
    let ambient_sky = sample_sky(N);  // Sample at NORMAL direction
    let ambient_diffuse = base_color * ambient_sky * material.ambient_strength;
    
    let total_diffuse = direct_diffuse + ambient_diffuse;
    
    // 2. SPECULAR (roughness affects shininess only)
    let ndoth = max(dot(N, H), 0.0);
    let shininess = mix(4.0, 128.0, 1.0 - material.roughness);
    let direct_spec = pow(ndoth, shininess) * sky.sun_color;
    
    let env_spec = sample_sky(R);  // Sample at REFLECTION direction
    
    let total_specular = (direct_spec + env_spec) * material.specular_strength;
    
    // 3. REFLECTIONS (metallic surfaces - always sharp)
    let env_reflection = sample_sky(R);  // Sample at REFLECTION direction
    
    // 4. COMBINE
    // Metals have no diffuse, dielectrics have no reflections
    let final_color = total_diffuse * (1.0 - material.metallic) 
                    + total_specular 
                    + env_reflection * material.metallic;
    
    return vec4<f32>(final_color, 1.0);
}
```

## Sky Sampling Directions Summary

The sky is sampled in different directions for different lighting components:

| Component | Direction | Variable | Why |
|-----------|-----------|----------|-----|
| **Diffuse Ambient** | Surface Normal | `sample_sky(N)` | Light coming from hemisphere around surface |
| **Specular Highlight** | Reflection | `sample_sky(R)` | Mirror reflection of environment |
| **Metallic Reflection** | Reflection | `sample_sky(R)` | Mirror reflection of environment |

Where:
- `N = normalize(normal)` - Surface normal
- `R = reflect(-V, N)` - Reflection of view direction about normal
- `V = normalize(camera_pos - world_pos)` - View direction

## Roughness Behavior Clarification

**What roughness DOES affect:**
- Specular highlight size (via shininess calculation)
- Smaller highlights on rough surfaces, sharper on smooth surfaces

**What roughness DOES NOT affect:**
- Reflection sharpness (reflections are always sharp)
- Reflection brightness (controlled by metallic only)

**Why:** Proper rough reflections require mipmaps or multi-sampling. The procedural sky has no mipmaps, so reflections are always sharp samples. This is era-appropriate for PS1/N64 aesthetics.

**Alternative (if needed):** Rough surfaces can simply fade reflection intensity:
```wgsl
let reflection_strength = material.metallic * (1.0 - material.roughness);
```
This dims rough reflections rather than blurring them.

## FFI API Design

```rust
// Set sky parameters (call once per frame or when sky changes)
pub fn set_sky(
    horizon_color: [f32; 3],
    zenith_color: [f32; 3],
    sun_direction: [f32; 3],  // Normalized
    sun_color: [f32; 3],      // Can exceed 1.0 for brightness
    sun_sharpness: f32,       // 10.0-1000.0 typical
)
```

**Default state:** All zeros (black sky, no sun, no lighting). Must call `set_sky()` in `init()` to enable any sky-based lighting.

```rust
// Internally used for:
// - Skybox background rendering
// - Environment reflections (matcap/PBR modes)
// - Specular highlights
// - Diffuse ambient lighting
```

## Synchronization with Directional Light

**Recommended:** Use the same `sun_direction` for both sky sun and directional light to maintain visual consistency.

```rust
// Single source of truth
let sun_dir = Vec3::new(0.3, 0.8, 0.5).normalize();

set_sky(horizon, zenith, sun_dir, sun_color, sharpness);
set_directional_light(sun_dir, sun_color);
```

## Example Presets

```rust
// Sunset
horizon: [1.0, 0.5, 0.3]
zenith:  [0.3, 0.1, 0.5]
sun_color: [3.0, 1.8, 0.9]  // Bright warm sun (values > 1.0 for intensity)
sharpness: 100.0

// Midday
horizon: [0.7, 0.8, 0.9]
zenith:  [0.3, 0.5, 0.9]
sun_color: [2.0, 1.9, 1.8]  // Bright neutral sun
sharpness: 200.0

// Alien World
horizon: [0.8, 0.3, 0.8]
zenith:  [0.1, 0.0, 0.2]
sun_color: [1.2, 4.0, 2.0]  // Intense green-tinted sun
sharpness: 150.0

// Overcast (no sun)
horizon: [0.7, 0.7, 0.7]
zenith:  [0.5, 0.5, 0.5]
sun_color: [0.0, 0.0, 0.0]  // No sun visible
sharpness: 0.0
```

**Note:** Sun colors can and should exceed 1.0 to create visible bright sun discs against the sky gradient.

## Material Parameter Guidelines

```rust
// Typical material values:

// Matte dielectric (plastic, painted metal)
metallic: 0.0
roughness: 0.8
ambient_strength: 0.3
specular_strength: 0.5

// Glossy dielectric (polished wood, ceramic)
metallic: 0.0
roughness: 0.2
ambient_strength: 0.3
specular_strength: 0.8

// Raw metal (iron, steel)
metallic: 1.0
roughness: 0.6
ambient_strength: 0.2
specular_strength: 0.7

// Polished metal (chrome, mirror)
metallic: 1.0
roughness: 0.1
ambient_strength: 0.1
specular_strength: 1.0
```

## Performance Notes

- `sample_sky()` is ~6 operations: 1 mix, 1 dot, 1 max, 1 pow, 1 multiply, 1 add
- Can be called multiple times per pixel (diffuse + specular + reflection) with negligible cost
- No texture lookups, no branching
- Ideal for Emberware's performance constraints
- Total cost per pixel: ~18 operations for full lighting (3x sky samples)