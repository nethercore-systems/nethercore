# Nethercore Architecture Guide

## Overview

This document describes Nethercore's codebase organization principles, module structure, and maintainability guidelines. These principles were established after a major refactoring effort that reduced massive monolithic files into focused, maintainable modules.

## Core Principles

### The 800-Line Rule

**No Rust source file should exceed 800 lines of code (excluding generated code, tests, and blank lines/comments).**

**Rationale:**
- Files beyond 800 lines become difficult to navigate and understand
- Large files encourage further growth, creating "dumping ground" modules
- Splitting forces clearer separation of concerns
- Smaller files improve compilation times and IDE responsiveness

**When a file approaches 600 lines:**
- Consider splitting it into submodules
- Extract cohesive functionality into separate files
- Create a module directory if multiple related files emerge

**Exceptions:**
- Generated code (e.g., `shader_gen.rs` with template-generated shaders)
- Test modules with extensive test cases (keep related tests together)
- Files with clear boundaries that would be awkward to split

### Single Source of Truth

**Every piece of data should have exactly one canonical source. Derived values should reference that source, not duplicate it.**

**Rationale:**
- Duplicate definitions drift apart over time
- Changes require updating multiple locations (easy to miss one)
- Readers can't tell which source is authoritative
- Tests may pass with inconsistent values

**Common violations:**
- Hardcoded strings that duplicate values in config/specs structs
- Methods that return the same value as an existing field
- Constants defined in multiple modules
- Display names duplicating internal identifiers

**How to fix:**
```rust
// ❌ BAD: Duplicate source of truth
impl Console for MyConsole {
    fn specs() -> &'static ConsoleSpecs {
        &SPECS  // name: "My Console"
    }

    fn window_title() -> &'static str {
        "My Console"  // Duplicates specs.name!
    }
}

// ✅ GOOD: Single source of truth
impl Console for MyConsole {
    fn specs() -> &'static ConsoleSpecs {
        &SPECS  // name: "My Console"
    }
    // No window_title() - callers use Self::specs().name
}
```

**Guidelines:**
- Identify the canonical source for each piece of data
- Remove methods/fields that merely duplicate existing data
- Use derived accessors that reference the canonical source
- Document which struct/const is the source of truth for a domain

### Module Organization

Nethercore follows a **2-level module depth maximum** for most code:

```
crate/src/
├── lib.rs                 # Top level - re-exports public API
├── feature/
│   ├── mod.rs             # Feature entry - public interface + re-exports
│   ├── submodule_a.rs     # Implementation
│   └── submodule_b.rs     # Implementation
```

**Benefits:**
- Clear mental model: "Where is X?" → "In Y module" → "In Z submodule"
- Easy navigation: maximum 3 files to locate any functionality
- Prevents deep nesting that obscures code organization

### Module Structure Patterns

#### Pattern 1: Single-File Module

For simple, focused functionality under ~600 lines:

```rust
// src/feature.rs
pub struct Feature { /* ... */ }

impl Feature {
    pub fn new() -> Self { /* ... */ }
    pub fn do_thing(&mut self) { /* ... */ }
}

#[cfg(test)]
mod tests { /* ... */ }
```

**Use when:**
- Functionality is cohesive and self-contained
- File is under 600 lines
- No clear sub-components emerge

#### Pattern 2: Multi-File Module

For complex features over ~600 lines:

```rust
// src/feature/mod.rs
mod submodule_a;
mod submodule_b;

pub use submodule_a::TypeA;
pub use submodule_b::TypeB;

pub struct Feature {
    a: TypeA,
    b: TypeB,
}

// Main orchestration logic stays here
impl Feature {
    pub fn new() -> Self { /* ... */ }
}

// src/feature/submodule_a.rs
pub struct TypeA { /* ... */ }

impl TypeA {
    pub(super) fn internal_method(&self) { /* ... */ }
}

// src/feature/submodule_b.rs
pub struct TypeB { /* ... */ }
```

**Use when:**
- Feature exceeds 600 lines
- Clear sub-responsibilities emerge
- Related types/functions can be grouped

#### Pattern 3: Registration Module (FFI/Plugin Systems)

For systems with many small functions that need central registration:

```rust
// src/ffi/mod.rs
mod audio;
mod graphics;
mod input;

pub fn register_all(linker: &mut Linker) -> Result<()> {
    audio::register(linker)?;
    graphics::register(linker)?;
    input::register(linker)?;
    Ok(())
}

// src/ffi/audio/mod.rs
pub(super) fn register(linker: &mut Linker) -> Result<()> {
    linker.func_wrap("env", "play_sound", play_sound)?;
    linker.func_wrap("env", "stop_sound", stop_sound)?;
    Ok(())
}

fn play_sound(/* ... */) { /* ... */ }
fn stop_sound(/* ... */) { /* ... */ }
```

**Use when:**
- Many small functions organized by domain
- Central registration/initialization required
- Each domain is 100-400 lines

### Re-export Strategy

Nethercore uses a **3-tier visibility system**:

```rust
// lib.rs - Public API (external crates)
pub use feature::PublicType;

// feature/mod.rs - Module API (crate-internal)
pub(crate) use submodule::InternalType;
pub use submodule::PublicType;

// feature/submodule.rs - Implementation
pub struct PublicType { /* ... */ }
pub(super) struct ParentAccessType { /* ... */ }
struct PrivateType { /* ... */ }
```

**Visibility levels:**
- `pub` - Public API, stable interface
- `pub(crate)` - Internal to crate, can change freely
- `pub(super)` - Parent module only, tight coupling
- (no modifier) - Private to file

**Guidelines:**
- Default to most restrictive visibility
- Use `pub(super)` for parent-child communication
- Only expose through lib.rs what external crates need
- Mark internal helpers as `pub(crate)` if used across modules

## Real Examples from Nethercore

### Example 1: ZX FFI split by domain

Keep `mod.rs` focused on registration/orchestration, and split the API by domain:

```
nethercore-zx/src/ffi/
├── mod.rs                  # Registration/orchestration only
├── assets.rs
├── audio/                  # Music/tracker/sound
├── draw_2d/                # Sprites/shapes/text
├── environment/            # Sky + environment state
├── keyframes/              # Keyframe load/query/access
├── mesh_generators/        # Procedural mesh helpers
├── camera.rs
├── config.rs
├── draw_3d.rs
├── input.rs
├── lighting.rs
├── material.rs
├── mesh.rs
├── render_state.rs
├── rom.rs
├── skinning.rs
├── texture.rs
├── transform.rs
└── viewport.rs
```

**Key decisions:**
- Split by *game-facing API domain* (audio/graphics/input/etc.), not by technical layer.
- Keep registration in `mod.rs`; keep implementations in domain modules.
- Keep the “init-only / rollback-safe” rules co-located with the FFI code that enforces them.

### Example 2: Graphics subsystem uses “subsystem folders”

`nethercore-zx/src/graphics/` is organized around responsibilities that change together:

```
nethercore-zx/src/graphics/
├── buffer/                 # GPU buffers + retained mesh storage
├── frame/                  # Per-frame submission + bind-group caching
├── pipeline/               # Pipeline keys, creation, cache
├── unified_shading_state/  # Packed shading/environment state
├── vertex/                 # Vertex formats + helpers
└── zx_graphics.rs          # High-level graphics facade
```

### Example 3: State split between config, staging, rollback

Keep init config and rollback-relevant state explicit, and keep “FFI staging” isolated:

```
nethercore-zx/src/state/
├── config.rs               # Init-time console config (ZXInitConfig)
├── ffi_state/              # Per-frame staging written by FFI calls
├── rollback_state.rs       # Rollback-reachable state
├── resources.rs            # Pending resources created during init()
└── pool.rs                 # Dedup/dirty-tracking pools
```

## When to Split a File

### Split when:

1. **File exceeds 600 lines** - Start planning the split
2. **File exceeds 800 lines** - Split immediately
3. **Multiple concerns emerge** - Even if under 600 lines, clear boundaries suggest splitting
4. **Team velocity slows** - Hard-to-navigate files slow development
5. **Merge conflicts increase** - Large files create contention

### Keep together when:

1. **Tight coupling** - Types that change together should stay together
2. **Single clear responsibility** - One cohesive purpose under 600 lines
3. **Splitting would create awkward dependencies** - Circular deps or excessive pub(super)
4. **Generated code** - Template-generated content is exempt from line limits

## Watchdog Comments

For files that should remain small, add watchdog comments:

```rust
//! # WATCHDOG: Keep this file under 200 lines
//!
//! This file should ONLY contain:
//! - Module declarations
//! - Public re-exports
//! - Registration function
//!
//! ❌ DO NOT add implementations here
//! ✅ DO add them to domain-specific submodules
```

Place these at the top of:
- Central registration files (ffi/mod.rs)
- API boundary files (lib.rs)
- Any file prone to accumulation

## Code Ownership

### Module Responsibilities

Each module should have a **single, clear purpose** expressible in one sentence:

- ✅ **Good**: "`ffi/audio/mod.rs` registers WebAssembly host functions for audio playback"
- ❌ **Bad**: "`utils.rs` contains various helper functions"

### Avoid "Utility Dumping Grounds"

Common anti-patterns to avoid:

```rust
// ❌ BAD: Generic dumping ground
src/utils.rs            2,500 lines
src/helpers.rs          1,800 lines
src/common.rs           3,200 lines
```

Instead, organize by domain:

```rust
// ✅ GOOD: Specific, focused modules
src/math/
├── vector.rs           200 lines
├── matrix.rs           250 lines
└── transform.rs        180 lines

src/string/
├── parsing.rs          150 lines
└── formatting.rs       120 lines
```

### Discoverability

Code should be easy to find:

1. **Naming matches intent**: "I need audio playback" → "Check `audio.rs`"
2. **Logical grouping**: Related functionality in same module
3. **Consistent patterns**: Similar problems solved similarly across codebase

## Migration Strategy

When refactoring a large file:

1. **Analyze** - Identify natural boundaries (structs, function groups, concerns)
2. **Plan** - Sketch the new structure (5-10 files max per split)
3. **Create** - Make new files with focused responsibilities
4. **Move** - Migrate code chunk by chunk
5. **Test** - Verify each step compiles and passes tests
6. **Commit** - Commit after each successful migration step
7. **Clean** - Remove old file once all content migrated

**Example commit sequence:**
```
git commit -m "Phase 1: Create ffi/audio/ with audio FFI functions"
git commit -m "Phase 2: Create ffi/graphics.rs with graphics FFI functions"
git commit -m "Phase 3: Create ffi/mod.rs with registration logic"
git commit -m "Phase 4: Remove old ffi.rs monolithic file"
```

## Success Metrics

These are the practical outcomes we want:

- **Navigation stays fast**: most features live in one obvious module folder.
- **Diffs stay local**: changes for a feature touch a small number of focused files.
- **Merge conflicts stay low**: fewer “everyone edits the same file” hotspots.
- **File size stays bounded**: proactively split before files become dumping grounds.

## Tools & Enforcement

### Manual Checks

```bash
# Find files exceeding 800 lines
find . -name "*.rs" -exec wc -l {} \; | awk '$1 > 800 {print $2 " - " $1 " lines"}'

# Count lines in a module
wc -l src/feature/*.rs
```

### Future: CI Enforcement

Add to `.github/workflows/lint.yml`:

```yaml
- name: Check file size limits
  run: |
    MAX_LINES=800
    VIOLATIONS=$(find . -name "*.rs" -not -path "*/target/*" -exec wc -l {} \; | awk -v max=$MAX_LINES '$1 > max {print}')
    if [ -n "$VIOLATIONS" ]; then
      echo "Files exceeding $MAX_LINES lines:"
      echo "$VIOLATIONS"
      exit 1
    fi
```

## Contributing Guidelines

When adding new code:

1. **Start small** - New features begin in single files
2. **Watch the line count** - Split proactively at ~600 lines
3. **Follow patterns** - Match existing module structure
4. **Test continuously** - Ensure tests pass after each change
5. **Document boundaries** - Add module-level docs explaining scope

When reviewing code:

1. **Check line counts** - Flag files approaching 600 lines
2. **Verify organization** - Ensure code matches module's purpose
3. **Suggest splits** - Propose restructuring for large PRs
4. **Praise good structure** - Recognize well-organized code

## Related Documentation

- [FFI Reference](./ffi.md) - Public WebAssembly host function API
- [NCHS Protocol](./nchs.md) - Pre-GGRS handshake and session negotiation
- [ZX Rendering Architecture](./zx/rendering.md) - Graphics system deep dive
- [ROM Format](./rom-format.md) - ROM/cart format specification

## Questions?

For architecture questions or refactoring guidance, consult this document first. If your situation isn't covered, consider:

1. Looking at similar patterns in the existing codebase
2. Discussing with the team
3. Proposing an update to this document

**Remember**: These are guidelines, not laws. Use judgment, but default to the established patterns for consistency.
