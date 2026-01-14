# Replay Debug Action Integration

> **Status:** Implemented (schema + compilation; execution depends on the replay backend)
> Last reviewed: 2026-01-14

## Overview

Replay scripts can invoke **debug actions** at specific frames (e.g. "Load Level") to set up game state without recording long input sequences. This keeps scripts short, stable, and focused on the behavior under test.

## Current State

### Debug Actions (Implemented)

Games can register callable actions with parameters:

```rust
// In game's init()
debug_action_begin(b"Load Level".as_ptr(), 10, b"debug_load_level".as_ptr(), 16);
debug_action_param_i32(b"level".as_ptr(), 5, 1);  // param "level", default 1
debug_action_end();
```

The debug inspector UI shows these as buttons with editable parameter fields. When clicked, it calls the exported WASM function with the parameter values.

**Registry structure:**
```rust
pub struct RegisteredAction {
    pub name: String,           // "Load Level"
    pub full_path: String,      // "debug/Load Level"
    pub func_name: String,      // "debug_load_level"
    pub params: Vec<ActionParam>,
}

pub struct ActionParam {
    pub name: String,           // "level"
    pub param_type: ValueType,  // I32
    pub default: ActionParamValue,
}
```

### Replay Scripts (Implemented)

Current `.ncrs` format:

```toml
console = "zx"
seed = 12345
players = 1

[[frames]]
f = 0
p1 = "idle"
snap = true

[[frames]]
f = 60
p1 = "right+a"
assert = "$player_x > 100"
```

## Script Format: Actions

### Script Format Addition

Replay scripts support optional `action` and `action_params` fields on frame entries:

```toml
console = "zx"
seed = 12345
players = 1

# Frame 0: Invoke debug action to skip to level 2
[[frames]]
f = 0
action = "Load Level"
action_params = { level = 2 }

# Frame 1: Start actual input recording from level 2
[[frames]]
f = 1
p1 = "idle"
snap = true

[[frames]]
f = 120
p1 = "right+a"
assert = "$player_x > 100"
```

### Parser Changes

Implemented in `core/src/replay/script/ast.rs` (`FrameEntry`):

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FrameEntry {
    pub f: u64,
    #[serde(default)]
    pub p1: Option<InputValue>,
    #[serde(default)]
    pub p2: Option<InputValue>,
    #[serde(default)]
    pub p3: Option<InputValue>,
    #[serde(default)]
    pub p4: Option<InputValue>,
    #[serde(default)]
    pub snap: bool,
    #[serde(default)]
    pub assert: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_params: Option<HashMap<String, ActionParamValue>>,
}
```

### Executor Changes

Execution order is implemented in `core/src/replay/runtime/headless.rs`:

```rust
let actions = self.executor.current_actions();
if !actions.is_empty() {
    backend.execute_actions(actions.as_slice())?;
}

let inputs = self.executor.current_inputs();
backend.apply_inputs(inputs)?;

backend.update()?;

// (optional) snapshots + assertions...

self.executor.advance_frame();
```

The per-frame query helpers live in `core/src/replay/runtime/executor/mod.rs` (`current_actions`, `current_inputs`, `needs_snapshot`, `current_assertions`).

### Compiled Script Changes

Implemented in `core/src/replay/script/compiler.rs`:

```rust
#[derive(Debug, Clone)]
pub struct CompiledAction {
    pub frame: u64,
    pub name: String,
    pub params: HashMap<String, ActionParamValue>,
}

pub struct CompiledScript {
    // ... existing fields ...
    pub actions: Vec<CompiledAction>,
}
```

## Use Cases

### 1. Level-Specific Testing

Test level 3 boss without playing through levels 1-2:

```toml
console = "zx"
seed = 42
players = 1

[[frames]]
f = 0
action = "Load Level"
action_params = { level = 3 }

[[frames]]
f = 1
snap = true  # Capture initial state of level 3

[[frames]]
f = 300
assert = "$boss_health < 100"  # Verify boss took damage
```

### 2. Spawn Testing

Test enemy behavior by spawning specific enemies:

```toml
[[frames]]
f = 0
action = "Spawn Enemy"
action_params = { enemy_type = "goblin", x = 100, y = 50 }

[[frames]]
f = 1
p1 = "right+a"  # Attack the spawned enemy
snap = true
```

### 3. State Setup for Regression Tests

Set up specific game state for testing edge cases:

```toml
[[frames]]
f = 0
action = "Set Player Health"
action_params = { health = 1 }

[[frames]]
f = 0
action = "Set Player Position"
action_params = { x = 500, y = 200 }

[[frames]]
f = 1
p1 = "a"  # Jump
assert = "$player_alive == 1"  # Should survive
```

### 4. AI-Assisted Debugging with Claude Code

The zx-dev plugin can leverage this for targeted debugging:

```markdown
**User:** "The game crashes when I beat the level 2 boss with low health"

**Claude:** Let me create a test script to reproduce this:

1. Skip to level 2 boss room
2. Set player health to 1
3. Simulate boss defeat sequence
4. Check for crash/invalid state
```

Generated script:
```toml
console = "zx"
seed = 12345
players = 1

[[frames]]
f = 0
action = "Load Level"
action_params = { level = 2 }

[[frames]]
f = 0
action = "Skip To Boss"

[[frames]]
f = 0
action = "Set Player Health"
action_params = { health = 1 }

[[frames]]
f = 1
snap = true

# Simulate killing the boss
[[frames]]
f = 60
p1 = "a"
# ... attack inputs ...

[[frames]]
f = 500
assert = "$game_state != CRASHED"
snap = true
```

## Tooling Guidance

When writing or generating replay scripts, prefer debug actions over long input sequences:

- Use `action = "Load Level"` to skip menus/tutorials
- Use `action = "Set Player Position"`/`"Spawn Enemy"` to construct test scenarios
- Combine actions with `snap = true` + `assert = "..."` to make failures obvious and diffable

## Compatibility

- Scripts without `action` fields work unchanged
- Actions are optional - pure input scripts remain valid
- Games without debug actions can still use input-only scripts
- Binary format (`.ncrp`) does not include actions (input-only)

## Current Behavior (Implemented)

- Invocation timing: actions run **before** inputs for the same frame in `core/src/replay/runtime/headless.rs`.
- Multiple actions per frame: supported by emitting multiple `[[frames]]` entries with the same `f`.
- Backend hook: actions are routed through `HeadlessBackend::execute_actions` (default no-op).
- Recording: the replay recorder currently captures inputs only (no debug actions).
