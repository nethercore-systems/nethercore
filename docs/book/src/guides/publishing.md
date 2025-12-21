# Publishing Your Game

This guide covers everything you need to know about packaging and publishing your Nethercore game.

## Overview

The publishing process:

1. **Build** your game (compile to WASM)
2. **Pack** assets into a ROM file (optional)
3. **Test** the final build
4. **Upload** to nethercore.systems
5. **Share** with the world

## Building for Release

### Using nether-cli (Recommended)

The nether CLI handles compilation and packaging:

```bash
# Build WASM
nether build

# Package into ROM
nether pack

# Or do both
nether build && nether pack
```

### Manual Build

If you prefer manual control:

```bash
# Build optimized WASM
cargo build --target wasm32-unknown-unknown --release

# Your WASM file
ls target/wasm32-unknown-unknown/release/your_game.wasm
```

## Game Manifest (nether.toml)

Create `nether.toml` in your project root:

```toml
[game]
id = "my-game"              # Unique identifier (lowercase, hyphens ok)
title = "My Game"           # Display name
author = "Your Name"        # Creator credit
version = "1.0.0"           # Semantic version
description = "A fun game"  # Short description

[build]
script = "cargo build --target wasm32-unknown-unknown --release"
wasm = "target/wasm32-unknown-unknown/release/my_game.wasm"

# Optional: Assets to include in ROM
[[assets.textures]]
id = "player"
path = "assets/player.png"

[[assets.meshes]]
id = "level"
path = "assets/level.ewzmesh"

[[assets.sounds]]
id = "jump"
path = "assets/jump.wav"
```

## ROM File Format

Nethercore ROMs (`.nczx` files) bundle:

- Your compiled WASM game
- Pre-processed assets (textures, meshes, sounds)
- Game metadata

Benefits of ROM packaging:
- **Faster loading** - Assets are already GPU-ready
- **Single file** - Easy to distribute
- **Verified** - Content integrity checked

## Testing Your Build

Always test the final build:

```bash
# Test the WASM directly
nether run target/wasm32-unknown-unknown/release/my_game.wasm

# Or test the packed ROM
nether run my_game.nczx
```

Verify:
- Game starts correctly
- All assets load
- No console errors
- Multiplayer works (test with two controllers)

## Upload Requirements

### Required Files

| File | Format | Description |
|------|--------|-------------|
| Game | `.wasm` or `.nczx` | Your compiled game |
| Icon | 64×64 PNG | Library thumbnail |

### Optional Files

| File | Format | Description |
|------|--------|-------------|
| Screenshots | PNG | Game page gallery (up to 5) |
| Banner | 1280×720 PNG | Featured games banner |

### Metadata

- **Title** - Your game's name
- **Description** - What your game is about (Markdown supported)
- **Category** - Arcade, Puzzle, Action, etc.
- **Tags** - Searchable keywords

## Publishing Process

### 1. Create Developer Account

Visit [nethercore.systems/register](https://nethercore.systems/register)

### 2. Access Dashboard

Log in and go to [nethercore.systems/dashboard](https://nethercore.systems/dashboard)

### 3. Upload Game

1. Click "Upload New Game"
2. Fill in title and description
3. Select category and tags
4. Upload your game file
5. Upload icon (required) and screenshots (optional)
6. Click "Publish"

### 4. Game Page

Your game gets a public page:
```
nethercore.systems/game/your-game-id
```

## Updating Your Game

To release an update:

1. Bump version in `nether.toml`
2. Build and test new version
3. Go to Dashboard → Your Game → Edit
4. Upload new game file
5. Update version number
6. Save changes

Players with old versions will be prompted to update.

## Content Guidelines

Games must:
- Be appropriate for all ages
- Not contain malware or harmful code
- Not violate copyright
- Actually be playable

Games must NOT:
- Contain excessive violence or adult content
- Harvest user data
- Attempt to break out of the sandbox
- Impersonate other developers' games

## Troubleshooting

### "WASM validation failed"

Your WASM file may be corrupted or built incorrectly.

Fix:
```bash
# Clean build
cargo clean
cargo build --target wasm32-unknown-unknown --release
```

### "Asset not found"

Asset paths in `nether.toml` are relative to the project root.

Verify:
```bash
# Check if file exists
ls assets/player.png
```

### "ROM too large"

Nethercore has size limits for fair distribution.

Reduce size:
- Compress textures
- Use smaller audio sample rates
- Remove unused assets

### "Game crashes on load"

Usually a panic in `init()`.

Debug:
1. Test locally first
2. Check console for error messages
3. Simplify `init()` to isolate the issue

## Best Practices

1. **Test thoroughly** before publishing
2. **Write a good description** - help players find your game
3. **Create an appealing icon** - first impressions matter
4. **Include screenshots** - show off your game
5. **Respond to feedback** - engage with players
6. **Update regularly** - fix bugs, add features

## Distribution Alternatives

Besides nethercore.systems, you can distribute:

### Direct Download
Share the `.wasm` or `.nczx` file directly. Players load it in the Nethercore player.

### GitHub Releases
Host on GitHub as release artifacts.

### itch.io
Upload as a downloadable file with instructions.

---

Ready to publish? Head to [nethercore.systems](https://nethercore.systems) and share your creation with the world!
