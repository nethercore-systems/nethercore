# Replay Screenshot Support — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add `screenshot` support to `.ncrs` replay scripts and integrate script execution into the graphical player, enabling automated screenshot capture of all 24 EPU showcase presets in one run.

**Architecture:** Extend the replay script format with a `screenshot` flag per frame. Add a `--replay` CLI flag to the ZX player binary. The graphical player loads and executes the script frame-by-frame, overriding inputs from the script and triggering `ScreenCapture` when `screenshot = true`. A new `Console::decode_replay_bytes()` trait method bridges console-specific replay byte encoding to the generic player pipeline. The player auto-exits when the script completes.

**Tech Stack:** Rust, TOML (serde), wgpu (screenshot capture), clap (CLI)

---

### Task 1: Add `screenshot` field to replay script AST

**Files:**
- Modify: `core/src/replay/script/ast.rs:30-65`

**Step 1: Add field to FrameEntry**

In `FrameEntry`, add after the `snap` field (line 52):

```rust
    /// Request a screenshot capture after rendering this frame
    #[serde(default)]
    pub screenshot: bool,
```

**Step 2: Run tests to verify no breakage**

Run: `cargo test -p nethercore-core --lib replay`
Expected: Some tests FAIL because FrameEntry constructors in tests are missing the new field.

**Step 3: Fix test constructors**

Add `screenshot: false` to every `FrameEntry` literal in:
- `core/src/replay/script/compiler.rs` (tests at bottom: ~lines 320, 331, 337, 380, 392, 434, 444, 484)
- `core/src/replay/script/decompiler.rs:36` (add field to decompile output)

**Step 4: Run tests to verify they pass**

Run: `cargo test -p nethercore-core --lib replay`
Expected: PASS

**Step 5: Commit**

```bash
git add core/src/replay/script/ast.rs core/src/replay/script/compiler.rs core/src/replay/script/decompiler.rs
git commit -m "replay: add screenshot field to FrameEntry AST"
```

---

### Task 2: Compile screenshot frames in the script compiler

**Files:**
- Modify: `core/src/replay/script/compiler.rs:47-68` (CompiledScript struct)
- Modify: `core/src/replay/script/compiler.rs:119-216` (compile method)

**Step 1: Write test**

Add to the `tests` module in `core/src/replay/script/compiler.rs`:

```rust
    #[test]
    fn test_compile_with_screenshots() {
        let script = ReplayScript {
            console: "zx".to_string(),
            seed: 0,
            players: 1,
            frames: vec![
                FrameEntry {
                    f: 0,
                    p1: None,
                    p2: None,
                    p3: None,
                    p4: None,
                    snap: false,
                    screenshot: true,
                    assert: None,
                    action: None,
                    action_params: None,
                },
                FrameEntry {
                    f: 10,
                    p1: Some(InputValue::Symbolic("a".to_string())),
                    p2: None,
                    p3: None,
                    p4: None,
                    snap: false,
                    screenshot: true,
                    assert: None,
                    action: None,
                    action_params: None,
                },
            ],
        };

        let layout = MockLayout;
        let compiler = Compiler::new(&layout);
        let compiled = compiler.compile(&script).unwrap();

        assert_eq!(compiled.screenshot_frames, vec![0, 10]);
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p nethercore-core --lib replay::script::compiler::tests::test_compile_with_screenshots`
Expected: FAIL — `screenshot_frames` field doesn't exist on `CompiledScript`.

**Step 3: Add field to CompiledScript and compile it**

In `CompiledScript` struct (line ~61), add after `snap_frames`:

```rust
    /// Frames that need screenshot capture
    pub screenshot_frames: Vec<u64>,
```

In `Compiler::compile()`, add alongside `snap_frames` collection:

After `let mut snap_frames = Vec::new();` (~line 125), add:
```rust
        let mut screenshot_frames = Vec::new();
```

Inside the `for entry in &script.frames` loop, after the `if entry.snap` block (~line 143), add:
```rust
            if entry.screenshot {
                screenshot_frames.push(entry.f);
            }
```

In the `Ok(CompiledScript { ... })` return, add after `snap_frames,`:
```rust
            screenshot_frames,
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p nethercore-core --lib replay::script::compiler::tests::test_compile_with_screenshots`
Expected: PASS

**Step 5: Commit**

```bash
git add core/src/replay/script/compiler.rs
git commit -m "replay: compile screenshot frames from script"
```

---

### Task 3: Add `needs_screenshot()` to ScriptExecutor

**Files:**
- Modify: `core/src/replay/runtime/executor/mod.rs:68-105`

**Step 1: Add method**

After `needs_snapshot()` (~line 105), add:

```rust
    /// Check if current frame needs a screenshot
    pub fn needs_screenshot(&self) -> bool {
        self.script.screenshot_frames.contains(&self.current_frame)
    }
```

**Step 2: Run tests**

Run: `cargo test -p nethercore-core --lib replay`
Expected: PASS

**Step 3: Commit**

```bash
git add core/src/replay/runtime/executor/mod.rs
git commit -m "replay: add needs_screenshot() to ScriptExecutor"
```

---

### Task 4: Add `decode_replay_bytes` to Console trait

**Files:**
- Modify: `core/src/console.rs:177` (Console trait)
- Modify: `core/src/test_utils.rs` (test console impl)
- Modify: `nethercore-zx/src/console.rs` (NethercoreZX impl)

**Step 1: Add trait method**

In the `Console` trait in `core/src/console.rs`, add after `map_input`:

```rust
    /// Decode replay script raw bytes into console input.
    ///
    /// Used by the graphical replay runner to convert script-encoded
    /// input bytes into the console's native input type.
    fn decode_replay_bytes(&self, bytes: &[u8]) -> Self::Input;
```

**Step 2: Run build to see what fails**

Run: `cargo build -p nethercore-core`
Expected: FAIL — test_utils::TestConsole doesn't implement the new method.

**Step 3: Implement for TestConsole**

In `core/src/test_utils.rs`, in the `impl Console for TestConsole` block (near `map_input`), add:

```rust
    fn decode_replay_bytes(&self, bytes: &[u8]) -> Self::Input {
        let mut buttons = 0u16;
        if !bytes.is_empty() {
            buttons = bytes[0] as u16;
        }
        buttons
    }
```

**Step 4: Implement for NethercoreZX**

In `nethercore-zx/src/console.rs`, in the `impl Console for NethercoreZX` block, add:

```rust
    fn decode_replay_bytes(&self, bytes: &[u8]) -> Self::Input {
        crate::replay::ZxInputLayout::bytes_to_zinput(bytes)
    }
```

**Step 5: Run build**

Run: `cargo build`
Expected: PASS

**Step 6: Commit**

```bash
git add core/src/console.rs core/src/test_utils.rs nethercore-zx/src/console.rs
git commit -m "console: add decode_replay_bytes trait method"
```

---

### Task 5: Add `--replay` CLI flag and `replay_script` to StandaloneConfig

**Files:**
- Modify: `core/src/app/player/types.rs:63-78` (StandaloneConfig)
- Modify: `nethercore-zx/src/bin/main.rs:39-110` (Args struct)
- Modify: `nethercore-zx/src/bin/main.rs:164-174` (config construction)

**Step 1: Add field to StandaloneConfig**

In `core/src/app/player/types.rs`, add to `StandaloneConfig` after `connection_mode`:

```rust
    /// Replay script path (.ncrs file) for automated playback
    pub replay_script: Option<PathBuf>,
```

**Step 2: Fix compilation errors**

Every place that constructs `StandaloneConfig` or `PlayerConfig` must add `replay_script: None`. Find them:

Run: `cargo build 2>&1` — look for missing field errors.

In `nethercore-zx/src/bin/main.rs`, the config construction (~line 164) needs `replay_script` added (will be wired in step 3).

**Step 3: Add CLI arg and wire to config**

In `nethercore-zx/src/bin/main.rs`, add to `Args` struct after the preview section:

```rust
    // === Replay Mode ===
    /// Run a replay script (.ncrs) for automated playback and screenshots
    #[arg(long, value_name = "FILE")]
    replay: Option<PathBuf>,
```

In the config construction block (~line 164), add:
```rust
        replay_script: args.replay,
```

**Step 4: Run build**

Run: `cargo build -p nethercore-zx`
Expected: PASS

**Step 5: Commit**

```bash
git add core/src/app/player/types.rs nethercore-zx/src/bin/main.rs
git commit -m "player: add --replay CLI flag and replay_script config"
```

---

### Task 6: Integrate replay executor into StandaloneApp

**Files:**
- Modify: `core/src/app/player/mod.rs:1-81` (StandaloneApp struct and new())
- Modify: `core/src/app/player/trait_impl.rs:66-120` (advance_simulation)
- Modify: `core/src/app/player/lifecycle.rs:86-213` (run_game_frame)

This is the largest task. It wires up the ScriptExecutor to drive inputs and screenshots.

**Step 1: Add replay state to StandaloneApp**

In `core/src/app/player/mod.rs`, add import at top:

```rust
use crate::replay::script::{CompiledScript, InputLayout};
use crate::replay::runtime::executor::ScriptExecutor;
```

Add fields to `StandaloneApp` struct (after `_loader_marker`):

```rust
    /// Active replay script executor (when --replay is used)
    replay_executor: Option<ScriptExecutor>,
```

In `StandaloneApp::new()`, initialize:

```rust
            replay_executor: None,
```

**Step 2: Load replay script on init**

In `core/src/app/player/init.rs`, after the game is loaded and runner is created, if `config.replay_script` is set, load and compile the script. Find the appropriate init point where the game is already running.

Actually, the script needs to be loaded after the runner is set up because we need an `InputLayout` to compile the script. The `InputLayout` is console-specific (`ZxInputLayout`). Since `StandaloneApp` is generic over `C: Console`, we need a way to get an `InputLayout` from the console.

Add to `Console` trait in `core/src/console.rs`:

```rust
    /// Get the input layout for replay script compilation.
    /// Returns None if the console doesn't support replay scripts.
    fn replay_input_layout(&self) -> Option<Box<dyn crate::replay::script::InputLayout>> {
        None
    }
```

Implement in `NethercoreZX`:

```rust
    fn replay_input_layout(&self) -> Option<Box<dyn nethercore_core::replay::script::InputLayout>> {
        Some(Box::new(crate::replay::ZxInputLayout))
    }
```

And in `TestConsole` (return `None`).

Then in the init path (after runner is set up), load the script:

```rust
if let Some(ref script_path) = self.config.replay_script {
    match crate::replay::script::ReplayScript::from_file(script_path) {
        Ok(script) => {
            if let Some(session) = self.runner.as_ref().and_then(|r| r.session()) {
                if let Some(layout) = session.runtime.console().replay_input_layout() {
                    match crate::replay::script::Compiler::new(layout.as_ref()).compile(&script) {
                        Ok(compiled) => {
                            tracing::info!(
                                "Replay script loaded: {} frames, {} screenshots",
                                compiled.frame_count,
                                compiled.screenshot_frames.len()
                            );
                            self.replay_executor = Some(ScriptExecutor::new(compiled));
                        }
                        Err(e) => tracing::error!("Failed to compile replay script: {}", e),
                    }
                } else {
                    tracing::error!("Console does not support replay scripts");
                }
            }
        }
        Err(e) => tracing::error!("Failed to load replay script: {}", e),
    }
}
```

**Step 3: Override inputs in run_game_frame when replay is active**

In `core/src/app/player/lifecycle.rs`, modify `run_game_frame()`. Replace the input section (~lines 111-131) with:

```rust
        // Get inputs: from replay script or from input manager
        if let Some(ref executor) = self.replay_executor {
            // Replay mode: use script inputs
            if let Some(frame_inputs) = executor.current_inputs() {
                let console = session.runtime.console().clone();
                for (player_idx, bytes) in frame_inputs.iter().enumerate() {
                    let console_input = console.decode_replay_bytes(bytes);
                    if let Some(game) = session.runtime.game_mut() {
                        game.set_input(player_idx, console_input.clone());
                    }
                    if let Err(e) = session.runtime.add_local_input(player_idx, console_input) {
                        tracing::error!("Failed to add replay input for player {}: {:?}", player_idx, e);
                    }
                }
            }
        } else {
            // Normal mode: use input manager
            let all_inputs = self.input_manager.get_all_inputs();
            for &player_handle in local_players.iter() {
                let raw_input = all_inputs[player_handle];
                let console_input = session.runtime.console().map_input(&raw_input);
                if let Some(game) = session.runtime.game_mut() {
                    game.set_input(player_handle, console_input.clone());
                }
                if let Err(e) = session.runtime.add_local_input(player_handle, console_input) {
                    tracing::error!("Failed to add local input for handle {}: {:?}", player_handle, e);
                }
            }
        }
```

Note: The `clone()` on console_input is needed because `set_input` and `add_local_input` both consume/borrow it. Check if `C::Input: Clone` is already required — it should be since `Console: Clone`.

**Step 4: Advance replay and handle screenshots in advance_simulation**

In `core/src/app/player/trait_impl.rs`, modify `advance_simulation()` (~line 66). After the `match self.run_game_frame()` block, add:

```rust
        // Advance replay executor and request screenshots
        if let Some(ref mut executor) = self.replay_executor {
            if executor.needs_screenshot() {
                self.capture.request_screenshot();
            }
            executor.advance_frame();
            if executor.is_complete() {
                tracing::info!("Replay script complete");
                self.should_exit = true;
            }
        }
```

Place this AFTER `run_game_frame()` succeeds and `execute_draw_commands()` runs, but BEFORE the error handling. Specifically, inside the `Ok((game_running, did_render))` arm, after `execute_draw_commands()`:

```rust
            Ok((game_running, did_render)) => {
                self.last_sim_rendered = did_render;

                if did_render {
                    self.execute_draw_commands();
                }

                // Advance replay executor and request screenshots
                if did_render {
                    if let Some(ref mut executor) = self.replay_executor {
                        if executor.needs_screenshot() {
                            self.capture.request_screenshot();
                        }
                        executor.advance_frame();
                        if executor.is_complete() {
                            tracing::info!("Replay script complete");
                            self.should_exit = true;
                        }
                    }
                }

                if !game_running {
                    tracing::info!("Game requested quit");
                    self.should_exit = true;
                }
            }
```

**Step 5: Run build**

Run: `cargo build`
Expected: PASS (fix any compilation issues)

**Step 6: Commit**

```bash
git add core/src/app/player/mod.rs core/src/app/player/trait_impl.rs core/src/app/player/lifecycle.rs core/src/app/player/init.rs core/src/console.rs core/src/test_utils.rs nethercore-zx/src/console.rs
git commit -m "player: integrate replay script executor with graphical player"
```

---

### Task 7: Wait for pending screenshots before exiting

**Files:**
- Modify: `core/src/app/player/trait_impl.rs`

The `ScreenCapture` saves screenshots in background threads. If the player exits immediately when the script completes, pending screenshots may be lost. We need to wait for all saves to complete.

**Step 1: Add drain logic**

In the replay completion block (from Task 6 step 4), instead of immediately setting `should_exit`, check if captures are still pending:

```rust
                        if executor.is_complete() {
                            // Don't exit yet if screenshots are still being saved
                            if !self.capture.has_pending_saves() {
                                tracing::info!("Replay complete, all screenshots saved");
                                self.should_exit = true;
                            }
                        }
```

Check if `has_pending_saves()` exists on `ScreenCapture`. If not, add it to `core/src/capture.rs`:

```rust
    /// Check if there are screenshots/GIFs still being saved in background threads
    pub fn has_pending_saves(&self) -> bool {
        // Check if any save operations are in progress
        self.pending_save.is_some()
    }
```

Look at the `ScreenCapture` struct to find the correct field name for pending operations.

**Step 2: Run build and test**

Run: `cargo build`
Expected: PASS

**Step 3: Commit**

```bash
git add core/src/capture.rs core/src/app/player/trait_impl.rs
git commit -m "player: wait for pending screenshots before replay exit"
```

---

### Task 8: Pass `--replay` through library command builder (optional, for library UI launching)

**Files:**
- Modify: `library/src/registry/launcher.rs:29-42` (PlayerOptions)
- Modify: `library/src/registry/player.rs:79-151` (build_cargo_run_player_command)
- Modify: `library/src/registry/player.rs:154-203` (build_player_command)
- Modify: `library/src/main.rs:186-204` (parse_player_options)

**Step 1: Add field to PlayerOptions**

```rust
    /// Replay script path (.ncrs file)
    pub replay_script: Option<PathBuf>,
```

**Step 2: Pass in command builders**

In both `build_player_command` and `build_cargo_run_player_command`, add after the preview args:

```rust
    if let Some(ref replay) = options.replay_script {
        cmd.arg("--replay");
        cmd.arg(replay);
    }
```

**Step 3: Parse from CLI**

In `library/src/main.rs` `parse_player_options`, add:

```rust
            "--replay" => {
                if let Some(path) = iter.next() {
                    options.replay_script = Some(PathBuf::from(path));
                }
            }
```

**Step 4: Build and test**

Run: `cargo build -p nethercore-library`
Expected: PASS

**Step 5: Commit**

```bash
git add library/src/registry/launcher.rs library/src/registry/player.rs library/src/main.rs
git commit -m "library: pass --replay through command builder"
```

---

### Task 9: Create EPU showcase screenshot replay script

**Files:**
- Create: `examples/3-inspectors/epu-showcase/screenshot-all.ncrs`

**Step 1: Create the script**

The EPU showcase starts on preset 0. Pressing A advances to the next preset. We need 10 frames per preset (to let the scene settle), then a screenshot. The A button press is a single-frame event (button_pressed), so it needs to be pressed on one frame and released on the next.

```toml
console = "zx"
seed = 0
players = 1

# Preset 1 (index 0) — starts here
[[frames]]
f = 9
screenshot = true

# Advance to preset 2
[[frames]]
f = 10
p1 = "a"

[[frames]]
f = 20
screenshot = true

# Advance to preset 3
[[frames]]
f = 21
p1 = "a"

[[frames]]
f = 31
screenshot = true

# Advance to preset 4
[[frames]]
f = 32
p1 = "a"

[[frames]]
f = 42
screenshot = true

# Advance to preset 5
[[frames]]
f = 43
p1 = "a"

[[frames]]
f = 53
screenshot = true

# Advance to preset 6
[[frames]]
f = 54
p1 = "a"

[[frames]]
f = 64
screenshot = true

# Advance to preset 7
[[frames]]
f = 65
p1 = "a"

[[frames]]
f = 75
screenshot = true

# Advance to preset 8
[[frames]]
f = 76
p1 = "a"

[[frames]]
f = 86
screenshot = true

# Advance to preset 9
[[frames]]
f = 87
p1 = "a"

[[frames]]
f = 97
screenshot = true

# Advance to preset 10
[[frames]]
f = 98
p1 = "a"

[[frames]]
f = 108
screenshot = true

# Advance to preset 11
[[frames]]
f = 109
p1 = "a"

[[frames]]
f = 119
screenshot = true

# Advance to preset 12
[[frames]]
f = 120
p1 = "a"

[[frames]]
f = 130
screenshot = true

# Advance to preset 13
[[frames]]
f = 131
p1 = "a"

[[frames]]
f = 141
screenshot = true

# Advance to preset 14
[[frames]]
f = 142
p1 = "a"

[[frames]]
f = 152
screenshot = true

# Advance to preset 15
[[frames]]
f = 153
p1 = "a"

[[frames]]
f = 163
screenshot = true

# Advance to preset 16
[[frames]]
f = 164
p1 = "a"

[[frames]]
f = 174
screenshot = true

# Advance to preset 17
[[frames]]
f = 175
p1 = "a"

[[frames]]
f = 185
screenshot = true

# Advance to preset 18
[[frames]]
f = 186
p1 = "a"

[[frames]]
f = 196
screenshot = true

# Advance to preset 19
[[frames]]
f = 197
p1 = "a"

[[frames]]
f = 207
screenshot = true

# Advance to preset 20
[[frames]]
f = 208
p1 = "a"

[[frames]]
f = 218
screenshot = true

# Advance to preset 21
[[frames]]
f = 219
p1 = "a"

[[frames]]
f = 229
screenshot = true

# Advance to preset 22
[[frames]]
f = 230
p1 = "a"

[[frames]]
f = 240
screenshot = true

# Advance to preset 23
[[frames]]
f = 241
p1 = "a"

[[frames]]
f = 251
screenshot = true

# Advance to preset 24
[[frames]]
f = 252
p1 = "a"

[[frames]]
f = 262
screenshot = true
```

**Step 2: Commit**

```bash
git add examples/3-inspectors/epu-showcase/screenshot-all.ncrs
git commit -m "epu-showcase: add screenshot-all replay script for 24 presets"
```

---

### Task 10: End-to-end test

**Step 1: Build the EPU showcase WASM**

Run:
```bash
cd examples/3-inspectors/epu-showcase
cargo build --release --target wasm32-unknown-unknown
```

**Step 2: Build the player**

Run:
```bash
cargo build -p nethercore-zx
```

**Step 3: Run replay**

Run:
```bash
cargo run -p nethercore-zx -- examples/3-inspectors/epu-showcase/target/wasm32-unknown-unknown/release/epu_showcase.wasm --replay examples/3-inspectors/epu-showcase/screenshot-all.ncrs
```

Expected: Window opens, presets cycle automatically, 24 screenshots are saved, player exits.

**Step 4: Verify screenshots**

Check the screenshots directory for 24 PNG files.

**Step 5: Run full test suite**

Run: `cargo test`
Expected: PASS
