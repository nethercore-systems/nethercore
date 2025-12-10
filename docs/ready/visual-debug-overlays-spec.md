# Visual Debug Overlays Specification

## Overview

Visual Debug Overlays allow games to render debug visualizations directly in the game world - collision boxes, AI paths, spawn points, velocity vectors, and more. Unlike the Debug Inspection panel (which shows data), overlays show spatial information in context.

This enables:
- **Collision debugging**: See hitboxes, trigger volumes, physics shapes
- **AI debugging**: Visualize pathfinding, line-of-sight, decision trees
- **Spatial debugging**: Spawn points, waypoints, camera frustums
- **Performance debugging**: Draw call regions, LOD boundaries

## Architecture

### Overlay Types

```rust
/// Categories of debug overlays (can be toggled independently)
pub enum OverlayCategory {
    /// Collision shapes, hitboxes, triggers
    Collision,

    /// AI paths, vision cones, behavior states
    AI,

    /// Physics forces, velocities, constraints
    Physics,

    /// Camera frustums, viewports, render regions
    Camera,

    /// Spawn points, waypoints, navigation meshes
    Navigation,

    /// Draw calls, batches, culling regions
    Performance,

    /// Custom game-specific overlays
    Custom(u32),
}

/// Visual style for overlay rendering
pub struct OverlayStyle {
    /// Primary color (RGBA, 0-255)
    pub color: [u8; 4],

    /// Line thickness for wireframes (pixels)
    pub line_width: f32,

    /// Fill opacity (0.0 = wireframe only, 1.0 = solid)
    pub fill_opacity: f32,

    /// Whether to depth-test (false = always visible)
    pub depth_test: bool,

    /// Label to display (optional)
    pub label: Option<String>,
}
```

### Rendering Pipeline

Debug overlays render in a separate pass after the main game render:

```
┌─────────────────────────────────────────────────────────────┐
│                    Frame Rendering                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. Game calls render()                                     │
│     └─ Normal draw commands                                 │
│                                                             │
│  2. Game calls debug_* overlay functions                    │
│     └─ Commands buffered (not rendered yet)                 │
│                                                             │
│  3. Host executes main render pass                          │
│     └─ Normal game visuals                                  │
│                                                             │
│  4. Host executes debug overlay pass                        │
│     └─ Semi-transparent overlays on top                     │
│     └─ Text labels rendered last                            │
│                                                             │
│  5. Host renders egui (debug panel, etc.)                   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Overlay Buffer

```rust
/// Single overlay draw command
pub enum OverlayCommand {
    /// Axis-aligned bounding box
    Box3D {
        min: [f32; 3],
        max: [f32; 3],
        style: OverlayStyle,
    },

    /// Oriented bounding box
    OrientedBox3D {
        center: [f32; 3],
        half_extents: [f32; 3],
        rotation: [f32; 4], // quaternion
        style: OverlayStyle,
    },

    /// Sphere
    Sphere {
        center: [f32; 3],
        radius: f32,
        style: OverlayStyle,
    },

    /// Capsule (cylinder with hemispherical caps)
    Capsule {
        start: [f32; 3],
        end: [f32; 3],
        radius: f32,
        style: OverlayStyle,
    },

    /// Line segment
    Line {
        start: [f32; 3],
        end: [f32; 3],
        style: OverlayStyle,
    },

    /// Arrow (line with arrowhead)
    Arrow {
        start: [f32; 3],
        end: [f32; 3],
        head_size: f32,
        style: OverlayStyle,
    },

    /// Path (connected line segments)
    Path {
        points: Vec<[f32; 3]>,
        closed: bool,
        style: OverlayStyle,
    },

    /// Triangle
    Triangle {
        vertices: [[f32; 3]; 3],
        style: OverlayStyle,
    },

    /// Circle (in world space)
    Circle {
        center: [f32; 3],
        radius: f32,
        normal: [f32; 3],
        style: OverlayStyle,
    },

    /// Cone (for vision cones, lights, etc.)
    Cone {
        apex: [f32; 3],
        direction: [f32; 3],
        height: f32,
        angle: f32, // radians
        style: OverlayStyle,
    },

    /// Frustum (for camera debugging)
    Frustum {
        corners: [[f32; 3]; 8], // near TL/TR/BL/BR, far TL/TR/BL/BR
        style: OverlayStyle,
    },

    /// World-space text label
    Text {
        position: [f32; 3],
        text: String,
        style: OverlayStyle,
    },

    /// 2D screen-space rectangle
    Rect2D {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        style: OverlayStyle,
    },

    /// 2D screen-space text
    Text2D {
        x: f32,
        y: f32,
        text: String,
        style: OverlayStyle,
    },
}

/// Buffer holding all overlay commands for a frame
pub struct OverlayBuffer {
    /// Commands organized by category
    pub commands: HashMap<OverlayCategory, Vec<OverlayCommand>>,

    /// Which categories are enabled
    pub enabled_categories: HashSet<OverlayCategory>,
}
```

## FFI API

### Category Control

```rust
// Enable/disable overlay categories
extern "C" fn debug_overlay_enable(category: u32);
extern "C" fn debug_overlay_disable(category: u32);
extern "C" fn debug_overlay_toggle(category: u32);
extern "C" fn debug_overlay_is_enabled(category: u32) -> i32;

// Enable/disable all overlays
extern "C" fn debug_overlay_enable_all();
extern "C" fn debug_overlay_disable_all();
```

### Style Configuration

```rust
// Set default style for subsequent draws
extern "C" fn debug_overlay_set_color(r: u8, g: u8, b: u8, a: u8);
extern "C" fn debug_overlay_set_line_width(width: f32);
extern "C" fn debug_overlay_set_fill_opacity(opacity: f32);
extern "C" fn debug_overlay_set_depth_test(enabled: i32);

// Push/pop style stack
extern "C" fn debug_overlay_push_style();
extern "C" fn debug_overlay_pop_style();
```

### 3D Primitives

```rust
// Axis-aligned bounding box
extern "C" fn debug_draw_box(
    category: u32,
    min_x: f32, min_y: f32, min_z: f32,
    max_x: f32, max_y: f32, max_z: f32,
);

// Oriented bounding box (center + half-extents + quaternion)
extern "C" fn debug_draw_obb(
    category: u32,
    cx: f32, cy: f32, cz: f32,
    hx: f32, hy: f32, hz: f32,
    qx: f32, qy: f32, qz: f32, qw: f32,
);

// Sphere
extern "C" fn debug_draw_sphere(
    category: u32,
    cx: f32, cy: f32, cz: f32,
    radius: f32,
);

// Capsule
extern "C" fn debug_draw_capsule(
    category: u32,
    x1: f32, y1: f32, z1: f32,
    x2: f32, y2: f32, z2: f32,
    radius: f32,
);

// Line
extern "C" fn debug_draw_line(
    category: u32,
    x1: f32, y1: f32, z1: f32,
    x2: f32, y2: f32, z2: f32,
);

// Arrow (line with head)
extern "C" fn debug_draw_arrow(
    category: u32,
    x1: f32, y1: f32, z1: f32,
    x2: f32, y2: f32, z2: f32,
    head_size: f32,
);

// Circle in world space
extern "C" fn debug_draw_circle(
    category: u32,
    cx: f32, cy: f32, cz: f32,
    radius: f32,
    nx: f32, ny: f32, nz: f32, // normal
);

// Cone (for vision cones, spotlights)
extern "C" fn debug_draw_cone(
    category: u32,
    ax: f32, ay: f32, az: f32,  // apex
    dx: f32, dy: f32, dz: f32,  // direction
    height: f32,
    angle_degrees: f32,
);

// Triangle
extern "C" fn debug_draw_triangle(
    category: u32,
    x1: f32, y1: f32, z1: f32,
    x2: f32, y2: f32, z2: f32,
    x3: f32, y3: f32, z3: f32,
);
```

### Path Drawing

```rust
// Begin a path
extern "C" fn debug_path_begin(category: u32);

// Add point to current path
extern "C" fn debug_path_point(x: f32, y: f32, z: f32);

// End path (open)
extern "C" fn debug_path_end();

// End path (closed - connect last to first)
extern "C" fn debug_path_end_closed();
```

### Text Labels

```rust
// 3D world-space text (billboarded)
extern "C" fn debug_draw_text_3d(
    category: u32,
    x: f32, y: f32, z: f32,
    text_ptr: u32, text_len: u32,
);

// 2D screen-space text
extern "C" fn debug_draw_text_2d(
    category: u32,
    x: f32, y: f32,
    text_ptr: u32, text_len: u32,
);

// Formatted text with value (convenience)
extern "C" fn debug_draw_text_3d_f32(
    category: u32,
    x: f32, y: f32, z: f32,
    label_ptr: u32, label_len: u32,
    value: f32,
);
```

### 2D Overlays (Screen Space)

```rust
// Screen-space rectangle
extern "C" fn debug_draw_rect_2d(
    category: u32,
    x: f32, y: f32,
    width: f32, height: f32,
);

// Screen-space line
extern "C" fn debug_draw_line_2d(
    category: u32,
    x1: f32, y1: f32,
    x2: f32, y2: f32,
);

// Screen-space circle
extern "C" fn debug_draw_circle_2d(
    category: u32,
    cx: f32, cy: f32,
    radius: f32,
);
```

### Coordinate Transforms

```rust
// Project world position to screen coordinates
// Returns 0 if behind camera, 1 if visible
// Writes screen x,y to output pointers
extern "C" fn debug_world_to_screen(
    world_x: f32, world_y: f32, world_z: f32,
    screen_x_ptr: u32, screen_y_ptr: u32,
) -> i32;
```

## Usage Patterns

### Collision Boxes

```rust
// In game's render() function
fn render_debug_collision(entity: &Entity) {
    // Set collision category style
    debug_overlay_set_color(255, 0, 0, 128);  // Red, semi-transparent
    debug_overlay_set_fill_opacity(0.2);
    debug_overlay_set_depth_test(false);  // Always visible

    // Draw hitbox
    debug_draw_box(
        COLLISION,
        entity.x - entity.width/2.0,
        entity.y,
        entity.z - entity.depth/2.0,
        entity.x + entity.width/2.0,
        entity.y + entity.height,
        entity.z + entity.depth/2.0,
    );

    // Draw attack range
    debug_overlay_set_color(255, 255, 0, 64);  // Yellow
    debug_draw_sphere(COLLISION, entity.x, entity.y + 1.0, entity.z, entity.attack_range);
}
```

### AI Vision Cones

```rust
fn render_debug_ai(enemy: &Enemy) {
    debug_overlay_set_color(0, 255, 0, 64);  // Green
    debug_overlay_set_fill_opacity(0.1);

    // Vision cone
    debug_draw_cone(
        AI,
        enemy.x, enemy.eye_height, enemy.z,  // apex
        enemy.facing_x, 0.0, enemy.facing_z,  // direction
        enemy.vision_range,  // height
        enemy.vision_angle,  // angle in degrees
    );

    // Current target
    if let Some(target) = enemy.target {
        debug_overlay_set_color(255, 0, 0, 255);  // Red
        debug_draw_line(
            AI,
            enemy.x, enemy.eye_height, enemy.z,
            target.x, target.y + 1.0, target.z,
        );
    }

    // State label
    debug_draw_text_3d(
        AI,
        enemy.x, enemy.y + 2.5, enemy.z,
        format!("State: {:?}", enemy.state),
    );
}
```

### Pathfinding

```rust
fn render_debug_path(path: &[Vec3]) {
    debug_overlay_set_color(0, 128, 255, 255);  // Blue
    debug_overlay_set_line_width(2.0);

    debug_path_begin(NAVIGATION);
    for point in path {
        debug_path_point(point.x, point.y, point.z);
    }
    debug_path_end();

    // Mark waypoints
    debug_overlay_set_color(255, 255, 0, 255);  // Yellow
    for (i, point) in path.iter().enumerate() {
        debug_draw_sphere(NAVIGATION, point.x, point.y, point.z, 0.2);
        debug_draw_text_3d(NAVIGATION, point.x, point.y + 0.5, point.z, &format!("{}", i));
    }
}
```

### Physics Vectors

```rust
fn render_debug_physics(body: &RigidBody) {
    // Velocity arrow
    debug_overlay_set_color(0, 255, 255, 255);  // Cyan
    debug_draw_arrow(
        PHYSICS,
        body.x, body.y, body.z,
        body.x + body.vx, body.y + body.vy, body.z + body.vz,
        0.1,
    );

    // Force arrows
    debug_overlay_set_color(255, 128, 0, 255);  // Orange
    for force in &body.forces {
        debug_draw_arrow(
            PHYSICS,
            body.x, body.y, body.z,
            body.x + force.x * 0.1, body.y + force.y * 0.1, body.z + force.z * 0.1,
            0.05,
        );
    }
}
```

## UI Integration

### Category Toggle Panel

In the debug panel (egui), add overlay controls:

```
┌─────────────────────────────────────┐
│ Debug Overlays                   [×]│
├─────────────────────────────────────┤
│ ☑ Collision    [color picker]       │
│ ☑ AI           [color picker]       │
│ ☐ Physics      [color picker]       │
│ ☐ Camera       [color picker]       │
│ ☑ Navigation   [color picker]       │
│ ☐ Performance  [color picker]       │
├─────────────────────────────────────┤
│ [Show All] [Hide All]               │
├─────────────────────────────────────┤
│ Options:                            │
│ ☐ Depth testing                     │
│ Line width: [━━━━━●━] 2.0          │
│ Fill opacity: [━━●━━━━] 0.3        │
└─────────────────────────────────────┘
```

### Hotkeys

| Key | Action |
|-----|--------|
| F4 | Toggle overlay panel |
| 1-6 | Toggle categories (Collision, AI, Physics, Camera, Navigation, Performance) |
| Shift+1-6 | Solo category (show only that one) |
| 0 | Hide all overlays |
| Shift+0 | Show all overlays |

## Performance Considerations

### Overlay Limits

```rust
/// Maximum overlays per frame to prevent runaway
const MAX_OVERLAYS_PER_FRAME: usize = 10_000;
const MAX_PATH_POINTS: usize = 1_000;
const MAX_TEXT_LENGTH: usize = 256;
```

### Batching

Group overlay commands by:
1. Category (for toggling)
2. Depth test mode (fewer state changes)
3. Primitive type (fewer draw calls)

### LOD for Overlays

For scenes with many entities:
- Small overlays auto-hide at distance
- Reduce sphere/circle segment count at distance
- Aggregate overlays for clustered entities

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Record overlays in replays | **No** | Overlays are transient debug output - regenerate from game state during playback |
| Custom category naming | **Yes** | Games can register custom categories with names and default colors for better discoverability |
| Per-entity filtering | **Categories only** (MVP) | Start simple. Add spatial filtering (near cursor) in future if needed. |
| Overlay persistence | **Cleared each frame** | Immediate-mode pattern - game redraws overlays each frame. Simpler, no stale data. |
| 2D vs 3D coordinate spaces | **Keep separate** | Clearer intent. `debug_draw_box()` vs `debug_draw_rect_2d()` are distinct use cases. |

### Custom Category Registration

Games can register custom categories beyond the built-in ones:

```rust
// Register a custom category
extern "C" fn debug_overlay_register_category(
    id: u32,              // Category ID (use values >= 100 for custom)
    name_ptr: u32,        // Null-terminated name string
    name_len: u32,
    default_color: u32,   // RGBA packed (0xRRGGBBAA)
);

// Example usage in game init():
debug_overlay_register_category(
    100,                  // Custom category ID
    "spawners\0".as_ptr(),
    8,
    0x00FF00FF,          // Green
);
```

Custom categories appear in the debug panel UI alongside built-in categories.

## Pros

1. **Spatial context**: See debug info where it matters in the game world
2. **Immediate feedback**: Changes visible instantly during development
3. **Category system**: Toggle groups on/off without code changes
4. **Standard primitives**: Common shapes cover most use cases
5. **Style customization**: Per-overlay colors, transparency, depth testing
6. **Integrates with debug panel**: Unified debug experience

## Cons

1. **Performance overhead**: Additional draw calls per frame
2. **Visual clutter**: Too many overlays can obscure the game
3. **3D math complexity**: Quaternions, transforms for OBB
4. **No persistence**: Must redraw every frame
5. **Fixed primitives**: Can't draw arbitrary meshes

## Implementation Complexity

**Estimated effort:** Medium

**Key components:**
1. Overlay buffer data structures - 0.5 days
2. FFI functions (all primitives) - 2 days
3. Overlay render pass (wgpu) - 3 days
   - Line rendering with width
   - Filled/wireframe shapes
   - Billboarded text
4. Category toggle system - 0.5 days
5. Debug panel UI integration - 1 day
6. Performance optimizations - 1 day
7. Testing - 1 day

**Total:** ~9 days

## Console-Agnostic Design

The overlay buffer lives in `GameStateWithConsole`, populated via FFI. Each console implements its own overlay renderer:
- `ZGraphics` implements `render_overlays(&self, buffer: &OverlayBuffer)`
- Future consoles can use 2D-only overlays or different rendering styles

## Way Forward: Implementation Guide

This section provides concrete implementation steps based on the current Emberware codebase architecture.

### Step 1: Add Overlay Types to Core

**File: `core/src/debug/overlay.rs` (new file)**

```rust
//! Visual debug overlay system

use std::collections::{HashMap, HashSet};

/// Overlay category (matches FFI constants)
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverlayCategory {
    Collision = 0,
    AI = 1,
    Physics = 2,
    Camera = 3,
    Navigation = 4,
    Performance = 5,
    Custom(u32),
}

impl From<u32> for OverlayCategory {
    fn from(v: u32) -> Self {
        match v {
            0 => Self::Collision,
            1 => Self::AI,
            2 => Self::Physics,
            3 => Self::Camera,
            4 => Self::Navigation,
            5 => Self::Performance,
            n => Self::Custom(n),
        }
    }
}

/// Current overlay style state
#[derive(Clone, Copy, Default)]
pub struct OverlayStyle {
    pub color: [u8; 4],
    pub line_width: f32,
    pub fill_opacity: f32,
    pub depth_test: bool,
}

/// Single overlay draw command
#[derive(Clone)]
pub enum OverlayCommand {
    Box3D { min: [f32; 3], max: [f32; 3], style: OverlayStyle },
    Sphere { center: [f32; 3], radius: f32, style: OverlayStyle },
    Line { start: [f32; 3], end: [f32; 3], style: OverlayStyle },
    Arrow { start: [f32; 3], end: [f32; 3], head_size: f32, style: OverlayStyle },
    Cone { apex: [f32; 3], dir: [f32; 3], height: f32, angle: f32, style: OverlayStyle },
    Text3D { pos: [f32; 3], text: String, style: OverlayStyle },
    // ... other commands
}

/// Buffer holding all overlay commands for a frame
#[derive(Default)]
pub struct OverlayBuffer {
    pub commands: HashMap<OverlayCategory, Vec<OverlayCommand>>,
    pub enabled_categories: HashSet<OverlayCategory>,
    style_stack: Vec<OverlayStyle>,
    current_style: OverlayStyle,
    path_points: Vec<[f32; 3]>,
    path_category: Option<OverlayCategory>,
}

impl OverlayBuffer {
    pub fn new() -> Self {
        let mut enabled = HashSet::new();
        enabled.insert(OverlayCategory::Collision);
        Self {
            enabled_categories: enabled,
            current_style: OverlayStyle {
                color: [255, 255, 255, 255],
                line_width: 1.0,
                fill_opacity: 0.3,
                depth_test: true,
            },
            ..Default::default()
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
        self.path_points.clear();
        self.path_category = None;
    }

    pub fn push_command(&mut self, category: OverlayCategory, cmd: OverlayCommand) {
        if self.enabled_categories.contains(&category) {
            self.commands.entry(category).or_default().push(cmd);
        }
    }

    pub fn set_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.current_style.color = [r, g, b, a];
    }

    pub fn push_style(&mut self) {
        self.style_stack.push(self.current_style);
    }

    pub fn pop_style(&mut self) {
        if let Some(s) = self.style_stack.pop() {
            self.current_style = s;
        }
    }

    pub fn style(&self) -> OverlayStyle {
        self.current_style
    }
}
```

### Step 2: Add Overlay Buffer to Console State

**File: `core/src/wasm/state.rs`**

Add overlay buffer to `GameStateWithConsole`:

```rust
use crate::debug::overlay::OverlayBuffer;

pub struct GameStateWithConsole<I: ConsoleInput, S: Send + Default> {
    pub game: GameState<I>,
    pub console_state: S,
    pub debug_registry: DebugRegistry,  // From debug inspection
    pub overlay_buffer: OverlayBuffer,  // NEW
}

impl<I: ConsoleInput, S: Send + Default + 'static> GameStateWithConsole<I, S> {
    pub fn new() -> Self {
        Self {
            game: GameState::new(),
            console_state: S::default(),
            debug_registry: DebugRegistry::new(),
            overlay_buffer: OverlayBuffer::new(),
        }
    }
}
```

### Step 3: Register FFI Functions

**File: `core/src/ffi.rs`**

Add overlay FFI functions:

```rust
pub fn register_common_ffi<I: ConsoleInput, S: Send + Default + 'static>(
    linker: &mut Linker<GameStateWithConsole<I, S>>,
) -> Result<()> {
    // ... existing registrations ...

    // Overlay functions
    linker.func_wrap("env", "debug_overlay_enable", debug_overlay_enable)?;
    linker.func_wrap("env", "debug_overlay_disable", debug_overlay_disable)?;
    linker.func_wrap("env", "debug_overlay_set_color", debug_overlay_set_color)?;
    linker.func_wrap("env", "debug_overlay_set_line_width", debug_overlay_set_line_width)?;
    linker.func_wrap("env", "debug_overlay_set_depth_test", debug_overlay_set_depth_test)?;
    linker.func_wrap("env", "debug_overlay_push_style", debug_overlay_push_style)?;
    linker.func_wrap("env", "debug_overlay_pop_style", debug_overlay_pop_style)?;
    linker.func_wrap("env", "debug_draw_box", debug_draw_box)?;
    linker.func_wrap("env", "debug_draw_sphere", debug_draw_sphere)?;
    linker.func_wrap("env", "debug_draw_line", debug_draw_line)?;
    linker.func_wrap("env", "debug_draw_arrow", debug_draw_arrow)?;
    // ... more primitive functions

    Ok(())
}

fn debug_overlay_set_color<I: ConsoleInput, S: Send + Default>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    r: u32, g: u32, b: u32, a: u32,
) {
    caller.data_mut().overlay_buffer.set_color(r as u8, g as u8, b as u8, a as u8);
}

fn debug_draw_box<I: ConsoleInput, S: Send + Default>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    category: u32,
    min_x: f32, min_y: f32, min_z: f32,
    max_x: f32, max_y: f32, max_z: f32,
) {
    let style = caller.data().overlay_buffer.style();
    let cmd = OverlayCommand::Box3D {
        min: [min_x, min_y, min_z],
        max: [max_x, max_y, max_z],
        style,
    };
    caller.data_mut().overlay_buffer.push_command(category.into(), cmd);
}

fn debug_draw_sphere<I: ConsoleInput, S: Send + Default>(
    mut caller: Caller<'_, GameStateWithConsole<I, S>>,
    category: u32,
    cx: f32, cy: f32, cz: f32, radius: f32,
) {
    let style = caller.data().overlay_buffer.style();
    let cmd = OverlayCommand::Sphere {
        center: [cx, cy, cz],
        radius,
        style,
    };
    caller.data_mut().overlay_buffer.push_command(category.into(), cmd);
}

// ... similar for other primitives
```

### Step 4: Implement Console Renderer

**File: `emberware-z/src/graphics/overlay_renderer.rs` (new file)**

```rust
use crate::graphics::ZGraphics;
use emberware_core::debug::overlay::{OverlayBuffer, OverlayCommand, OverlayStyle};
use wgpu::RenderPass;

/// Renders debug overlays for Emberware Z
pub struct OverlayRenderer {
    line_pipeline: wgpu::RenderPipeline,
    line_vertex_buffer: wgpu::Buffer,
    line_vertices: Vec<LineVertex>,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct LineVertex {
    position: [f32; 3],
    color: [f32; 4],
}

impl OverlayRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        // Create line rendering pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Overlay Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("overlay.wgsl").into()),
        });

        // ... pipeline setup ...

        Self {
            line_pipeline: todo!(),
            line_vertex_buffer: todo!(),
            line_vertices: Vec::new(),
        }
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        buffer: &OverlayBuffer,
        view_proj: &[[f32; 4]; 4],
    ) {
        self.line_vertices.clear();

        // Convert overlay commands to line vertices
        for (category, commands) in &buffer.commands {
            for cmd in commands {
                self.process_command(cmd);
            }
        }

        if self.line_vertices.is_empty() {
            return;
        }

        // Upload vertices and render
        // ...
    }

    fn process_command(&mut self, cmd: &OverlayCommand) {
        match cmd {
            OverlayCommand::Box3D { min, max, style } => {
                self.add_box_lines(*min, *max, style);
            }
            OverlayCommand::Sphere { center, radius, style } => {
                self.add_sphere_lines(*center, *radius, style);
            }
            OverlayCommand::Line { start, end, style } => {
                self.add_line(*start, *end, style);
            }
            // ... handle other commands
            _ => {}
        }
    }

    fn add_line(&mut self, start: [f32; 3], end: [f32; 3], style: &OverlayStyle) {
        let color = [
            style.color[0] as f32 / 255.0,
            style.color[1] as f32 / 255.0,
            style.color[2] as f32 / 255.0,
            style.color[3] as f32 / 255.0,
        ];
        self.line_vertices.push(LineVertex { position: start, color });
        self.line_vertices.push(LineVertex { position: end, color });
    }

    fn add_box_lines(&mut self, min: [f32; 3], max: [f32; 3], style: &OverlayStyle) {
        // 12 edges of a box
        let corners = [
            [min[0], min[1], min[2]], [max[0], min[1], min[2]],
            [max[0], max[1], min[2]], [min[0], max[1], min[2]],
            [min[0], min[1], max[2]], [max[0], min[1], max[2]],
            [max[0], max[1], max[2]], [min[0], max[1], max[2]],
        ];
        let edges = [
            (0,1), (1,2), (2,3), (3,0),  // bottom
            (4,5), (5,6), (6,7), (7,4),  // top
            (0,4), (1,5), (2,6), (3,7),  // verticals
        ];
        for (a, b) in edges {
            self.add_line(corners[a], corners[b], style);
        }
    }

    fn add_sphere_lines(&mut self, center: [f32; 3], radius: f32, style: &OverlayStyle) {
        // Draw 3 circles (XY, XZ, YZ planes)
        const SEGMENTS: usize = 16;
        for i in 0..SEGMENTS {
            let a0 = (i as f32 / SEGMENTS as f32) * std::f32::consts::TAU;
            let a1 = ((i + 1) as f32 / SEGMENTS as f32) * std::f32::consts::TAU;
            let (s0, c0) = (a0.sin() * radius, a0.cos() * radius);
            let (s1, c1) = (a1.sin() * radius, a1.cos() * radius);

            // XY circle
            self.add_line(
                [center[0] + c0, center[1] + s0, center[2]],
                [center[0] + c1, center[1] + s1, center[2]],
                style,
            );
            // XZ circle
            self.add_line(
                [center[0] + c0, center[1], center[2] + s0],
                [center[0] + c1, center[1], center[2] + s1],
                style,
            );
            // YZ circle
            self.add_line(
                [center[0], center[1] + c0, center[2] + s0],
                [center[0], center[1] + c1, center[2] + s1],
                style,
            );
        }
    }
}
```

### Step 5: Integrate into Graphics Pipeline

**File: `emberware-z/src/graphics/mod.rs`**

Add overlay rendering after main scene:

```rust
impl ZGraphics {
    pub fn render_frame(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        z_state: &mut ZFFIState,
        clear_color: [f32; 4],
    ) {
        // ... existing render code ...

        // Render debug overlays (after main scene, before egui)
        if let Some(overlay_buffer) = &z_state.overlay_buffer {
            self.overlay_renderer.render(
                encoder,
                &self.render_target_view,
                &self.depth_view,
                overlay_buffer,
                &self.view_proj_matrix,
            );
        }
    }
}
```

### Step 6: Clear Overlay Buffer Each Frame

**File: `emberware-z/src/app/game_session.rs`**

Clear overlays before game's render():

```rust
impl App {
    fn run_game_frame(&mut self) -> Result<(bool, bool), RuntimeError> {
        // ...

        // Clear overlay buffer before render
        if let Some(session) = &mut self.game_session {
            if let Some(game) = session.runtime.game_mut() {
                game.store_mut().data_mut().overlay_buffer.clear();
            }
        }

        // Call game's render() - this populates overlay_buffer via FFI
        session.runtime.render()?;

        // ...
    }
}
```

### File Checklist

| File | Changes |
|------|---------|
| `core/src/debug/overlay.rs` | New file: OverlayBuffer, OverlayCommand, OverlayStyle types |
| `core/src/debug/mod.rs` | Export overlay module |
| `core/src/wasm/state.rs` | Add overlay_buffer to GameStateWithConsole |
| `core/src/ffi.rs` | Add ~20 overlay FFI functions |
| `emberware-z/src/graphics/overlay_renderer.rs` | New file: wgpu overlay renderer |
| `emberware-z/src/graphics/overlay.wgsl` | New file: overlay shader |
| `emberware-z/src/graphics/mod.rs` | Integrate overlay_renderer into render_frame |
| `emberware-z/src/app/game_session.rs` | Clear overlay buffer each frame |

### Test Cases

1. **Box rendering**: Call debug_draw_box, verify wireframe box appears
2. **Sphere rendering**: Call debug_draw_sphere, verify sphere circles appear
3. **Category toggle**: Disable category, verify its overlays hidden
4. **Style push/pop**: Verify style state correctly saved/restored
5. **Depth test**: Draw overlay behind object, verify depth test works
6. **Many overlays**: Draw 1000 boxes, verify performance acceptable
7. **Text labels**: Draw 3D text, verify billboard rendering

## Future Enhancements

1. **Gizmos**: Interactive handles for editing values in-world
2. **Screenshot with overlays**: Export debug visualization
3. **Overlay recording**: Record overlay state for replay analysis
4. **Custom meshes**: Draw arbitrary debug meshes
5. **Overlay grouping**: Hierarchical categories
6. **Distance fade**: Automatically fade overlays at distance
