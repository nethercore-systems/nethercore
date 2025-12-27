# Build Paddle

In this tutorial, you'll build a complete Paddle game from scratch. By the end, you'll have a fully playable game with:

- Two paddles (player-controlled or AI)
- Ball physics with collision detection
- Score tracking and win conditions
- Sound effects loaded from assets
- Title screen and game over states
- **Automatic online multiplayer** via Nethercore's rollback netcode

![Paddle game preview](../../assets/paddle-preview.png)

## What You'll Learn

| Part | Topics |
|------|--------|
| [Part 1: Setup & Drawing](./01-setup.md) | Project creation, FFI imports, `draw_rect()` |
| [Part 2: Paddle Movement](./02-paddles.md) | Input handling, game state |
| [Part 3: Ball Physics](./03-ball.md) | Velocity, collision detection |
| [Part 4: AI Opponent](./04-ai.md) | Simple AI for single-player |
| [Part 5: Multiplayer](./05-multiplayer.md) | The magic of rollback netcode |
| [Part 6: Scoring & Win States](./06-scoring.md) | Game logic, state machine |
| [Part 7: Sound Effects](./07-sound.md) | Assets, `nether build`, audio playback |
| [Part 8: Polish & Publishing](./08-polish.md) | Title screen, publishing to archive |

## Prerequisites

Before starting this tutorial, you should have:

- Completed [Your First Game](../../getting-started/first-game.md)
- Rust and WASM target installed ([Prerequisites](../../getting-started/prerequisites.md))
- Basic understanding of the [game loop](../../getting-started/game-loop.md)

## Final Code

The complete source code for this tutorial is available in the examples:

```
nethercore/examples/7-games/paddle/
├── Cargo.toml
├── nether.toml
└── src/
    └── lib.rs
```

You can build and run it with:

```bash
cd examples/7-games/paddle
cargo build --target wasm32-unknown-unknown --release
nether run target/wasm32-unknown-unknown/release/paddle.wasm
```

## Time Investment

Each part takes about 10-15 minutes to complete. The full tutorial can be finished in about 2 hours.

---

**Ready?** Let's start with [Part 1: Setup & Drawing](./01-setup.md).
