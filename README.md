# Emberware

A 5th-generation fantasy console targeting PS1/N64/Saturn aesthetics with built-in rollback netcode.

## What's Here

| Directory | Description |
|-----------|-------------|
| `/emberware-z` | Native game runtime (Rust) |
| `/shared` | Shared types used by Emberware Z and platform |
| `/docs` | FFI documentation for game developers |
| `/examples` | Example games |

## For Game Developers

See [docs/ffi.md](./docs/ffi.md) for the complete FFI API reference.

### Quick Start

```rust
#[no_mangle]
pub extern "C" fn init() {
    // Called once at startup
}

#[no_mangle]
pub extern "C" fn update() {
    // Called every tick — game logic (deterministic!)
}

#[no_mangle]
pub extern "C" fn render() {
    // Called every frame — draw calls (can skip during rollback)
}
```

### Build Your Game

```bash
cargo build --target wasm32-unknown-unknown --release
```

### Upload

Visit [emberware.io](https://emberware.io) to create an account and upload your game.

## Console Specs

| Spec | Value |
|------|-------|
| Resolution | 360p, 540p (default), 720p, 1080p |
| Tick rate | 24, 30, 60 (default), 120 fps |
| RAM | 16MB |
| VRAM | 8MB |
| CPU budget | 4ms per tick (at 60fps) |
| ROM size | 32MB max |
| Netcode | Deterministic rollback via GGRS |

## License

MIT License
