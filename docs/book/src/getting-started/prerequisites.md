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

**Windows:** download and run the Windows installer at [rustup.rs](https://rustup.rs/).
Install its requested Visual Studio C++ build tools, then open a fresh PowerShell window.
The commands below assume the Rust route on Windows; other language/toolchain routes
are separate and are not implied by that validation.

**macOS/Linux:**
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
rustup target list --installed
```

You should see:
- a `rustc ...` version line
- `wasm32-unknown-unknown` in the installed target list

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
Target: wasm32-unknown-wasi
```

{{#endtab}}

{{#tab name="Zig"}}

### Install Zig

The binding compile job in `.github/workflows/ffi-check.yml` pins **Zig 0.11.0**.
Use that version for the matching bindings, rather than an unpinned package-manager
release. This is a binding-compatibility baseline; the selected end-to-end
onboarding route in this guide is Windows/Rust, not a claim that every Zig
version or platform has been exercised.

Download the 0.11.0 archive for your OS from
[ziglang.org/download](https://ziglang.org/download/). Linux example:
```bash
# Extract and add to PATH
wget https://ziglang.org/download/0.11.0/zig-linux-x86_64-0.11.0.tar.xz
tar xf zig-linux-x86_64-0.11.0.tar.xz
export PATH=$PATH:$(pwd)/zig-linux-x86_64-0.11.0
```

### Get the Nethercore Bindings

Copy the native Zig bindings from the Nethercore repository:
```bash
cp include/zx.zig your-game/
```

Use the version-matched bindings; avoid copying signatures from older tutorials.

### Verify Installation

```bash
zig version
```

You should see:
```
0.11.0
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

## Install the SDK and player

Extract a version-matched Nethercore distribution to a directory of your choice.
Keep `nethercore.exe`, `nethercore-zx.exe`, `nether.exe` and `include/` together.
The library and CLI discover the ZX executable beside themselves; a CLI alone is
not a complete player installation. Do not mix bindings and executables from
different builds.

In PowerShell, set the directory you extracted (change the example path):

```powershell
$env:NETHERCORE_HOME = "$HOME\Nethercore"
$env:PATH = "$env:NETHERCORE_HOME;$env:PATH"
nether --help
nethercore-zx --help
```

These environment changes apply to this terminal only. The Rust toolchain above
is needed for development, not for launching a shared cartridge.

### Building the CLI from source instead

From the runtime checkout (this installs the CLI, not the standalone player):

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
