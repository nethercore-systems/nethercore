# Emberware Z Texture Compression Specification

**Status:** Proposal / Draft  
**Author:** Zerve  
**Last Updated:** December 2025

---

## Overview

Emberware Z texture format is determined by render mode. The build system detects mode by executing `init()` and compresses assets accordingly.

**Format rules:**
- Mode 0 (Unlit): RGBA8 — pixel-perfect, full alpha
- Mode 1-3 (Lit): BC7 — 4× compression, stipple transparency, correct material maps

---

## Render Mode = Texture Format

| Mode | Name | Texture Format | BPP | Features |
|------|------|----------------|-----|----------|
| 0 | Unlit | RGBA8 | 32 | Pixel art, full 256-level alpha |
| 1 | Matcap | BC7 | 8 | 4× compression, shader stipple |
| 2 | MRBP | BC7 | 8 | 4× compression, independent channels |
| 3 | SSBP | BC7 | 8 | 4× compression, independent channels |

**This is an aesthetic choice:**
- Mode 0: Pixel-perfect 3D (Minecraft-style, low-poly indie)
- Mode 1-3: Lit 3D with Saturn-style stipple transparency

---

## Why BC7?

| Format | BPP | RGB Channels | Alpha | Material Maps |
|--------|-----|--------------|-------|---------------|
| BC1 | 4 | Correlated | 1-bit cutout | ❌ Artifacts |
| BC2 | 8 | Correlated | 4-bit explicit | ❌ Artifacts |
| BC3 | 8 | Correlated | 8-bit smooth | ❌ Artifacts |
| **BC7** | **8** | **Independent** | **8-bit adaptive** | **✓ Correct** |

BC7 advantages:
- **Independent RGB channels** — material maps (M+R+E, S+D+E) encode correctly
- **Adaptive bit allocation** — encoder optimizes per block, no wasted bits
- **Same size as BC2/BC3** — no budget penalty vs inferior formats
- **Stipple-compatible** — smooth alpha → shader quantizes to 16 levels

---

## Texture Slot Usage

### Mode 0 (Unlit)

| Slot | Purpose | Format |
|------|---------|--------|
| 1 | Albedo | RGBA8 |
| 2-4 | — | Unused |

### Mode 1 (Matcap)

| Slot | Purpose | Format |
|------|---------|--------|
| 1 | Matcap | BC7 |
| 2 | Matcap | BC7 |
| 3 | Matcap | BC7 |
| 4 | Matcap | BC7 |

### Mode 2 (MRBP)

| Slot | Purpose | Format |
|------|---------|--------|
| 1 | Albedo | BC7 |
| 2 | Material (R=metallic, G=roughness, B=emissive) | BC7 |
| 3 | — | Unused |
| 4 | Environment | BC7 |

### Mode 3 (SSBP)

| Slot | Purpose | Format |
|------|---------|--------|
| 1 | Albedo | BC7 |
| 2 | Material (R=shininess, G=damping, B=emissive) | BC7 |
| 3 | Specular color | BC7 |
| 4 | Environment | BC7 |

---

## Transparency

Emberware Z uses stipple/screen-door transparency instead of alpha blending. See the **Transparency Specification** for full details on:

- Bayer ordered dithering (2×2, 4×4, 8×8, 16×16 patterns)
- Shader integration
- Alpha sources (texture alpha, uniform alpha)
- packedShadingState integration

**Key point:** BC7's 8-bit alpha is used as input to the stipple shader, which quantizes to the selected pattern's level count.

---

## Texture Data Layout (POD)

No magic bytes. No version field. Direct POD upload.

### RGBA8 (Mode 0)

```rust
#[repr(C)]
struct TextureHeader {
    width: u16,
    height: u16,
}

// Followed by: width × height × 4 bytes (RGBA order)
```

### BC7 (Mode 1-3)

```rust
#[repr(C)]
struct TextureHeader {
    width: u16,
    height: u16,
}

// Followed by: (width/4) × (height/4) × 16 bytes (BC7 blocks)
```

---

## BC7 Format Details

### Block Structure

Each 4×4 pixel block = 16 bytes.

BC7 has 8 encoding modes. The encoder selects optimal mode per block:

| Mode | Partitions | Color Bits | Alpha Bits | Best For |
|------|------------|------------|------------|----------|
| 0 | 3 | 4.4.4 | — | Complex color, no alpha |
| 1 | 2 | 6.6.6 | — | Moderate color variation |
| 2 | 3 | 5.5.5 | — | Simple color patterns |
| 3 | 2 | 7.7.7 | — | High color precision |
| 4 | 1 | 5.5.5 | 6 separate | Color + alpha |
| 5 | 1 | 7.7.7 | 8 | High quality RGBA |
| 6 | 1 | 7.7.7.7 | — | RGBA as single set |
| 7 | 2 | 5.5.5.5 | — | Two-subset RGBA |

The encoder picks the best mode automatically. Simple alpha (like constant 0.5 for stipple) uses fewer bits, leaving more for RGB quality.

### Size Comparison

| Resolution | RGBA8 | BC7 | Savings |
|------------|-------|-----|---------|
| 32×32 | 4 KB | 1 KB | 4× |
| 64×64 | 16 KB | 4 KB | 4× |
| 128×128 | 64 KB | 16 KB | 4× |
| 256×256 | 256 KB | 64 KB | 4× |

---

## Build-Time Mode Detection

The build system executes `init()` to determine texture format.

### Process

```
ember build
├── 1. Compile WASM
├── 2. Execute init() with stub FFI
│   ├── Capture set_render_mode(N)
│   ├── Capture set_resolution(W, H)
│   └── Capture all load_texture() calls
├── 3. Validate configuration
├── 4. Compress textures based on mode
│   ├── Mode 0 → RGBA8 (passthrough)
│   └── Mode 1-3 → BC7 (compress)
└── 5. Bundle WASM + assets → game.ewz
```

### Validation Rules

```rust
fn validate(ctx: &BuildContext) -> Result<(), BuildError> {
    // set_render_mode: 0 or 1 call
    if ctx.render_mode_calls > 1 {
        return Err("set_render_mode() called multiple times");
    }
    
    // set_resolution: 0 or 1 call
    if ctx.resolution_calls > 1 {
        return Err("set_resolution() called multiple times");
    }
    
    Ok(())
}
```

### Format Selection

```rust
fn texture_format(mode: Option<u32>) -> TextureFormat {
    match mode.unwrap_or(0) {
        0 => TextureFormat::Rgba8,
        1 | 2 | 3 => TextureFormat::Bc7,
        _ => panic!("Invalid render mode"),
    }
}
```

---

## BC7 Encoding (Rust)

### Recommended Crates

| Crate | Quality | Speed | Notes |
|-------|---------|-------|-------|
| `intel_tex_2` | High | Fast (ISPC) | Best quality, requires ISPC runtime |
| `bc7enc` | High | Medium | Pure Rust, no dependencies |
| `texpresso` | Good | Fast | Simple API |
| `image-dds` | Varies | Varies | Uses system encoder |

### Using `intel_tex_2` (Recommended)

```toml
# Cargo.toml
[build-dependencies]
intel_tex_2 = "0.2"
image = "0.24"
```

```rust
use intel_tex_2::{bc7, RgbaSurface};
use image::RgbaImage;

fn compress_bc7(img: &RgbaImage) -> Vec<u8> {
    let width = img.width() as usize;
    let height = img.height() as usize;
    let pixels = img.as_raw();
    
    let surface = RgbaSurface {
        width,
        height,
        stride: width * 4,
        data: pixels,
    };
    
    // High quality encoding (slow but best results)
    let settings = bc7::opaque_ultra_fast_settings();  // or slow_settings() for release
    
    bc7::compress_blocks(&settings, &surface)
}
```

### Using `bc7enc` (Pure Rust)

```toml
# Cargo.toml
[build-dependencies]
bc7enc = "0.1"
```

```rust
use bc7enc::{encode_bc7, EncodeSettings};

fn compress_bc7(rgba_pixels: &[u8], width: u32, height: u32) -> Vec<u8> {
    let settings = EncodeSettings::default();
    encode_bc7(rgba_pixels, width, height, &settings)
}
```

### Using `texpresso`

```toml
# Cargo.toml
[build-dependencies]
texpresso = "2.0"
```

```rust
use texpresso::Format;

fn compress_bc7(rgba_pixels: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut output = vec![0u8; Format::Bc7.compressed_size(width, height)];
    Format::Bc7.compress(rgba_pixels, width, height, texpresso::Params::default(), &mut output);
    output
}
```

### Build Integration Example

```rust
// build.rs or ember build tool
fn compress_texture(raw_png: &[u8], mode: RenderMode) -> Vec<u8> {
    let img = image::load_from_memory(raw_png)
        .expect("Invalid PNG")
        .to_rgba8();
    
    let (width, height) = (img.width() as u16, img.height() as u16);
    
    let mut output = Vec::new();
    
    // Write header
    output.extend_from_slice(&width.to_le_bytes());
    output.extend_from_slice(&height.to_le_bytes());
    
    // Write pixel data
    match mode {
        RenderMode::Unlit => {
            // RGBA8: raw pixels
            output.extend_from_slice(img.as_raw());
        }
        _ => {
            // BC7: compressed
            let bc7_data = compress_bc7(&img);
            output.extend_from_slice(&bc7_data);
        }
    }
    
    output
}
```

---

## sRGB vs Linear

| Texture Type | Color Space | wgpu Format |
|--------------|-------------|-------------|
| Albedo | sRGB | `Rgba8UnormSrgb` / `Bc7RgbaUnormSrgb` |
| Matcap | sRGB | `Bc7RgbaUnormSrgb` |
| Specular color | sRGB | `Bc7RgbaUnormSrgb` |
| Material (M+R+E) | Linear | `Bc7RgbaUnorm` |
| Environment | sRGB | `Bc7RgbaUnormSrgb` |

Material properties are numeric data, not colors—store as linear.

---

## wgpu Integration

### Texture Creation

```rust
fn create_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    mode: RenderMode,
    is_material: bool,
) -> wgpu::Texture {
    let format = match mode {
        RenderMode::Unlit => {
            wgpu::TextureFormat::Rgba8UnormSrgb
        }
        _ => {
            if is_material {
                wgpu::TextureFormat::Bc7RgbaUnorm  // Linear
            } else {
                wgpu::TextureFormat::Bc7RgbaUnormSrgb  // sRGB
            }
        }
    };
    
    device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}
```

### Texture Upload

```rust
fn upload_texture(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    data: &[u8],
    width: u32,
    height: u32,
    is_bc7: bool,
) {
    let bytes_per_row = if is_bc7 {
        // BC7: 16 bytes per 4×4 block
        (width / 4) * 16
    } else {
        // RGBA8: 4 bytes per pixel
        width * 4
    };
    
    let rows = if is_bc7 { height / 4 } else { height };
    
    queue.write_texture(
        texture.as_image_copy(),
        data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(bytes_per_row),
            rows_per_image: Some(rows),
        },
        wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
    );
}
```

---

## Procedural Textures

Runtime-created textures are always RGBA8 (any mode):

```rust
let handle = create_texture(64, 64);
update_texture(handle, pixel_data.as_ptr());
texture_bind(0, handle);
```

Procedural textures bypass compression. Use for:
- UI elements
- Dynamic effects
- Small generated content

---

## Budget Examples

### Fighting Game (SSBP, 10 Characters, 4 Stages)

**Mode 0 (Unlit pixel art):**
```
10 characters × 64×64 RGBA8 = 160 KB
4 stages × 128×128 RGBA8 = 256 KB
UI = 64 KB
───────────────────────────────────
Total: 480 KB
```

**Mode 3 (SSBP with BC7):**
```
10 characters × 3 slots × 128×128 BC7 = 480 KB
4 stages × 3 slots × 128×128 BC7 = 192 KB
Environment = 64 KB
UI (procedural RGBA8) = 64 KB
───────────────────────────────────────────
Total: 800 KB
```

Both fit easily in 16 MB ROM.

---

## Pixel Art on Mode 0

Mode 0 with RGBA8 preserves pixel-perfect textures:

- Use nearest-neighbor filtering (`FilterMode::Nearest`)
- Every pixel exactly as authored
- Full 256-level alpha for smooth transparency
- Or use 0/255 alpha for manual stipple patterns

For stipple in Mode 0, artists must paint the pattern themselves (shader doesn't auto-stipple RGBA8).

---

## Open Questions

1. **Default mode** — Should missing `set_render_mode()` default to mode 0 or error?

2. **Encoder quality settings** — Fast for debug builds, high quality for release?

3. **Mipmap generation** — Generate at build time, load time, or not at all?

4. **Stipple as separate spec** — Should stipple/transparency warrant its own spec document?

---

## References

- [BC7 Format (Microsoft)](https://learn.microsoft.com/en-us/windows/win32/direct3d11/bc7-format)
- [intel_tex_2 crate](https://crates.io/crates/intel_tex_2)
- [bc7enc crate](https://crates.io/crates/bc7enc)
- [texpresso crate](https://crates.io/crates/texpresso)
- [Bayer Dithering](https://en.wikipedia.org/wiki/Ordered_dithering)
- [wgpu TextureFormat](https://docs.rs/wgpu/latest/wgpu/enum.TextureFormat.html)
