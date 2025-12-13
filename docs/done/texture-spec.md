# Emberware Z Texture Compression Specification

**Status:** Implementation Spec (ready for implementation)
**Author:** Zerve
**Last Updated:** December 2025

---

## Overview

Emberware Z texture format is determined by render mode. The build system detects mode by executing `init()` with stub FFI and compresses assets accordingly.

**Format Rules:**
| Mode | Name | Texture Format | BPP | Features |
|------|------|----------------|-----|----------|
| 0 | Unlit | RGBA8 | 32 | Pixel-perfect, full 256-level alpha |
| 1 | Matcap | BC7 | 8 | 4× compression, stipple transparency |
| 2 | MRBP | BC7 | 8 | 4× compression, independent channels |
| 3 | SSBP | BC7 | 8 | 4× compression, independent channels |

**This is an aesthetic choice:**
- Mode 0: Pixel-perfect 3D (Minecraft-style, low-poly indie)
- Mode 1-3: Lit 3D with Saturn-style stipple transparency

---

## Why BC7?

| Format | BPP | RGB Channels | Alpha | Material Maps |
|--------|-----|--------------|-------|---------------|
| BC1 | 4 | Correlated | 1-bit cutout | Artifacts |
| BC2 | 8 | Correlated | 4-bit explicit | Artifacts |
| BC3 | 8 | Correlated | 8-bit smooth | Artifacts |
| **BC7** | **8** | **Independent** | **8-bit adaptive** | **Correct** |

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
| 0 | Albedo | RGBA8 |
| 1-3 | — | Unused |

### Mode 1 (Matcap)

| Slot | Purpose | Format |
|------|---------|--------|
| 0-3 | Matcap layers | BC7 |

### Mode 2 (MRBP - Metallic-Roughness-Blinn-Phong)

| Slot | Purpose | Format | Color Space |
|------|---------|--------|-------------|
| 0 | Albedo | BC7 | sRGB |
| 1 | Material (R=metallic, G=roughness, B=emissive) | BC7 | **Linear** |
| 2 | — | Unused | — |
| 3 | Environment | BC7 | sRGB |

### Mode 3 (SSBP - Specular-Shininess-Blinn-Phong)

| Slot | Purpose | Format | Color Space |
|------|---------|--------|-------------|
| 0 | Albedo | BC7 | sRGB |
| 1 | Material (R=shininess, G=damping, B=emissive) | BC7 | **Linear** |
| 2 | Specular color | BC7 | sRGB |
| 3 | Environment | BC7 | sRGB |

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

### Header (both formats)

```rust
#[repr(C)]
struct TextureHeader {
    width: u16,   // Max 65535, convert to u32 at upload time
    height: u16,
}
```

**Size:** 4 bytes

### RGBA8 (Mode 0)

```
[Header: 4 bytes]
[Pixels: width × height × 4 bytes, RGBA order]
```

**Total:** 4 + (W × H × 4) bytes

### BC7 (Mode 1-3)

```
[Header: 4 bytes]
[Blocks: (width/4) × (height/4) × 16 bytes]
```

**Total:** 4 + ((W/4) × (H/4) × 16) bytes

**Constraint:** BC7 dimensions must be multiples of 4. Build tool pads if needed (edge-clamp).

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

The build system executes `init()` with stub FFI to auto-detect configuration.

### Process

```
ember build
├── 1. Compile WASM (cargo build --target wasm32-unknown-unknown)
├── 2. Execute init() with StubFFIState
│   ├── Capture render_mode(N) call → mode
│   ├── Capture set_resolution(W, H) call → resolution
│   └── Capture all load_texture() / rom_texture() calls → texture list
├── 3. Validate configuration
├── 4. Compress textures based on detected mode
│   ├── Mode 0 → RGBA8 (passthrough, resize header to u16)
│   └── Mode 1-3 → BC7 (compress with texpresso)
└── 5. Bundle WASM + assets → game.ewz
```

### StubFFIState Design

```rust
// core/src/analysis/stub_state.rs

/// Minimal state for build-time analysis - captures init() calls without GPU
pub struct StubFFIState {
    pub render_mode: Option<u8>,
    pub resolution: Option<(u32, u32)>,
    pub tick_rate: Option<u32>,
    pub texture_requests: Vec<TextureRequest>,
    pub mesh_requests: Vec<MeshRequest>,
}

pub struct TextureRequest {
    pub handle: u32,
    pub width: u32,
    pub height: u32,
    pub source: TextureSource,
}

pub enum TextureSource {
    WasmMemory { ptr: u32 },      // load_texture()
    RomPack { id: String },       // rom_texture()
}
```

### Stub FFI Registration

```rust
// core/src/analysis/stub_ffi.rs

pub fn register_stub_ffi(
    linker: &mut Linker<GameStateWithConsole<ZInput, StubFFIState>>
) -> Result<()> {
    // Config captures
    linker.func_wrap("env", "render_mode", |mut caller: Caller<'_, _>, mode: u32| {
        caller.data_mut().console.render_mode = Some(mode as u8);
    })?;

    linker.func_wrap("env", "set_resolution", |mut caller: Caller<'_, _>, w: u32, h: u32| {
        caller.data_mut().console.resolution = Some((w, h));
    })?;

    // Texture capture (returns fake handle, doesn't load pixels)
    linker.func_wrap("env", "load_texture",
        |mut caller: Caller<'_, _>, w: u32, h: u32, _ptr: u32| -> u32 {
            let state = &mut caller.data_mut().console;
            let handle = state.texture_requests.len() as u32 + 1;
            state.texture_requests.push(TextureRequest {
                handle,
                width: w,
                height: h,
                source: TextureSource::WasmMemory { ptr: _ptr },
            });
            handle
        }
    )?;

    // ROM texture capture
    linker.func_wrap("env", "rom_texture",
        |mut caller: Caller<'_, _>, id_ptr: u32, id_len: u32| -> u32 {
            let memory = caller.data().game.memory.unwrap();
            let id = read_string(&memory, &caller, id_ptr, id_len);
            let state = &mut caller.data_mut().console;
            let handle = state.texture_requests.len() as u32 + 1;
            state.texture_requests.push(TextureRequest {
                handle,
                width: 0,  // Unknown until ROM parsed
                height: 0,
                source: TextureSource::RomPack { id },
            });
            handle
        }
    )?;

    // All other FFI functions: no-op stubs
    register_noop_stubs(linker)?;

    Ok(())
}
```

### Analysis Execution

```rust
// core/src/analysis/mod.rs

pub struct AnalysisResult {
    pub render_mode: u8,
    pub resolution: Option<(u32, u32)>,
    pub texture_ids: Vec<String>,  // ROM texture IDs requested
}

pub fn analyze_wasm(wasm_bytes: &[u8]) -> Result<AnalysisResult> {
    let engine = WasmEngine::new()?;
    let module = engine.load_module(wasm_bytes)?;

    let mut linker = Linker::new(engine.engine());
    register_stub_ffi(&mut linker)?;

    let mut game = GameInstance::<ZInput, StubFFIState>::new(
        &engine, &module, &linker
    )?;

    game.init()?;

    let state = game.console_state();
    Ok(AnalysisResult {
        render_mode: state.render_mode.unwrap_or(0),
        resolution: state.resolution,
        texture_ids: state.texture_requests.iter()
            .filter_map(|r| match &r.source {
                TextureSource::RomPack { id } => Some(id.clone()),
                _ => None,
            })
            .collect(),
    })
}
```

### Validation Rules

```rust
fn validate(result: &AnalysisResult) -> Result<(), BuildError> {
    // render_mode must be 0-3
    if result.render_mode > 3 {
        return Err(BuildError::InvalidRenderMode(result.render_mode));
    }

    // No other validation needed - single calls are enforced by FFI
    Ok(())
}
```

---

## BC7 Encoding (Rust)

### Recommended Crate: texpresso

| Crate | License | Quality | Speed | Dependencies |
|-------|---------|---------|-------|--------------|
| **`texpresso`** | **MIT** | Good | Fast | **Pure Rust** |
| `intel_tex_2` | MIT/Apache | High | Fast | ISPC binaries |
| `bc7enc` | MIT | High | Medium | Pure Rust |

**Why texpresso:**
- Pure Rust (no ISPC binaries, no static linking issues)
- MIT license
- ~3K lines of code, minimal dependencies
- Supports BC7 with good quality

### Cargo.toml

```toml
# tools/ember-export/Cargo.toml
[dependencies]
texpresso = "2.0"
image = "0.24"
```

### Compression Implementation

```rust
// tools/ember-export/src/texture.rs

use texpresso::Format;
use image::RgbaImage;

pub fn compress_texture(
    img: &RgbaImage,
    mode: u8,
    slot: u8,
) -> Vec<u8> {
    let width = img.width() as u16;
    let height = img.height() as u16;
    let pixels = img.as_raw();

    let mut output = Vec::new();

    // Write u16 header
    output.extend_from_slice(&width.to_le_bytes());
    output.extend_from_slice(&height.to_le_bytes());

    match mode {
        0 => {
            // Mode 0: RGBA8 passthrough
            output.extend_from_slice(pixels);
        }
        1 | 2 | 3 => {
            // Mode 1-3: BC7 compression
            let w = width as usize;
            let h = height as usize;

            // Pad to multiple of 4 if needed
            let (padded_w, padded_h, padded_pixels) = pad_to_block_size(pixels, w, h);

            let compressed_size = Format::Bc7.compressed_size(padded_w, padded_h);
            let mut bc7_data = vec![0u8; compressed_size];

            Format::Bc7.compress(
                &padded_pixels,
                padded_w,
                padded_h,
                texpresso::Params::default(),
                &mut bc7_data,
            );

            output.extend_from_slice(&bc7_data);
        }
        _ => panic!("Invalid render mode: {}", mode),
    }

    output
}

fn pad_to_block_size(pixels: &[u8], w: usize, h: usize) -> (usize, usize, Vec<u8>) {
    let padded_w = (w + 3) & !3;  // Round up to multiple of 4
    let padded_h = (h + 3) & !3;

    if padded_w == w && padded_h == h {
        return (w, h, pixels.to_vec());
    }

    // Create padded image (edge-clamp)
    let mut padded = vec![0u8; padded_w * padded_h * 4];
    for y in 0..padded_h {
        let src_y = y.min(h - 1);
        for x in 0..padded_w {
            let src_x = x.min(w - 1);
            let src_idx = (src_y * w + src_x) * 4;
            let dst_idx = (y * padded_w + x) * 4;
            padded[dst_idx..dst_idx + 4].copy_from_slice(&pixels[src_idx..src_idx + 4]);
        }
    }

    (padded_w, padded_h, padded)
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

### Format Selection

```rust
// emberware-z/src/graphics/texture_manager.rs

fn texture_format(mode: u8, slot: u8, is_bc7: bool) -> wgpu::TextureFormat {
    if !is_bc7 {
        return wgpu::TextureFormat::Rgba8UnormSrgb;
    }

    // BC7 format depends on slot (material maps are linear)
    let is_material_slot = slot == 1 && (mode == 2 || mode == 3);

    if is_material_slot {
        wgpu::TextureFormat::Bc7RgbaUnorm  // Linear for material data
    } else {
        wgpu::TextureFormat::Bc7RgbaUnormSrgb  // sRGB for color
    }
}
```

### Texture Creation

```rust
fn create_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    mode: u8,
    slot: u8,
    is_bc7: bool,
) -> wgpu::Texture {
    let format = texture_format(mode, slot, is_bc7);

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
    let (bytes_per_row, rows) = if is_bc7 {
        // BC7: 16 bytes per 4×4 block
        ((width / 4) * 16, height / 4)
    } else {
        // RGBA8: 4 bytes per pixel
        (width * 4, height)
    };

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

## PackedTexture Format

```rust
// z-common/src/formats/z_data_pack.rs

#[derive(Serialize, Deserialize, Clone)]
pub struct PackedTexture {
    pub id: String,
    pub width: u16,               // Max 65535
    pub height: u16,              // Max 65535
    pub format: TextureFormat,    // Determines data interpretation
    pub data: Vec<u8>,            // RGBA8 pixels or BC7 blocks
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextureFormat {
    #[default]
    Rgba8,
    Bc7,
    Bc7Linear,  // For material maps (slot 1 in modes 2-3)
}
```

---

## Procedural Textures

Runtime-created textures are always RGBA8 regardless of mode:

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

## Files to Modify

### New Files

| Path | Purpose |
|------|---------|
| `core/src/analysis/mod.rs` | Analysis module entry |
| `core/src/analysis/stub_state.rs` | StubFFIState definition |
| `core/src/analysis/stub_ffi.rs` | Stub FFI registration |

### Modified Files

| Path | Changes |
|------|---------|
| `tools/ember-export/Cargo.toml` | Add `texpresso = "2.0"` |
| `tools/ember-export/src/texture.rs` | BC7 compression path |
| `tools/ember-cli/src/build.rs` | Call analysis before pack |
| `z-common/src/formats/texture.rs` | u16 header, format enum |
| `z-common/src/formats/z_data_pack.rs` | TextureFormat field |
| `emberware-z/src/graphics/texture_manager.rs` | BC7 upload, format selection |
| `emberware-z/src/ffi/rom.rs` | Handle BC7 textures from ROM |

---

## Migration Path

1. **Phase 1:** Update header to u16, keep RGBA8 only (non-breaking)
2. **Phase 2:** Add TextureFormat enum to PackedTexture (backward compatible with default)
3. **Phase 3:** Implement build-time analysis (new capability)
4. **Phase 4:** Add BC7 compression in ember-export
5. **Phase 5:** Add BC7 upload path in runtime

---

## Test Cases

### Header Parsing

```rust
#[test]
fn test_header_parsing() {
    // 64×32 texture header
    let data = [
        0x40, 0x00,  // width = 64 (little-endian u16)
        0x20, 0x00,  // height = 32 (little-endian u16)
        // ... pixel data follows
    ];

    let width = u16::from_le_bytes([data[0], data[1]]);
    let height = u16::from_le_bytes([data[2], data[3]]);

    assert_eq!(width, 64);
    assert_eq!(height, 32);
}

#[test]
fn test_header_max_dimensions() {
    // Max supported: 65535×65535
    let data = [0xFF, 0xFF, 0xFF, 0xFF];
    let width = u16::from_le_bytes([data[0], data[1]]);
    let height = u16::from_le_bytes([data[2], data[3]]);

    assert_eq!(width, 65535);
    assert_eq!(height, 65535);
}
```

### RGBA8 Output Size (Mode 0)

```rust
#[test]
fn test_rgba8_output_size() {
    // 64×64 RGBA8 texture
    let width: u16 = 64;
    let height: u16 = 64;

    let header_size = 4;  // 2 bytes width + 2 bytes height
    let pixel_size = (width as usize) * (height as usize) * 4;
    let total = header_size + pixel_size;

    assert_eq!(total, 4 + 16384);  // 16388 bytes
    assert_eq!(total, 16388);
}

#[test]
fn test_rgba8_32x32() {
    let total = 4 + (32 * 32 * 4);
    assert_eq!(total, 4100);  // ~4 KB
}
```

### BC7 Output Size (Mode 1-3)

```rust
#[test]
fn test_bc7_output_size() {
    // 64×64 BC7 texture
    let width: u16 = 64;
    let height: u16 = 64;

    let header_size = 4;
    let blocks_x = (width as usize) / 4;  // 16 blocks
    let blocks_y = (height as usize) / 4;  // 16 blocks
    let bc7_size = blocks_x * blocks_y * 16;  // 16 bytes per block
    let total = header_size + bc7_size;

    assert_eq!(blocks_x, 16);
    assert_eq!(blocks_y, 16);
    assert_eq!(bc7_size, 4096);  // 4 KB compressed
    assert_eq!(total, 4100);
}

#[test]
fn test_bc7_compression_ratio() {
    // Verify 4× compression ratio
    let rgba8_pixels = 64 * 64 * 4;  // 16384 bytes
    let bc7_blocks = (64 / 4) * (64 / 4) * 16;  // 4096 bytes

    assert_eq!(rgba8_pixels / bc7_blocks, 4);
}

#[test]
fn test_bc7_sizes() {
    fn bc7_size(w: usize, h: usize) -> usize {
        (w / 4) * (h / 4) * 16
    }

    assert_eq!(bc7_size(32, 32), 1024);    // 1 KB
    assert_eq!(bc7_size(64, 64), 4096);    // 4 KB
    assert_eq!(bc7_size(128, 128), 16384); // 16 KB
    assert_eq!(bc7_size(256, 256), 65536); // 64 KB
}
```

### Padding to Block Size

```rust
#[test]
fn test_pad_to_block_size_already_aligned() {
    let (padded_w, padded_h) = pad_dimensions(64, 64);
    assert_eq!(padded_w, 64);
    assert_eq!(padded_h, 64);
}

#[test]
fn test_pad_to_block_size_needs_padding() {
    // 30×30 → 32×32 (round up to multiple of 4)
    let (padded_w, padded_h) = pad_dimensions(30, 30);
    assert_eq!(padded_w, 32);
    assert_eq!(padded_h, 32);
}

#[test]
fn test_pad_to_block_size_edge_cases() {
    assert_eq!(pad_dimensions(1, 1), (4, 4));
    assert_eq!(pad_dimensions(4, 4), (4, 4));
    assert_eq!(pad_dimensions(5, 5), (8, 8));
    assert_eq!(pad_dimensions(7, 9), (8, 12));
    assert_eq!(pad_dimensions(100, 100), (100, 100));
    assert_eq!(pad_dimensions(101, 103), (104, 104));
}

fn pad_dimensions(w: usize, h: usize) -> (usize, usize) {
    let padded_w = (w + 3) & !3;
    let padded_h = (h + 3) & !3;
    (padded_w, padded_h)
}
```

### Format Selection

```rust
#[test]
fn test_format_selection_mode0() {
    // Mode 0 always RGBA8
    assert_eq!(texture_format(0, 0, false), TextureFormat::Rgba8UnormSrgb);
    assert_eq!(texture_format(0, 1, false), TextureFormat::Rgba8UnormSrgb);
}

#[test]
fn test_format_selection_mode1_matcap() {
    // Mode 1: all slots BC7 sRGB
    for slot in 0..4 {
        assert_eq!(texture_format(1, slot, true), TextureFormat::Bc7RgbaUnormSrgb);
    }
}

#[test]
fn test_format_selection_mode2_mrbp() {
    // Mode 2: slot 0 = albedo (sRGB), slot 1 = material (Linear), slot 3 = env (sRGB)
    assert_eq!(texture_format(2, 0, true), TextureFormat::Bc7RgbaUnormSrgb);  // Albedo
    assert_eq!(texture_format(2, 1, true), TextureFormat::Bc7RgbaUnorm);      // Material (Linear!)
    assert_eq!(texture_format(2, 3, true), TextureFormat::Bc7RgbaUnormSrgb);  // Environment
}

#[test]
fn test_format_selection_mode3_ssbp() {
    // Mode 3: slot 0 = albedo (sRGB), slot 1 = material (Linear), slot 2 = specular (sRGB)
    assert_eq!(texture_format(3, 0, true), TextureFormat::Bc7RgbaUnormSrgb);  // Albedo
    assert_eq!(texture_format(3, 1, true), TextureFormat::Bc7RgbaUnorm);      // Material (Linear!)
    assert_eq!(texture_format(3, 2, true), TextureFormat::Bc7RgbaUnormSrgb);  // Specular color
    assert_eq!(texture_format(3, 3, true), TextureFormat::Bc7RgbaUnormSrgb);  // Environment
}

#[test]
fn test_material_slot_is_linear() {
    // Only slot 1 in modes 2 and 3 should be linear
    fn is_linear(mode: u8, slot: u8) -> bool {
        slot == 1 && (mode == 2 || mode == 3)
    }

    // Mode 0: no linear slots
    assert!(!is_linear(0, 0));
    assert!(!is_linear(0, 1));

    // Mode 1: no linear slots (matcaps are colors)
    assert!(!is_linear(1, 0));
    assert!(!is_linear(1, 1));

    // Mode 2: only slot 1 is linear
    assert!(!is_linear(2, 0));
    assert!(is_linear(2, 1));  // Material map
    assert!(!is_linear(2, 3));

    // Mode 3: only slot 1 is linear
    assert!(!is_linear(3, 0));
    assert!(is_linear(3, 1));  // Material map
    assert!(!is_linear(3, 2));
    assert!(!is_linear(3, 3));
}
```

### wgpu Upload Calculations

```rust
#[test]
fn test_rgba8_upload_layout() {
    let width: u32 = 64;
    let height: u32 = 64;
    let is_bc7 = false;

    let bytes_per_row = width * 4;  // 4 bytes per pixel
    let rows = height;

    assert_eq!(bytes_per_row, 256);
    assert_eq!(rows, 64);
}

#[test]
fn test_bc7_upload_layout() {
    let width: u32 = 64;
    let height: u32 = 64;
    let is_bc7 = true;

    let bytes_per_row = (width / 4) * 16;  // 16 bytes per 4×4 block
    let rows = height / 4;  // Number of block rows

    assert_eq!(bytes_per_row, 256);  // 16 blocks × 16 bytes
    assert_eq!(rows, 16);            // 16 block rows
}

#[test]
fn test_bc7_upload_various_sizes() {
    fn bc7_layout(w: u32, h: u32) -> (u32, u32) {
        ((w / 4) * 16, h / 4)
    }

    assert_eq!(bc7_layout(32, 32), (128, 8));
    assert_eq!(bc7_layout(64, 64), (256, 16));
    assert_eq!(bc7_layout(128, 128), (512, 32));
    assert_eq!(bc7_layout(256, 256), (1024, 64));
}
```

### Build-Time Analysis

```rust
#[test]
fn test_analysis_default_mode() {
    // No render_mode() call → defaults to mode 0
    let state = StubFFIState::default();
    let result = AnalysisResult {
        render_mode: state.render_mode.unwrap_or(0),
        resolution: None,
        texture_ids: vec![],
    };

    assert_eq!(result.render_mode, 0);
}

#[test]
fn test_analysis_captures_render_mode() {
    let mut state = StubFFIState::default();

    // Simulate render_mode(2) call
    state.render_mode = Some(2);

    assert_eq!(state.render_mode, Some(2));
}

#[test]
fn test_analysis_captures_textures() {
    let mut state = StubFFIState::default();

    // Simulate rom_texture("player") call
    state.texture_requests.push(TextureRequest {
        handle: 1,
        width: 0,
        height: 0,
        source: TextureSource::RomPack { id: "player".to_string() },
    });

    // Simulate rom_texture("enemy") call
    state.texture_requests.push(TextureRequest {
        handle: 2,
        width: 0,
        height: 0,
        source: TextureSource::RomPack { id: "enemy".to_string() },
    });

    let ids: Vec<_> = state.texture_requests.iter()
        .filter_map(|r| match &r.source {
            TextureSource::RomPack { id } => Some(id.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(ids, vec!["player", "enemy"]);
}

#[test]
fn test_validation_valid_modes() {
    for mode in 0..=3 {
        let result = AnalysisResult {
            render_mode: mode,
            resolution: None,
            texture_ids: vec![],
        };
        assert!(validate(&result).is_ok());
    }
}

#[test]
fn test_validation_invalid_mode() {
    let result = AnalysisResult {
        render_mode: 4,  // Invalid!
        resolution: None,
        texture_ids: vec![],
    };
    assert!(validate(&result).is_err());
}
```

### Compression Round-Trip (Integration)

```rust
#[test]
fn test_rgba8_round_trip() {
    // Create 4×4 test image (smallest BC7 block)
    let pixels: Vec<u8> = (0..64).collect();  // 4×4×4 = 64 bytes

    let output = compress_texture_mode0(&pixels, 4, 4);

    // Header + pixels
    assert_eq!(output.len(), 4 + 64);

    // Verify header
    assert_eq!(&output[0..4], &[4, 0, 4, 0]);  // 4×4 in u16 LE

    // Verify pixels unchanged
    assert_eq!(&output[4..], &pixels[..]);
}

#[test]
fn test_bc7_output_has_correct_size() {
    // 8×8 test image
    let pixels = vec![0u8; 8 * 8 * 4];  // 256 bytes RGBA8

    let output = compress_texture_bc7(&pixels, 8, 8);

    // Header (4) + BC7 blocks (2×2 blocks × 16 bytes = 64)
    assert_eq!(output.len(), 4 + 64);
}
```

### PackedTexture Serialization

```rust
#[test]
fn test_packed_texture_format_default() {
    let tex = PackedTexture {
        id: "test".to_string(),
        width: 64,
        height: 64,
        format: TextureFormat::default(),
        data: vec![],
    };

    assert_eq!(tex.format, TextureFormat::Rgba8);
}

#[test]
fn test_texture_format_equality() {
    assert_eq!(TextureFormat::Rgba8, TextureFormat::Rgba8);
    assert_eq!(TextureFormat::Bc7, TextureFormat::Bc7);
    assert_eq!(TextureFormat::Bc7Linear, TextureFormat::Bc7Linear);

    assert_ne!(TextureFormat::Rgba8, TextureFormat::Bc7);
    assert_ne!(TextureFormat::Bc7, TextureFormat::Bc7Linear);
}
```

### Edge Cases

```rust
#[test]
fn test_1x1_texture_padded_to_4x4() {
    // 1×1 must be padded to 4×4 for BC7
    let (w, h) = pad_dimensions(1, 1);
    assert_eq!((w, h), (4, 4));

    // BC7 size for 4×4 = 1 block = 16 bytes
    let bc7_size = (w / 4) * (h / 4) * 16;
    assert_eq!(bc7_size, 16);
}

#[test]
fn test_non_square_texture() {
    // 128×64 texture
    let bc7_size = (128 / 4) * (64 / 4) * 16;
    assert_eq!(bc7_size, 32 * 16 * 16);  // 8192 bytes

    let rgba8_size = 128 * 64 * 4;
    assert_eq!(rgba8_size, 32768);  // 32 KB

    // Compression ratio still 4×
    assert_eq!(rgba8_size / bc7_size, 4);
}

#[test]
fn test_procedural_texture_always_rgba8() {
    // Procedural textures bypass compression regardless of mode
    for mode in 0..=3 {
        let format = procedural_texture_format(mode);
        assert_eq!(format, TextureFormat::Rgba8UnormSrgb);
    }
}
```

---

## Error Handling

All texture loading errors result in a **WASM trap** (panic). There is no graceful fallback for corrupted or missing textures.

### Build-Time Errors

| Error | Cause | Behavior |
|-------|-------|----------|
| Invalid render mode | `render_mode()` returns value > 3 | Build fails with `BuildError::InvalidRenderMode` |
| Missing texture | Asset ID not found in manifest | Build fails with `BuildError::MissingAsset` |
| Image decode failure | Corrupt PNG/JPG source file | Build fails with `BuildError::ImageDecode` |
| BC7 compression failure | Internal texpresso error | Build fails with `BuildError::CompressionFailed` |

### Runtime Errors (WASM Traps)

| Error | Cause | Behavior |
|-------|-------|----------|
| Invalid handle | `rom_texture()` with unknown ID | Trap: "texture not found: {id}" |
| Corrupted header | Header dimensions = 0 or > 4096 | Trap: "invalid texture dimensions" |
| Truncated data | BC7 data shorter than expected | Trap: "texture data truncated" |
| Out of VRAM | GPU allocation failure | Trap: "VRAM allocation failed" |

### Validation (Build-Time)

```rust
fn validate_texture_header(header: &TextureHeader) -> Result<(), TextureError> {
    if header.width == 0 || header.height == 0 {
        return Err(TextureError::InvalidDimensions);
    }
    if header.width > 4096 || header.height > 4096 {
        return Err(TextureError::DimensionsTooLarge);
    }
    // BC7 requires dimensions divisible by 4
    if header.format == TextureFormat::Bc7 {
        if header.width % 4 != 0 || header.height % 4 != 0 {
            return Err(TextureError::NotBlockAligned);
        }
    }
    Ok(())
}
```

### Why Trap Instead of Fallback?

- **Fail fast** — Corrupted assets indicate a serious build/packaging bug
- **No silent failures** — Pink checkerboard textures hide problems
- **Init-only loading** — All textures load in `init()`, so traps happen before gameplay
- **Determinism** — Rollback netcode requires identical state; fallback textures could desync

---

## Resolved Questions

| Question | Resolution |
|----------|------------|
| Default mode | Missing `render_mode()` defaults to mode 0 (Unlit) |
| Encoder quality | Use `texpresso::Params::default()` for all builds |
| Mipmap generation | Not at this time (future enhancement) |
| Stipple separate spec | Yes, transparency has its own spec |
| Error handling | Trap on all errors (no fallback textures) |

---

## References

- [BC7 Format (Microsoft)](https://learn.microsoft.com/en-us/windows/win32/direct3d11/bc7-format)
- [texpresso crate](https://crates.io/crates/texpresso) — **Verified:** v2.0.2, pure Rust, MIT license, BC7 (BPTC) support confirmed
- [intel_tex_2 crate](https://crates.io/crates/intel_tex_2)
- [Bayer Dithering](https://en.wikipedia.org/wiki/Ordered_dithering)
- [wgpu TextureFormat](https://docs.rs/wgpu/latest/wgpu/enum.TextureFormat.html)
