# Part 8: Polish & Publishing

Your Paddle game is complete! Let's add some final polish and publish it to the Emberware Archive.

## What You'll Learn

- Adding control hints
- Final ember.toml configuration
- Building a release ROM
- Publishing to emberware.io

## Add Control Hints

Let's add helpful text on the title screen:

```rust
fn render_title() {
    unsafe {
        // Title
        draw_text_bytes(b"PADDLE", SCREEN_WIDTH / 2.0 - 100.0, 150.0, 64.0, COLOR_WHITE);

        // Mode indicator
        if IS_TWO_PLAYER {
            draw_text_bytes(b"2 PLAYER MODE", SCREEN_WIDTH / 2.0 - 100.0, 250.0, 24.0, COLOR_WHITE);
        } else {
            draw_text_bytes(b"1 PLAYER VS AI", SCREEN_WIDTH / 2.0 - 100.0, 250.0, 24.0, COLOR_WHITE);
        }

        // Start prompt
        draw_text_bytes(b"Press A to Start", SCREEN_WIDTH / 2.0 - 120.0, 350.0, 24.0, COLOR_GRAY);

        // Controls hint
        draw_text_bytes(b"Controls: Left Stick or D-Pad Up/Down",
                       250.0, 450.0, 18.0, COLOR_GRAY);
    }
}
```

## Complete ember.toml

Back in [Part 1.5](./01b-assets.md), we created our `ember.toml`. Here's the complete version with all metadata:

```toml
[game]
id = "paddle"
title = "Paddle"
author = "Your Name"
version = "1.0.0"
description = "Classic Paddle game with AI and multiplayer support"

[build]
script = "cargo build --target wasm32-unknown-unknown --release"
wasm = "target/wasm32-unknown-unknown/release/paddle.wasm"

# Texture assets
[[assets.textures]]
id = "paddle"
path = "assets/paddle.png"

[[assets.textures]]
id = "ball"
path = "assets/ball.png"

# Sound assets
[[assets.sounds]]
id = "hit"
path = "assets/hit.wav"

[[assets.sounds]]
id = "score"
path = "assets/score.wav"

[[assets.sounds]]
id = "win"
path = "assets/win.wav"
```

### Project Structure

Your final project should look like:

```
paddle/
├── Cargo.toml
├── ember.toml
├── assets/
│   ├── paddle.png
│   ├── ball.png
│   ├── hit.wav
│   ├── score.wav
│   └── win.wav
└── src/
    └── lib.rs
```

## Build for Release

### Using ember build

Build your game with all assets bundled:

```bash
ember build
```

This creates a `.ewz` ROM file containing:
- Your compiled WASM code
- All converted and compressed assets
- Game metadata

### Verify the Build

Check your ROM was created:

```bash
ls -la *.ewz
```

You should see something like:
```
-rw-r--r-- 1 user user 45678 Dec 20 12:00 paddle.ewz
```

## Test Your Release Build

Run the final ROM:

```bash
ember run paddle.ewz
```

## Final Checklist

Before publishing, verify:

- [ ] Title screen displays correctly
- [ ] Both players can control paddles
- [ ] AI works when only one player
- [ ] Ball bounces correctly off walls and paddles
- [ ] Scores track correctly
- [ ] Game ends at 5 points
- [ ] Victory screen shows correct winner
- [ ] All sound effects play with proper panning
- [ ] Game restarts correctly

## Publishing to Emberware Archive

### 1. Create an Account

Visit [emberware.io/register](https://emberware.io/register) to create your developer account.

### 2. Prepare Assets

You'll need:
- **Icon** (64×64 PNG) — Shows in the game library
- **Screenshot(s)** (optional) — Shows on your game's page

### 3. Upload Your Game

1. Log in to [emberware.io](https://emberware.io)
2. Go to your [Dashboard](https://emberware.io/dashboard)
3. Click "Upload New Game"
4. Fill in the details:
   - Title: "Paddle"
   - Description: Your game description
   - Category: Arcade
5. Upload your `.ewz` ROM file
6. Add your icon and screenshots
7. Click "Publish"

### 4. Share Your Game

Once published, your game has a unique page at:
```
emberware.io/game/paddle
```

Share this link! Anyone with the Emberware player can play your game.

## What You've Built

Congratulations! Your Paddle game includes:

| Feature | Implementation |
|---------|---------------|
| **Graphics** | Court, paddles, ball with `draw_rect()` |
| **Input** | Analog stick and D-pad with `left_stick_y()`, `button_held()` |
| **Physics** | Ball movement, wall bouncing, paddle collision |
| **AI** | Simple ball-following AI opponent |
| **Multiplayer** | Automatic online play via rollback netcode |
| **Game Flow** | Title, Playing, GameOver states |
| **Scoring** | Point tracking, win conditions |
| **Audio** | Sound effects loaded from ROM with stereo panning |
| **Assets** | Textures and sounds bundled with `ember build` |

## What's Next?

### Enhance Your Paddle Game

Ideas to try:
- Add ball speed increase after each hit
- Create power-ups that spawn randomly
- Add particle effects when scoring
- Implement 4-player mode
- Use sprite textures for paddles and ball

### Build More Games

Check out these resources:
- **[Example Games](https://github.com/emberware/emberware/tree/main/examples)** — 28+ examples
- **[API Reference](../../cheat-sheet.md)** — All available functions
- **[Asset Pipeline](../../guides/asset-pipeline.md)** — Advanced asset workflows
- **[Render Modes Guide](../../guides/render-modes.md)** — 3D graphics

### Join the Community

- Share your game in [GitHub Discussions](https://github.com/emberware/emberware/discussions)
- Report bugs or request features
- Help other developers

## Complete Source Code

The final source code is available at:
```
emberware/examples/paddle/
```

You can compare your code or use it as a reference.

---

## Summary

In this tutorial, you learned:

1. **Setup** — Creating an Emberware project
2. **Assets** — Using `ember.toml` and `ember build` for textures and sounds
3. **Drawing** — Using `draw_rect()` for 2D graphics
4. **Input** — Reading sticks and buttons
5. **Physics** — Ball movement and collision
6. **AI** — Simple opponent behavior
7. **Multiplayer** — How rollback netcode "just works"
8. **Game Flow** — State machines for menus
9. **Audio** — Loading and playing sound effects from ROM
10. **Publishing** — Sharing your game with the world

**You're now an Emberware game developer!**
