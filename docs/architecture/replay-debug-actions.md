# Replay Debug Action Integration

> **Status:** Proposed (not yet implemented)
> **Depends on:** Replay System (implemented), Debug Inspector Actions (implemented)

## Overview

This document describes how to extend the replay system to invoke debug actions during script execution. This enables scenarios like "skip to level 2" without recording through menus, making test scripts more focused and maintainable.

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

## Proposed Extension

### Script Format Addition

Add optional `action` and `action_params` fields to frame entries:

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

Extend `FrameEntry` in `script/parser.rs`:

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FrameEntry {
    pub f: u64,
    pub p1: Option<InputValue>,
    pub p2: Option<InputValue>,
    pub p3: Option<InputValue>,
    pub p4: Option<InputValue>,
    pub snap: bool,
    pub assert: Option<String>,

    // NEW: Debug action invocation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_params: Option<HashMap<String, ActionParamValue>>,
}
```

### Executor Changes

In `runtime/executor.rs`, before applying inputs for a frame:

```rust
fn execute_frame(&mut self, frame: u64) -> Result<()> {
    // 1. Check for debug action invocation
    if let Some(action_name) = self.get_action_for_frame(frame) {
        self.invoke_debug_action(action_name)?;
    }

    // 2. Apply player inputs (existing code)
    self.apply_inputs(frame)?;

    // 3. Run game tick
    self.console.tick()?;

    // 4. Evaluate assertions and snapshots (existing code)
    self.evaluate_frame(frame)?;

    Ok(())
}

fn invoke_debug_action(&mut self, action: &CompiledAction) -> Result<()> {
    // Look up registered action by name
    let registry = self.console.debug_registry();
    let registered = registry.actions
        .iter()
        .find(|a| a.name == action.name)
        .ok_or_else(|| anyhow!("Unknown debug action: {}", action.name))?;

    // Build parameter values (use script values, fall back to defaults)
    let mut params = Vec::new();
    for param_def in &registered.params {
        let value = action.params
            .get(&param_def.name)
            .cloned()
            .unwrap_or(param_def.default.clone());
        params.push(value);
    }

    // Call the WASM export
    self.wasm.call_action(&registered.func_name, &params)?;

    Ok(())
}
```

### Compiled Script Changes

Add action tracking to `CompiledScript`:

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

## Integration with zx-dev Plugin

The zx-dev plugin's debugging skill should:

1. **Query available actions** - Read the game's registered debug actions
2. **Generate targeted scripts** - Use actions to set up specific test scenarios
3. **Combine with assertions** - Verify expected behavior after action invocation
4. **Minimize input sequences** - Use actions to skip irrelevant gameplay

### Plugin Knowledge Updates

Add to the debugging skill:

```markdown
## Debug Actions in Replay Scripts

When creating test scripts, prefer debug actions over long input sequences:

**Instead of:**
- Recording 5 minutes of menu navigation
- Playing through tutorial levels
- Manually positioning enemies

**Use:**
- `action = "Load Level"` to skip to specific levels
- `action = "Set Player Position"` to place player
- `action = "Spawn Enemy"` to create test scenarios

This makes scripts:
- Faster to execute
- More focused on the bug
- Less brittle to unrelated changes
```

## Implementation Order

1. **Parser extension** - Add `action`/`action_params` to `FrameEntry`
2. **Compiler extension** - Track actions in `CompiledScript`
3. **Executor extension** - Invoke actions before frame inputs
4. **WASM bridge** - Add `call_action()` to WASM engine
5. **Plugin update** - Teach zx-dev about action-based scripts

## Compatibility

- Scripts without `action` fields work unchanged
- Actions are optional - pure input scripts remain valid
- Games without debug actions can still use input-only scripts
- Binary format (`.ncrp`) does not include actions (input-only)

## Open Questions

1. **Action timing** - Should actions be invoked at start of frame, or before first input?
2. **Multiple actions per frame** - Allow array of actions? Or one per frame entry?
3. **Action recording** - Should the recorder capture manual debug action invocations?
4. **Error handling** - What happens if an action fails mid-script?
