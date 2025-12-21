# Prerequisites

Before you start building games for Nethercore ZX, you'll need to set up your development environment.

## Choose Your Language

Nethercore ZX games are compiled to WebAssembly. You can write games in several languages:

| Language | Best For |
|----------|----------|
| **Rust** | Full ecosystem support, best tooling |
| **C/C++** | Existing codebases, familiar to game devs |
| **Zig** | Modern systems programming, C interop |

This guide shows setup for each language. Pick one and follow its setup instructions.

## Language Setup

{{#tabs global="lang"}}

{{#tab name="Rust"}}

### Install Rust

Install Rust using rustup:

**Windows/macOS/Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Or visit [rustup.rs](https://rustup.rs/) for platform-specific installers.

### Add WebAssembly Target

After installing Rust, add the WASM compilation target:

```bash
rustup target add wasm32-unknown-unknown
```

### Verify Installation

```bash
# Check Rust version
rustc --version

# Check WASM target is installed
rustup target list --installed | grep wasm32
```

You should see:
```
rustc 1.75.0 (82e1608df 2023-12-21)
wasm32-unknown-unknown
```

{{#endtab}}

{{#tab name="C/C++"}}

### Install wasi-sdk

wasi-sdk provides a clang toolchain configured for WebAssembly:

**macOS (Homebrew):**
```bash
brew install wasi-sdk
```

**Linux/Windows:**

Download from [wasi-sdk releases](https://github.com/WebAssembly/wasi-sdk/releases):
```bash
# Linux example (adjust version as needed)
wget https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-24/wasi-sdk-24.0-x86_64-linux.tar.gz
tar xzf wasi-sdk-24.0-x86_64-linux.tar.gz
sudo mv wasi-sdk-24.0 /opt/wasi-sdk
```

**Windows (manual):**
1. Download the Windows release from GitHub
2. Extract to `C:\wasi-sdk`
3. Add to PATH or set `WASI_SDK_PATH` environment variable

### Get the Nethercore Header

Download `zx.h` from the Nethercore repository:
```bash
# From the nethercore repo
cp include/zx.h your-game/
```

Or add the include path to your build.

### Verify Installation

```bash
# Check clang version
/opt/wasi-sdk/bin/clang --version
```

You should see:
```
clang version 18.1.2 (https://github.com/aspect-build/llvm-project ...)
Target: wasm32-unknown-wasi
```

{{#endtab}}

{{#tab name="Zig"}}

### Install Zig

**macOS (Homebrew):**
```bash
brew install zig
```

**Linux/Windows:**

Download from [ziglang.org/download](https://ziglang.org/download/):
```bash
# Extract and add to PATH
wget https://ziglang.org/download/0.13.0/zig-linux-x86_64-0.13.0.tar.xz
tar xf zig-linux-x86_64-0.13.0.tar.xz
export PATH=$PATH:$(pwd)/zig-linux-x86_64-0.13.0
```

### Get the Nethercore Bindings

Copy the native Zig bindings from the Nethercore repository:
```bash
cp include/zx.zig your-game/
```

Or declare the FFI imports directly in your code (see examples).

### Verify Installation

```bash
zig version
```

You should see:
```
0.13.0
```

{{#endtab}}

{{#endtabs}}

## Code Editor (Optional but Recommended)

Any text editor works, but we recommend one with language support:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
- **VS Code** with rust-analyzer extension
- **RustRover** (JetBrains IDE for Rust)
- **Neovim** with rust-analyzer LSP
{{#endtab}}

{{#tab name="C/C++"}}
- **VS Code** with C/C++ extension (clangd)
- **CLion** (JetBrains IDE)
- **Neovim** with clangd LSP
{{#endtab}}

{{#tab name="Zig"}}
- **VS Code** with Zig Language extension
- **Neovim** with zls (Zig Language Server)
{{#endtab}}

{{#endtabs}}

## Optional: Nethercore CLI

The `nether` CLI tool provides convenient commands for building and running games:

```bash
cargo install --path tools/nether-cli
```

This gives you commands like:
- `nether build` - Compile your game
- `nether run` - Run your game in the player
- `nether pack` - Package your game into a ROM file

---

**Next:** [Your First Game](./first-game.md)
