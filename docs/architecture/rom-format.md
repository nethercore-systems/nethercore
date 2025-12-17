# Emberware ROM Format Specification

## Overview

Emberware uses console-specific binary ROM formats for game distribution. Each fantasy console has its own ROM format with a unique file extension and structure, ensuring type-safety and preventing incompatible games from being loaded on the wrong console.

**Current ROM Formats:**
- `.ewz` - Emberware ZX (3d Home console)
- `.ewc` - Emberware Chroma (2d Handheld console)

**Why Console-Specific?**
- ✅ Type-safe - impossible to load wrong ROM in wrong console
- ✅ Console-specific metadata embedded in ROM structure
- ✅ Compile-time guarantees about compatibility
- ✅ No runtime format detection needed
- ✅ Each console can evolve independently

**Why Bitcode?**
- ✅ Compact binary format with excellent compression
- ✅ Native Rust struct serialization - type-safe
- ✅ Fast deserialization (faster than ZIP + TOML parsing)
- ✅ Games can `#[derive(Encode)]` for ROM metadata
- ✅ No external dependencies for inspection (just Rust tooling)

## Emberware Z ROM Format (.ewz)

### File Structure

```
game.ewz (binary file)
├── Magic bytes: "EWZ\0" (4 bytes)
└── ZRom (bitcode-encoded):
    ├── version: u32
    ├── metadata: ZMetadata
    ├── code: Vec<u8> (WASM bytes)
    ├── thumbnail: Option<Vec<u8>> (PNG, extracted locally)
    └── screenshots: Vec<Vec<u8>> (PNG, stored in ROM only)
```

### Magic Bytes

All Emberware Z ROM files start with the magic bytes: `EWZ\0` (hex: `45 57 5A 00`)

This allows tools to quickly identify the ROM type and reject invalid files.

### ZRom Structure

```rust
pub struct ZRom {
    /// ROM format version (currently 1)
    pub version: u32,

    /// Game metadata
    pub metadata: ZMetadata,

    /// Compiled WASM code
    pub code: Vec<u8>,

    /// Optional thumbnail (256x256 PNG, extracted locally during installation)
    pub thumbnail: Option<Vec<u8>>,

    /// Optional screenshots (PNG bytes, max 5)
    /// These are stored in the ROM but NOT extracted during installation
    /// to save disk space. They can be displayed when viewing ROM info
    /// or on the platform game page.
    pub screenshots: Vec<Vec<u8>>,
}
```

### ZMetadata Structure

```rust
pub struct ZMetadata {
    // Core game info
    /// Game slug (e.g., "platformer")
    pub id: String,

    /// Display title
    pub title: String,

    /// Primary author/studio name (for display)
    pub author: String,

    /// Semantic version (e.g., "1.0.0")
    pub version: String,

    /// Game description
    pub description: String,

    /// Category tags
    pub tags: Vec<String>,

    // Platform integration (optional foreign keys)
    /// UUID linking to platform game record
    pub platform_game_id: Option<String>,

    /// UUID linking to platform user/studio
    pub platform_author_id: Option<String>,

    // Creation info
    /// ISO 8601 timestamp when ROM was created
    pub created_at: String,

    /// xtask version that created this ROM
    pub tool_version: String,

    // Z-specific settings
    /// Render mode: 0=Unlit, 1=Matcap, 2=PBR, 3=Hybrid
    pub render_mode: Option<u32>,

    /// Default resolution (e.g., "640x480")
    pub default_resolution: Option<String>,

    /// Target FPS
    pub target_fps: Option<u32>,
}
```

### Field Descriptions

#### Core Fields (Required)

- **id**: Game identifier (slug format, e.g., "super-platformer")
  - Used for file system directories
  - Must be unique per author
  - Should be URL-safe (lowercase, hyphens only)

- **title**: Human-readable game title (e.g., "Super Platformer")
  - Displayed in library UI
  - Can contain spaces, capitals, special characters

- **author**: Primary author or studio name
  - Simple string for offline display
  - Platform backend handles complex credits (multiple authors, roles, etc.)

- **version**: Semantic version string (e.g., "1.0.0")
  - Must follow semver format: MAJOR.MINOR.PATCH
  - Used for update detection

- **description**: Game description/summary
  - Short paragraph describing the game
  - Displayed in library UI and platform

- **tags**: Array of category tags
  - Used for filtering and search
  - Examples: "platformer", "action", "puzzle", "multiplayer"

#### Platform Integration (Optional)

- **platform_game_id**: UUID linking to platform game record
  - Populated when downloading from platform
  - Enables "Check for updates", "View on platform"
  - `null` for offline/local ROMs

- **platform_author_id**: UUID linking to platform user/studio
  - Links to author profile on platform
  - `null` for offline/local ROMs

**Note:** The platform backend handles complex credits:
- Multiple authors/collaborators
- Specific roles (programmer, artist, composer, etc.)
- Rich profiles with avatars, bios, social links
- Game pages with full descriptions and media galleries

The ROM keeps a simple `author` string for offline display when the platform is unavailable.

#### Creation Info (Auto-populated)

- **created_at**: ISO 8601 timestamp
  - Generated automatically when ROM is created
  - Example: "2025-01-15T14:30:00Z"

- **tool_version**: Version of xtask that created the ROM
  - Example: "0.1.0"
  - Useful for debugging ROM issues

#### Console-Specific Settings (Emberware Z)

- **render_mode**: Rendering mode for the game
  - `0` = Unlit (flat shading, no lighting)
  - `1` = Matcap (matcap-based lighting)
  - `2` = PBR-lite (physically-based rendering)
  - `3` = Hybrid (mix of techniques)
  - Optional - defaults to PBR-lite if not specified

- **default_resolution**: Preferred window resolution
  - Format: "WIDTHxHEIGHT" (e.g., "640x480", "1280x720")
  - Optional - launcher uses default if not specified

- **target_fps**: Target frame rate
  - Integer FPS value (e.g., 60, 30)
  - Optional - launcher uses default if not specified

## Validation Rules

When a ROM is loaded, the following validation is performed:

### Format Validation
- Magic bytes must be "EWZ\0"
- File must deserialize successfully using bitcode

### Version Validation
- ROM version must be ≤ current supported version (currently 1)
- Future versions may introduce new features while maintaining backward compatibility

### Metadata Validation
- Required fields must not be empty:
  - `id`
  - `title`
  - `author`
  - `version`
  - `description`

### WASM Code Validation
- Code must be at least 4 bytes
- Must start with WASM magic bytes: `\0asm` (hex: `00 61 73 6D`)
- This ensures the WASM module is valid before installation

### Asset Validation
- Thumbnails and screenshots are NOT validated during ROM creation
- PNG validation happens during loading (not critical for ROM validity)
- Max 5 screenshots allowed

### Console Settings Validation
- **render_mode**: Must be 0-3 if specified
- **default_resolution**: No format validation (parsed at runtime)
- **target_fps**: No validation (any positive integer)

## Installation to Local Library

When a ROM is installed to the local library, the following happens:

```
~/.emberware/games/{game_id}/
├── manifest.json        # Backward compatibility with existing library UI
├── rom.wasm            # Extracted WASM code
└── thumbnail.png       # Extracted thumbnail (if present)
                        # Screenshots NOT extracted (save disk space)
```

**Why not extract screenshots?**
- Screenshots are only needed when viewing ROM info or on platform
- Extracting them wastes disk space (5 screenshots × ~500KB = ~2.5MB per game)
- They remain in the ROM file and can be displayed on-demand

## Creating ROMs

See [distributing-games.md](./distributing-games.md) for a complete guide on creating and distributing ROM files.

Quick example:

```bash
cargo xtask cart create-z game.wasm \
  --id my-game \
  --title "My Awesome Game" \
  --author "YourName" \
  --version "1.0.0" \
  --description "A fun game!" \
  --tag platformer \
  --tag action \
  --thumbnail assets/thumbnail.png \
  --screenshot assets/screenshot1.png \
  --render-mode 2 \
  --default-resolution "640x480" \
  --target-fps 60 \
  --output my-game.ewz
```

## Inspecting ROMs

You can inspect a ROM's metadata without installing it:

```bash
cargo xtask cart info my-game.ewz
```

This displays:
- Game information (title, author, version, description, tags)
- Creation info (timestamp, tool version, ROM version)
- Platform integration (UUIDs if present)
- Console settings (render mode, resolution, FPS)
- ROM contents (file sizes, thumbnail, screenshots)

## Future Console Formats

When new consoles are added (e.g., Emberware Chroma), they will follow the same pattern:

```rust
// Emberware Chroma ROM (.ewc)
pub const EWC_VERSION: u32 = 1;
pub const EWC_MAGIC: &[u8; 4] = b"EWC\0";

pub struct ChromaRom {
    pub version: u32,
    pub metadata: ChromaMetadata,
    pub code: Vec<u8>,
    pub thumbnail: Option<Vec<u8>>,
    pub screenshots: Vec<Vec<u8>>,
}

pub struct ChromaMetadata {
    // Core fields (same as Z for consistency)
    pub id: String,
    pub title: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub tags: Vec<String>,
    pub platform_game_id: Option<String>,
    pub platform_author_id: Option<String>,
    pub created_at: String,
    pub tool_version: String,

    // Chroma-specific settings
    pub palette: Option<String>,     // "nes", "snes", "genesis"
    pub sprite_limit: Option<u32>,
    pub audio_channels: Option<u32>,
}
```

This ensures:
- Each console has type-safe ROM handling
- Console-specific metadata is first-class
- Tools can dispatch based on file extension
- No runtime format detection needed

## Technical Details

### Bitcode Serialization

ROMs use the [bitcode](https://crates.io/crates/bitcode) crate for serialization with native `Encode`/`Decode` traits (NOT serde).

**Why native Encode/Decode?**
- Faster encoding/decoding (no serde overhead)
- Smaller ROM files (~10-20% smaller than serde-bitcode)
- Simpler dependency tree
- Purpose-built for binary formats

**Format Properties:**
- Binary (not human-readable)
- Deterministic (same input → same output)
- Compact (better compression than JSON/TOML)
- Fast (faster than ZIP + parsing)

### File Size Expectations

Typical ROM sizes:

```
Minimal game (hello-world):
- WASM code: ~1KB
- Metadata: ~200 bytes
- Total: ~1.2KB

Medium game:
- WASM code: ~100KB
- Thumbnail: ~20KB
- Metadata: ~500 bytes
- Total: ~120KB

Large game with assets:
- WASM code: ~500KB
- Thumbnail: ~20KB
- 5 screenshots: ~2.5MB
- Metadata: ~1KB
- Total: ~3MB
```

## Error Messages

Common validation errors and their meanings:

**"Invalid EWZ magic bytes"**
- File is not an Emberware Z ROM
- File may be corrupted
- Wrong ROM type (trying to load .ewc as .ewz)

**"Unsupported EWZ version: X"**
- ROM was created with a newer tool version
- Update your launcher to support this ROM

**"Game ID cannot be empty"**
- ROM metadata is missing required `id` field

**"Invalid WASM code (missing \\0asm magic bytes)"**
- WASM file is corrupted or invalid
- WASM compilation may have failed

**"Failed to decode EWZ ROM"**
- ROM file is corrupted
- ROM was created with incompatible bitcode version

## Backward Compatibility

ROM installation maintains backward compatibility with the existing library system:

- **manifest.json** is still created during installation
- Existing library UI works unchanged
- Raw WASM files can still be loaded for development
- `build-examples` continues to work as before

The ROM format is an **addition** for distribution, not a replacement for development workflows.

## Best Practices

1. **Always validate ROMs before distribution**
   - Use `cargo xtask cart info` to verify metadata
   - Test installation with `install_z_rom()`
   - Ensure WASM code runs correctly

2. **Include a thumbnail**
   - Makes games more discoverable in library UI
   - 256x256 PNG, will be auto-resized if larger
   - Keep file size reasonable (~20KB max)

3. **Add descriptive tags**
   - Helps users find games by genre/category
   - Use standard tags: "platformer", "puzzle", "action", etc.
   - Max 5-10 tags recommended

4. **Version ROMs properly**
   - Use semantic versioning (MAJOR.MINOR.PATCH)
   - Increment MAJOR for breaking changes
   - Increment MINOR for new features
   - Increment PATCH for bug fixes

5. **Test on target console**
   - ROMs are console-specific
   - Ensure console settings match your game's needs
   - Test with different render modes if applicable

## See Also

- [distributing-games.md](./distributing-games.md) - Complete guide for game developers
- [ffi.md](./ffi.md) - Emberware FFI API reference
- [emberware-z.md](./emberware-z.md) - Z-specific API documentation
