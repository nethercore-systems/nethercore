# Distributing Emberware Games

This guide covers how to package and distribute your Emberware games as ROM files for players to install and play.

## Table of Contents

1. [Overview](#overview)
2. [Development vs. Distribution](#development-vs-distribution)
3. [Creating a ROM](#creating-a-rom)
4. [Adding Assets (Thumbnails & Screenshots)](#adding-assets-thumbnails--screenshots)
5. [Console-Specific Settings](#console-specific-settings)
6. [Testing Your ROM](#testing-your-rom)
7. [Distributing Your Game](#distributing-your-game)
8. [Platform Integration](#platform-integration)
9. [Versioning and Updates](#versioning-and-updates)
10. [Troubleshooting](#troubleshooting)

## Overview

Emberware uses console-specific ROM formats for game distribution:

- **Emberware ZX**: `.ewz` files (PS1/N64 aesthetic)
- **Emberware Chroma**: `.ewc` files (future - 2D retro aesthetic)

ROM files package your game's WASM code, metadata, and assets into a single binary file that players can easily install and play.

## Development vs. Distribution

### During Development

While developing, you can use raw WASM files for fast iteration:

```bash
# Build your game
cd my-game
cargo build --target wasm32-unknown-unknown --release

# Copy to games directory manually or use build-examples
cargo xtask build-examples
```

This creates a simple directory structure:
```
~/.emberware/games/my-game/
├── manifest.json
└── rom.wasm
```

**Benefits:**
- Fast iteration
- No packaging overhead
- Easy to debug

### For Distribution

When you're ready to share your game, create a ROM file:

```bash
cargo xtask cart create-z my-game.wasm \
  --id my-game \
  --title "My Awesome Game" \
  --author "YourName" \
  --version "1.0.0" \
  --description "A fun platforming adventure!" \
  --tag platformer \
  --output my-game.ewz
```

**Benefits:**
- Single file for distribution
- Rich metadata (title, description, tags)
- Thumbnails and screenshots
- Version tracking
- Platform integration

## Creating a ROM

### Step 1: Build Your Game

First, compile your game to WASM:

```bash
cd my-game
cargo build --target wasm32-unknown-unknown --release
```

The WASM file will be at:
```
target/wasm32-unknown-unknown/release/my_game.wasm
```

### Step 2: Create the ROM

Use the `cart create-z` command with your game's metadata:

```bash
cargo xtask cart create-z \
  target/wasm32-unknown-unknown/release/my_game.wasm \
  --id my-game \
  --title "My Awesome Game" \
  --author "YourName" \
  --version "1.0.0" \
  --description "An exciting platforming adventure through mysterious worlds!" \
  --tag platformer \
  --tag action \
  --tag singleplayer \
  --output my-game.ewz
```

### Required Arguments

- **WASM file path**: Path to your compiled `.wasm` file
- `--id`: Game identifier (slug format, e.g., "my-game")
  - Use lowercase and hyphens only
  - Must be unique (used for file system directories)
- `--title`: Display name (e.g., "My Awesome Game")
- `--author`: Your name or studio name
- `--version`: Semantic version (e.g., "1.0.0")
- `--description`: Game description/summary
- `--output` / `-o`: Output ROM file path

### Optional Arguments

- `--tag`: Category tags (can be specified multiple times)
  - Examples: platformer, puzzle, action, adventure, multiplayer
- `--thumbnail`: Thumbnail image (PNG, auto-resized to 256x256)
- `--screenshot`: Screenshot images (PNG, max 5)
- `--render-mode`: Rendering mode (0-3, see below)
- `--default-resolution`: Default window size (e.g., "640x480")
- `--target-fps`: Target frame rate (e.g., 60)

## Adding Assets (Thumbnails & Screenshots)

### Thumbnails

Thumbnails are displayed in the game library UI:

```bash
cargo xtask cart create-z my_game.wasm \
  --id my-game \
  --title "My Game" \
  --author "YourName" \
  --version "1.0.0" \
  --description "..." \
  --thumbnail assets/thumbnail.png \
  --output my-game.ewz
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

Screenshots are stored in the ROM for viewing in ROM info or on the platform:

```bash
cargo xtask cart create-z my_game.wasm \
  --id my-game \
  --title "My Game" \
  --author "YourName" \
  --version "1.0.0" \
  --description "..." \
  --screenshot assets/screenshot1.png \
  --screenshot assets/screenshot2.png \
  --screenshot assets/screenshot3.png \
  --output my-game.ewz
```

**Requirements:**
- PNG format
- Max 5 screenshots
- Any resolution (keep reasonable for file size)

**Important:** Screenshots are NOT extracted during installation to save disk space. They remain in the ROM file and are displayed when viewing ROM info.

### Creating Assets

You can capture screenshots during gameplay using your OS's screenshot tool, or add a screenshot key to your game:

```rust
// Example: F12 to save screenshot
fn update() {
    if key_pressed(KEY_F12) {
        // Use your graphics backend to capture the frame buffer
        save_screenshot("screenshot.png");
    }
}
```

For thumbnails, you can:
- Capture a representative gameplay moment
- Create custom cover art
- Use your game's title screen

## Console-Specific Settings

### Emberware Z Settings

When creating an Emberware Z ROM, you can specify console-specific settings:

#### Render Mode

The render mode determines the visual style of your game:

```bash
--render-mode 0  # Unlit (flat shading, no lighting)
--render-mode 1  # Matcap (matcap-based lighting)
--render-mode 2  # PBR-lite (physically-based rendering) - default
--render-mode 3  # Hybrid (mix of techniques)
```

**Which to choose?**
- **Unlit (0)**: Retro flat-shaded look (e.g., early 3D games)
- **Matcap (1)**: Stylized lighting with matcap textures
- **PBR-lite (2)**: Modern PBR look (most realistic)
- **Hybrid (3)**: Mix matcap and PBR for unique styles

If not specified, defaults to PBR-lite (mode 2).

#### Default Resolution

Suggest a default window size for your game:

```bash
--default-resolution "640x480"   # Retro 4:3
--default-resolution "1280x720"  # HD 16:9
--default-resolution "1920x1080" # Full HD
```

Players can still resize the window, but this sets the initial size.

#### Target FPS

Specify your game's target frame rate:

```bash
--target-fps 60   # Smooth 60fps gameplay
--target-fps 30   # Cinematic 30fps
```

This is a hint to the launcher but doesn't enforce the frame rate.

### Full Example with All Settings

```bash
cargo xtask cart create-z \
  target/wasm32-unknown-unknown/release/my_game.wasm \
  --id my-platformer \
  --title "Super Platformer Adventure" \
  --author "Indie Dev Studio" \
  --version "1.0.0" \
  --description "Jump, run, and explore in this retro-inspired platformer!" \
  --tag platformer \
  --tag action \
  --tag retro \
  --thumbnail assets/thumbnail.png \
  --screenshot assets/screenshot1.png \
  --screenshot assets/screenshot2.png \
  --screenshot assets/screenshot3.png \
  --render-mode 2 \
  --default-resolution "640x480" \
  --target-fps 60 \
  --output super-platformer.ewz
```

## Testing Your ROM

### 1. Inspect the ROM

Before distributing, verify the ROM metadata:

```bash
cargo xtask cart info my-game.ewz
```

This displays all metadata, settings, and asset information.

**Check for:**
- Correct title, author, version
- Description is clear and accurate
- Tags are appropriate
- Console settings match your game
- File sizes are reasonable

### 2. Install Locally

Test the ROM installation process:

```rust
use emberware_core::library::{install_z_rom, DataDirProvider};
use std::path::Path;

// Install ROM (programmatically)
let rom_path = Path::new("my-game.ewz");
let game = install_z_rom(rom_path, &data_dir_provider)?;
```

Or use the launcher's library UI to install and play.

### 3. Verify Installation

Check that the game was installed correctly:

```bash
# Check games directory
ls ~/.emberware/games/my-game/

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
- Upload your `.ewz` file to a file host
- Share the download link
- Players install manually via drag-and-drop or library UI

**2. Platform Upload (Future)**
- Upload to the official Emberware platform
- Players can browse and download from the platform
- Automatic version updates
- Player ratings and reviews

**3. Itch.io / Game Jolt**
- Upload as a downloadable file
- Add installation instructions
- Link to Emberware launcher download

**4. GitHub Releases**
- Tag a release in your game's repository
- Attach the `.ewz` file as a release asset
- Players can download from releases page

### Distribution Checklist

Before distributing:

- ✅ ROM validates successfully (`cargo xtask cart info`)
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

1. Download and install the [Emberware Launcher](https://emberware.io/download)
2. Download the game ROM file: `my-game.ewz`
3. Open the Emberware Launcher
4. Drag and drop `my-game.ewz` onto the launcher window
5. Click the game in your library to play!
```

## Platform Integration

### Platform Foreign Keys

When uploading to the official Emberware platform, the platform will populate foreign keys in your ROM:

```bash
# Platform automatically adds these when you upload
--platform-game-id "uuid-of-game-record"
--platform-author-id "uuid-of-your-profile"
```

These UUIDs enable:
- "Check for updates" functionality
- "View on platform" links
- Download statistics
- Player reviews and ratings

You don't need to set these manually - the platform handles it during upload.

### Complex Credits

The ROM format uses a simple `author` field for offline display. For complex credits:

**In ROM:**
```
author: "Indie Dev Studio"
```

**On Platform:**
- Multiple authors/collaborators
- Specific roles (programmer, artist, composer, sound designer)
- Rich profiles with avatars, bios, social links
- Game pages with full descriptions and media galleries

The platform backend manages the complex credit system, while the ROM keeps it simple for offline play.

## Versioning and Updates

### Semantic Versioning

Use semantic versioning for your ROM files:

```
MAJOR.MINOR.PATCH

1.0.0 - Initial release
1.0.1 - Bug fix
1.1.0 - New feature
2.0.0 - Breaking change
```

**When to increment:**
- **MAJOR**: Breaking changes (incompatible save files, major rewrites)
- **MINOR**: New features, additions (new levels, characters)
- **PATCH**: Bug fixes, small improvements

### Publishing Updates

When you release a new version:

1. **Update version number:**
   ```bash
   --version "1.1.0"
   ```

2. **Create new ROM:**
   ```bash
   cargo xtask cart create-z ... --version "1.1.0" --output my-game-v1.1.0.ewz
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
1. Rebuild your game: `cargo build --target wasm32-unknown-unknown --release`
2. Verify the WASM file exists and is not empty
3. Check for build errors in cargo output

### "Game ID cannot be empty"

**Cause:** Missing or empty `--id` argument

**Fix:**
```bash
--id my-game  # Must be specified
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
```bash
--render-mode 2  # Valid: 0, 1, 2, or 3
```

### ROM installs but game won't launch

**Possible causes:**
1. WASM file has runtime errors (check launcher console)
2. Missing required FFI functions
3. Incompatible with current launcher version

**Fix:**
1. Test game with `cargo run` during development
2. Check launcher version compatibility
3. Review error messages in launcher console

## Best Practices

### Development Workflow

1. **Develop with raw WASM** for fast iteration
2. **Create ROM for testing** before each release
3. **Version consistently** using semantic versioning
4. **Test installation** on a clean setup
5. **Document changes** in update notes

### ROM File Naming

Use descriptive filenames:

```bash
# Good
my-game-v1.0.0.ewz
super-platformer.ewz
puzzle-quest-v2.1.0.ewz

# Avoid
game.ewz
test.ewz
final-final-v2.ewz
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
# 1. Build the game
cd my-platformer
cargo build --target wasm32-unknown-unknown --release

# 2. Create assets (manually or with tools)
# - assets/thumbnail.png (256x256)
# - assets/screenshot1.png (gameplay)
# - assets/screenshot2.png (boss fight)
# - assets/screenshot3.png (level select)

# 3. Create ROM
cargo xtask cart create-z \
  target/wasm32-unknown-unknown/release/my_platformer.wasm \
  --id super-platformer \
  --title "Super Platformer Adventure" \
  --author "Indie Dev Studio" \
  --version "1.0.0" \
  --description "Jump, run, and explore through mysterious worlds filled with challenging platforming and hidden secrets! Featuring 20 unique levels, boss battles, and unlockable characters." \
  --tag platformer \
  --tag action \
  --tag adventure \
  --thumbnail assets/thumbnail.png \
  --screenshot assets/screenshot1.png \
  --screenshot assets/screenshot2.png \
  --screenshot assets/screenshot3.png \
  --render-mode 2 \
  --default-resolution "640x480" \
  --target-fps 60 \
  --output super-platformer-v1.0.0.ewz

# 4. Verify ROM
cargo xtask cart info super-platformer-v1.0.0.ewz

# 5. Test installation
# - Install via launcher UI
# - Verify files in ~/.emberware/games/super-platformer/
# - Launch and play

# 6. Distribute
# - Upload to platform or file host
# - Share download link
# - Add to itch.io/GitHub releases
```

## See Also

- [rom-format.md](./rom-format.md) - Technical ROM format specification
- [ffi.md](./ffi.md) - Emberware FFI API reference
- [emberware-zx.md](./emberware-zx.md) - Emberware ZX console documentation

## Getting Help

If you run into issues:

1. Check this guide and the [rom-format.md](./rom-format.md) specification
2. Use `cargo xtask cart --help` for CLI reference
3. Inspect ROMs with `cargo xtask cart info`
4. Ask in the Emberware Discord/forums
5. Open an issue on GitHub if you find a bug
