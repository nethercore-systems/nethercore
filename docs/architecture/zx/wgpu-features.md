# wgpu Features for Desktop Backends

> Status: Reference
> Last reviewed: 2026-01-06

This document lists wgpu features commonly available on desktop backends (D3D12, Metal, Vulkan). These are **not enabled by default** but are safe to request on desktop platforms.

## Currently Enabled

| Feature | Purpose | Status |
|---------|---------|--------|
| `TEXTURE_COMPRESSION_BC` | BC1-7 compressed textures (DXT/BCn) | Enabled in `nethercore-zx/src/graphics/init.rs` |

## Recommended for Desktop

These features are universally supported on modern desktop GPUs and worth enabling:

### Texture Compression

| Feature | Description | D3D12 | Metal | Vulkan |
|---------|-------------|-------|-------|--------|
| `TEXTURE_COMPRESSION_BC` | BC1-7 block compression (DXT1/3/5, BC4-7). Standard for desktop textures. | ✅ | ✅ | ✅ |

### Rendering Features

| Feature | Description | D3D12 | Metal | Vulkan |
|---------|-------------|-------|-------|--------|
| `POLYGON_MODE_LINE` | Wireframe rendering (`PolygonMode::Line`) | ✅ | ✅ | ✅ |
| `POLYGON_MODE_POINT` | Point rendering (`PolygonMode::Point`) | ✅ | ✅ | ✅ |
| `DEPTH_CLIP_CONTROL` | Disable depth clipping (useful for shadow maps) | ✅ | ✅ | ✅ |
| `CONSERVATIVE_RASTERIZATION` | Conservative rasterization for voxelization/occlusion | ✅ | ⚠️ | ✅ |

### Sampler Features

| Feature | Description | D3D12 | Metal | Vulkan |
|---------|-------------|-------|-------|--------|
| `ADDRESS_MODE_CLAMP_TO_BORDER` | Sampler border clamp mode with custom border color | ✅ | ✅ | ✅ |
| `ADDRESS_MODE_CLAMP_TO_ZERO` | Clamp to transparent black (0,0,0,0) | ✅ | ✅ | ✅ |
| `SAMPLER_ANISOTROPY` | Anisotropic filtering (usually enabled by default via limits) | ✅ | ✅ | ✅ |

### Buffer/Binding Features

| Feature | Description | D3D12 | Metal | Vulkan |
|---------|-------------|-------|-------|--------|
| `PUSH_CONSTANTS` | Small uniforms without bind groups (up to 128 bytes) | ✅ | ✅ | ✅ |
| `MULTI_DRAW_INDIRECT` | Multiple indirect draws from one buffer | ✅ | ✅ | ✅ |
| `MULTI_DRAW_INDIRECT_COUNT` | Indirect draw count from GPU buffer | ✅ | ⚠️ | ✅ |
| `VERTEX_WRITABLE_STORAGE` | Write to storage buffers from vertex shaders | ✅ | ✅ | ✅ |

### Texture Features

| Feature | Description | D3D12 | Metal | Vulkan |
|---------|-------------|-------|-------|--------|
| `TEXTURE_FORMAT_16BIT_NORM` | R16/RG16/RGBA16 Unorm/Snorm formats | ✅ | ✅ | ✅ |
| `TEXTURE_COMPRESSION_ETC2` | ETC2/EAC compression (mobile-oriented, but supported) | ❌ | ✅ | ⚠️ |
| `TEXTURE_COMPRESSION_ASTC` | ASTC compression (mobile-oriented) | ❌ | ✅ | ⚠️ |
| `RG11B10UFLOAT_RENDERABLE` | Render to RG11B10Float format (HDR) | ✅ | ✅ | ✅ |
| `BGRA8UNORM_STORAGE` | Use BGRA8 as storage texture | ✅ | ✅ | ✅ |
| `FLOAT32_FILTERABLE` | Linear filtering on R32Float/RG32Float/RGBA32Float | ✅ | ✅ | ✅ |

### Shader Features

| Feature | Description | D3D12 | Metal | Vulkan |
|---------|-------------|-------|-------|--------|
| `SHADER_F16` | 16-bit float in shaders (half precision) | ✅ | ✅ | ✅ |
| `SHADER_I16` | 16-bit signed integers in shaders | ⚠️ | ✅ | ✅ |
| `SHADER_F64` | 64-bit floats in shaders (double precision) | ✅ | ❌ | ✅ |
| `SHADER_EARLY_DEPTH_TEST` | Force early depth test in fragment shader | ✅ | ✅ | ✅ |
| `DUAL_SOURCE_BLENDING` | Two outputs from fragment shader for blending | ✅ | ✅ | ✅ |

### Timestamp/Query Features

| Feature | Description | D3D12 | Metal | Vulkan |
|---------|-------------|-------|-------|--------|
| `TIMESTAMP_QUERY` | GPU timestamp queries for profiling | ✅ | ✅ | ✅ |
| `TIMESTAMP_QUERY_INSIDE_ENCODERS` | Timestamps inside render/compute passes | ✅ | ⚠️ | ✅ |
| `TIMESTAMP_QUERY_INSIDE_PASSES` | Timestamps inside passes (more granular) | ✅ | ⚠️ | ✅ |
| `PIPELINE_STATISTICS_QUERY` | Pipeline statistics (vertices, primitives, etc.) | ✅ | ❌ | ✅ |

### Indirect Rendering

| Feature | Description | D3D12 | Metal | Vulkan |
|---------|-------------|-------|-------|--------|
| `INDIRECT_FIRST_INSTANCE` | `first_instance` in indirect draw calls | ✅ | ✅ | ✅ |
| `MULTI_DRAW_INDIRECT` | Batch multiple draws in one call | ✅ | ✅ | ✅ |
| `MULTI_DRAW_INDIRECT_COUNT` | GPU-driven draw count | ✅ | ⚠️ | ✅ |

## Legend

- ✅ = Universally supported
- ⚠️ = Supported on most hardware but check adapter
- ❌ = Not supported on this backend

## Usage Example

```rust
// Query adapter for available features
let adapter_features = adapter.features();

// Build feature set for desktop
let mut required_features = wgpu::Features::empty();

// Texture compression (required for BC7 textures)
if adapter_features.contains(wgpu::Features::TEXTURE_COMPRESSION_BC) {
    required_features |= wgpu::Features::TEXTURE_COMPRESSION_BC;
}

// Wireframe debugging
if adapter_features.contains(wgpu::Features::POLYGON_MODE_LINE) {
    required_features |= wgpu::Features::POLYGON_MODE_LINE;
}

// Push constants for small uniforms
if adapter_features.contains(wgpu::Features::PUSH_CONSTANTS) {
    required_features |= wgpu::Features::PUSH_CONSTANTS;
}

// GPU profiling
if adapter_features.contains(wgpu::Features::TIMESTAMP_QUERY) {
    required_features |= wgpu::Features::TIMESTAMP_QUERY;
}

let (device, queue) = adapter
    .request_device(&wgpu::DeviceDescriptor {
        required_features,
        ..Default::default()
    })
    .await?;
```

## Recommended Desktop Feature Set

For Nethercore ZX, consider enabling these unconditionally on desktop:

```rust
const DESKTOP_FEATURES: wgpu::Features = wgpu::Features::TEXTURE_COMPRESSION_BC
    .union(wgpu::Features::POLYGON_MODE_LINE)
    .union(wgpu::Features::DEPTH_CLIP_CONTROL)
    .union(wgpu::Features::PUSH_CONSTANTS)
    .union(wgpu::Features::MULTI_DRAW_INDIRECT)
    .union(wgpu::Features::INDIRECT_FIRST_INSTANCE)
    .union(wgpu::Features::TIMESTAMP_QUERY)
    .union(wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER);
```

Then intersect with adapter capabilities:

```rust
let required_features = DESKTOP_FEATURES & adapter.features();
```

## Notes

1. **WebGPU vs Native**: Many features listed here are wgpu extensions beyond the WebGPU spec. They work on native backends but won't work if targeting WebGPU in browsers.

2. **Feature Detection**: Always check `adapter.features()` before requesting. Requesting unsupported features will cause device creation to fail.

3. **Mobile**: If you ever target mobile (Android/iOS), you'll need ASTC/ETC2 instead of BC compression, and many features above won't be available.

4. **Limits**: Some features also require specific limits (e.g., `max_push_constant_size` for push constants). Check `adapter.limits()` as well.

## References

- [wgpu Features documentation](https://docs.rs/wgpu/latest/wgpu/struct.Features.html)
- [WebGPU Feature Levels](https://www.w3.org/TR/webgpu/#feature-index)
