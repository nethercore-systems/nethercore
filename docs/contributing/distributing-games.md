# Distributing Nethercore Games

This guide covers how to package and distribute your Nethercore games as ROM files for players to install and play.

## Table of Contents

1. [Overview](#overview)
2. [Development Workflow](#development-workflow)
3. [Creating a nether.toml Manifest](#creating-a-nethert oml-manifest)
4. [Building Your Game](#building-your-game)
5. [Adding Assets (Thumbnails & Screenshots)](#adding-assets-thumbnails--screenshots)
6. [Console-Specific Settings](#console-specific-settings)
7. [Testing Your ROM](#testing-your-rom)
8. [Distributing Your Game](#distributing-your-game)
9. [Platform Integration](#platform-integration)
10. [Versioning and Updates](#versioning-and-updates)
11. [Troubleshooting](#troubleshooting)

## Overview

Nethercore uses console-specific ROM formats for game distribution:

- **Nethercore ZX**: `.nczx` files (PS1/N64 aesthetic)
- **Nethercore Chroma**: `.ncc` files (future - 2D retro aesthetic)

ROM files package your game's WASM code, metadata, and assets into a single binary file that players can easily install and play.

## Development Workflow

The `nether` CLI is your main build tool for Nethercore games. It handles everything from creating project manifests to building and running your games.

### Quick Start

```bash
# Create a new project manifest
nether init

# Build your game (compile + pack into .nczx ROM)
nether build

# Build and launch in emulator
nether run
```

### Workflow Overview

1. **Development**: Use `nether run` for fast iteration
2. **Testing**: Use `nether build` to create a ROM file for testing
3. **Distribution**: Share the `.nczx` ROM file with players

## Creating a nether.toml Manifest

The `nether.toml` file contains all metadata and configuration for your game.

### Step 1: Initialize Manifest

```bash
cd my-game
nether init
```

This creates a `nether.toml` file with default values.

### Step 2: Configure Metadata

Edit `nether.toml` to add your game's information:

```toml
[game]
id = "my-game"                          # Unique identifier (lowercase, hyphens only)
title = "My Awesome Game"                # Display name
author = "YourName"                      # Your name or studio
version = "1.0.0"                        # Semantic version
description = "A fun platforming adventure!"  # Game description
tags = ["platformer", "action", "singleplayer"]  # Category tags
render_mode = 2                          # 0=Lambert, 1=Matcap, 2=PBR (default), 3=Hybrid

[build]
# Build script to compile your game to WASM
# For Rust projects:
command = "cargo build --target wasm32-unknown-unknown --release"
wasm_path = "target/wasm32-unknown-unknown/release/my_game.wasm"

# For C projects:
# command = "zig build"
# wasm_path = "zig-out/bin/game.wasm"

# For Zig projects:
# command = "zig build"
# wasm_path = "zig-out/bin/game.wasm"
```

### Required Fields

- **id**: Unique game identifier (slug format: lowercase, hyphens)
- **title**: Display name shown in library
- **author**: Developer name or studio
- **version**: Semantic version (MAJOR.MINOR.PATCH)
- **description**: Brief game description
- **build.command**: Command to compile your game
- **build.wasm_path**: Path to compiled WASM output

### Optional Fields

- **tags**: Category tags (e.g., "platformer", "puzzle", "multiplayer")
- **render_mode**: Visual style (0-3, see Console-Specific Settings)
- **default_resolution**: Suggested window size (e.g., "640x480")
- **target_fps**: Target frame rate (e.g., 60)

## Building Your Game

### Build and Package

```bash
# Build WASM and create .nczx ROM
nether build

# Specify custom manifest location
nether build --manifest path/to/nether.toml

# Specify custom output path
nether build --output my-game.nczx
```

This will:
1. Run your build command to compile WASM
2. Bundle WASM + assets into a `.nczx` ROM file
3. Output the ROM to `{game-id}.nczx` (or path specified with `--output`)

### Build and Run

```bash
# Build and launch in emulator (fast iteration)
nether run
```

This builds your game and immediately launches it in the Nethercore player.

### Pack Only (Skip Compilation)

If you've already compiled your WASM manually:

```bash
# Just pack WASM + assets into ROM
nether pack

# Override WASM path
nether pack --wasm path/to/custom.wasm
```

## Adding Assets (Thumbnails & Screenshots)

### Thumbnails

Thumbnails are displayed in the game library UI. Add to your `nether.toml`:

```toml
[game]
# ... other fields ...
thumbnail = "assets/thumbnail.png"
```

**Requirements:**
- PNG format
- Recommended: 256x256 pixels (will be auto-resized if larger)
- Keep file size reasonable (~20KB max)

**Best Practices:**
- Show gameplay or key visual
- Use clear, recognizable art
- Avoid text-heavy images (may not be readable at small sizes)

### Screenshots

Screenshots are stored in the ROM for viewing on the platform or in ROM info:

```toml
[game]
# ... other fields ...

[[screenshots]]
path = "assets/screenshot1.png"

[[screenshots]]
path = "assets/screenshot2.png"

[[screenshots]]
path = "assets/screenshot3.png"
```

**Requirements:**
- PNG format
- Max 5 screenshots
- Any resolution (keep reasonable for file size)

**Important:** Screenshots are NOT extracted during installation to save disk space. They remain in the ROM file and are displayed when viewing ROM info.

### Example Manifest with Assets

```toml
[game]
id = "super-platformer"
title = "Super Platformer Adventure"
author = "Indie Dev Studio"
version = "1.0.0"
description = "Jump, run, and explore in this retro-inspired platformer!"
tags = ["platformer", "action", "retro"]
thumbnail = "assets/thumbnail.png"
render_mode = 2
default_resolution = "640x480"
target_fps = 60

[build]
command = "cargo build --target wasm32-unknown-unknown --release"
wasm_path = "target/wasm32-unknown-unknown/release/my_platformer.wasm"

[[screenshots]]
path = "assets/screenshot1.png"

[[screenshots]]
path = "assets/screenshot2.png"

[[screenshots]]
path = "assets/screenshot3.png"
```

### Creating Assets

You can capture screenshots during gameplay using Nethercore's built-in screenshot feature (F9 by default) or your OS's screenshot tool.

For thumbnails, you can:
- Capture a representative gameplay moment
- Create custom cover art
- Use your game's title screen

## Console-Specific Settings

### Nethercore ZX Settings

#### Render Mode

The render mode determines the visual style of your game:

```toml
[game]
render_mode = 0  # Lambert (simple diffuse shading)
# render_mode = 1  # Matcap (matcap-based lighting)
# render_mode = 2  # PBR-lite (physically-based rendering) - default
# render_mode = 3  # Hybrid (mix of techniques)
```

**Which to choose?**
- **Lambert (0)**: Retro flat-shaded look (e.g., early 3D games)
- **Matcap (1)**: Stylized lighting with matcap textures
- **PBR-lite (2)**: Modern PBR look (most realistic) - **default**
- **Hybrid (3)**: Mix matcap and PBR for unique styles

If not specified, defaults to PBR-lite (mode 2).

#### Default Resolution

Suggest a default window size for your game:

```toml
[game]
default_resolution = "640x480"    # Retro 4:3
# default_resolution = "1280x720"  # HD 16:9
# default_resolution = "1920x1080" # Full HD
```

Players can still resize the window, but this sets the initial size.

#### Target FPS

Specify your game's target frame rate:

```toml
[game]
target_fps = 60   # Smooth 60fps gameplay
# target_fps = 30  # Cinematic 30fps
```

This is a hint to the launcher but doesn't enforce the frame rate.

## Testing Your ROM

### 1. Build and Test Locally

```bash
# Build and run in emulator
nether run
```

This is the fastest way to test during development.

### 2. Test ROM Installation

Build a ROM and verify it works:

```bash
# Create ROM file
nether build --output my-game.nczx

# Launch the game library
cargo run -p nethercore-library

# Or copy ROM to games directory manually
cp my-game.nczx ~/.nethercore/roms/
```

### 3. Verify Installation

Check that the game was installed correctly:

```bash
# Check games directory
ls ~/.nethercore/games/my-game/

# Should contain:
# - manifest.json
# - rom.wasm
# - thumbnail.png (if thumbnail was included)
```

### 4. Test Gameplay

Launch the game and verify:
- Game loads and runs correctly
- No crashes or errors
- Save/load works (if applicable)
- Multiplayer works (if applicable)
- Console settings are applied correctly

## Distributing Your Game

### Sharing Options

**1. Direct Download**
- Upload your `.nczx` file to a file host
- Share the download link
- Players install manually via drag-and-drop or library UI

**2. Platform Upload (Future)**
- Upload to the official Nethercore platform at [nethercore.systems](https://nethercore.systems)
- Players can browse and download from the platform
- Automatic version updates
- Player ratings and reviews

**3. Itch.io / Game Jolt**
- Upload as a downloadable file
- Add installation instructions
- Link to Nethercore launcher download

**4. GitHub Releases**
- Tag a release in your game's repository
- Attach the `.nczx` file as a release asset
- Players can download from releases page

### Distribution Checklist

Before distributing:

- ✅ Game builds successfully with `nether build`
- ✅ Game runs correctly with `nether run`
- ✅ Version number follows semantic versioning
- ✅ Description is clear and accurate
- ✅ Appropriate tags are set
- ✅ Thumbnail is included and looks good
- ✅ Screenshots showcase gameplay (if included)
- ✅ Game has been tested on fresh install
- ✅ README or instructions provided for players
- ✅ License information is clear

### Installation Instructions for Players

Provide players with clear installation instructions:

```markdown
## How to Play

1. Download and install the [Nethercore Launcher](https://nethercore.systems/download)
2. Download the game ROM file: `my-game.nczx`
3. Open the Nethercore Launcher
4. Drag and drop `my-game.nczx` onto the launcher window
5. Click the game in your library to play!
```

## Platform Integration

### Platform Foreign Keys

When uploading to the official Nethercore platform, the platform will populate foreign keys in your ROM metadata. These enable:
- "Check for updates" functionality
- "View on platform" links
- Download statistics
- Player reviews and ratings

You don't need to set these manually - the platform handles it during upload.

### Complex Credits

The ROM format uses a simple `author` field for offline display. For complex credits:

**In nether.toml:**
```toml
[game]
author = "Indie Dev Studio"
```

**On Platform:**
- Multiple authors/collaborators
- Specific roles (programmer, artist, composer, sound designer)
- Rich profiles with avatars, bios, social links
- Game pages with full descriptions and media galleries

The platform backend manages the complex credit system, while the ROM keeps it simple for offline play.

## Versioning and Updates

### Semantic Versioning

Use semantic versioning in your `nether.toml`:

```toml
[game]
version = "1.0.0"  # MAJOR.MINOR.PATCH
```

**When to increment:**
- **MAJOR**: Breaking changes (incompatible save files, major rewrites)
- **MINOR**: New features, additions (new levels, characters)
- **PATCH**: Bug fixes, small improvements

**Examples:**
```
1.0.0 - Initial release
1.0.1 - Bug fix
1.1.0 - New feature
2.0.0 - Breaking change
```

### Publishing Updates

When you release a new version:

1. **Update version in nether.toml:**
   ```toml
   [game]
   version = "1.1.0"
   ```

2. **Build new ROM:**
   ```bash
   nether build --output my-game-v1.1.0.nczx
   ```

3. **Distribute:**
   - Upload to platform (auto-detects as update)
   - Or share new download link

4. **Notify players:**
   - Platform users get update notification
   - Direct download users need manual notification

### Save Compatibility

When updating, consider save file compatibility:

- **Minor/Patch updates**: Save files should work
- **Major updates**: May break save compatibility
  - Warn users in update notes
  - Provide save migration if possible

## Troubleshooting

### "Invalid WASM code (missing \\0asm magic bytes)"

**Cause:** WASM file is invalid or corrupted

**Fix:**
1. Rebuild your game: `nether build`
2. Verify the WASM file exists and is not empty
3. Check for build errors in output

### "Game ID cannot be empty"

**Cause:** Missing or empty `id` field in nether.toml

**Fix:**
```toml
[game]
id = "my-game"  # Must be specified
```

### "ROM file too large"

**Cause:** Screenshots or assets are too large

**Fix:**
1. Compress screenshots (use PNG optimization tools)
2. Reduce screenshot resolution if very large
3. Limit screenshots to 3-5 images
4. Keep thumbnail under 20KB

### "Invalid render mode: 4"

**Cause:** Render mode must be 0-3

**Fix:**
```toml
[game]
render_mode = 2  # Valid: 0, 1, 2, or 3
```

### ROM builds but game won't launch

**Possible causes:**
1. WASM file has runtime errors (check launcher console)
2. Missing required FFI functions
3. Incompatible with current launcher version

**Fix:**
1. Test game with `nether run` during development
2. Check launcher version compatibility
3. Review error messages in launcher console

### Build command fails

**Cause:** Build command in nether.toml is incorrect

**Fix:**
1. Verify the build command works independently
2. Check that `wasm_path` points to the correct output
3. Ensure all build dependencies are installed

## Best Practices

### Development Workflow

1. **Use `nether run`** for fast iteration during development
2. **Create ROM for testing** before each release with `nether build`
3. **Version consistently** using semantic versioning
4. **Test installation** on a clean setup
5. **Document changes** in update notes

### ROM File Naming

Use descriptive filenames when building:

```bash
# Good
nether build --output my-game-v1.0.0.nczx
nether build --output super-platformer.nczx
nether build --output puzzle-quest-v2.1.0.nczx

# Avoid
nether build --output game.nczx
nether build --output test.nczx
nether build --output final-final-v2.nczx
```

### Metadata Quality

- **Title**: Clear, memorable, searchable
- **Description**: 1-2 paragraphs, highlight key features
- **Tags**: 3-5 relevant tags, use standard categories
- **Author**: Consistent name across all games
- **Version**: Follow semantic versioning

### Asset Guidelines

**Thumbnails:**
- 256x256 pixels
- Show gameplay or key visual
- High contrast for visibility
- ~20KB file size

**Screenshots:**
- Showcase diverse gameplay
- High quality but not massive files
- 3-5 screenshots recommended
- Capture interesting moments

## Example: Complete Release Workflow

Here's a complete example of preparing a game for distribution:

```bash
# 1. Navigate to your project
cd my-platformer

# 2. Ensure nether.toml is complete
cat nether.toml
```

```toml
[game]
id = "super-platformer"
title = "Super Platformer Adventure"
author = "Indie Dev Studio"
version = "1.0.0"
description = "Jump, run, and explore through mysterious worlds filled with challenging platforming and hidden secrets! Featuring 20 unique levels, boss battles, and unlockable characters."
tags = ["platformer", "action", "adventure"]
thumbnail = "assets/thumbnail.png"
render_mode = 2
default_resolution = "640x480"
target_fps = 60

[build]
command = "cargo build --target wasm32-unknown-unknown --release"
wasm_path = "target/wasm32-unknown-unknown/release/my_platformer.wasm"

[[screenshots]]
path = "assets/screenshot1.png"

[[screenshots]]
path = "assets/screenshot2.png"

[[screenshots]]
path = "assets/screenshot3.png"
```

```bash
# 3. Create assets (if not already done)
# - assets/thumbnail.png (256x256)
# - assets/screenshot1.png (gameplay)
# - assets/screenshot2.png (boss fight)
# - assets/screenshot3.png (level select)

# 4. Build ROM
nether build --output super-platformer-v1.0.0.nczx

# 5. Test the ROM
nether run

# 6. Distribute
# - Upload to nethercore.systems
# - Upload to itch.io/GitHub releases
# - Share download link
```

## See Also

- [rom-format.md](../architecture/rom-format.md) - Technical ROM format specification
- [Getting Started](../book/src/getting-started/first-game.md) - Build your first game
- [Asset Pipeline](../book/src/guides/asset-pipeline.md) - Converting and bundling assets

## Getting Help

If you run into issues:

1. Check this guide and the [rom-format.md](../architecture/rom-format.md) specification
2. Use `nether --help` for CLI reference
3. Ask in the Nethercore Discord/forums
4. Open an issue on GitHub if you find a bug
