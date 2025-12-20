# Prerequisites

Before you start building games for Emberware ZX, you'll need to set up your development environment.

## Required Tools

### 1. Rust

Emberware games are written in Rust and compiled to WebAssembly. Install Rust using rustup:

**Windows/macOS/Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Or visit [rustup.rs](https://rustup.rs/) for platform-specific installers.

### 2. WebAssembly Target

After installing Rust, add the WASM compilation target:

```bash
rustup target add wasm32-unknown-unknown
```

### 3. Code Editor (Optional but Recommended)

Any text editor works, but we recommend one with Rust support:

- **VS Code** with rust-analyzer extension
- **RustRover** (JetBrains IDE for Rust)
- **Neovim** with rust-analyzer LSP

## Verify Installation

Check that everything is installed correctly:

```bash
# Check Rust version
rustc --version

# Check Cargo version
cargo --version

# Check WASM target is installed
rustup target list --installed | grep wasm32
```

You should see output similar to:
```
rustc 1.75.0 (82e1608df 2023-12-21)
cargo 1.75.0 (1d8b05cdd 2023-11-20)
wasm32-unknown-unknown
```

## Optional: Emberware CLI

The `ember` CLI tool provides convenient commands for building and running games:

```bash
cargo install --path tools/ember-cli
```

This gives you commands like:
- `ember build` - Compile your game
- `ember run` - Run your game in the player
- `ember pack` - Package your game into a ROM file

---

**Next:** [Your First Game](./first-game.md)
