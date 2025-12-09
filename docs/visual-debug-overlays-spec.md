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

## Pending Questions

### Q1: Should overlays be recorded in replays?
**Options:**
- A) No - overlays are transient debug output
- B) Yes - useful for reviewing debug visualization
- C) Separate "overlay replay" track

**Recommendation:** Option A - overlays regenerate from game state.

### Q2: Custom category naming?
Should games be able to name custom categories for the UI?

```rust
extern "C" fn debug_overlay_register_category(
    id: u32,
    name_ptr: u32, name_len: u32,
    default_color: u32,
);
```

**Recommendation:** Yes - improves discoverability.

### Q3: Per-entity overlay toggles?
Should there be a way to show/hide overlays for specific entities?
- A) Categories only (current proposal)
- B) Entity ID filtering
- C) Spatial filtering (only show overlays near cursor)

**Recommendation:** Start with A, add C later if needed.

### Q4: Overlay persistence?
Should overlays persist across frames, or be cleared each frame?
- A) Cleared each frame (current proposal) - game must redraw
- B) Persistent until explicitly cleared
- C) Time-based fade-out

**Recommendation:** Option A - simpler, matches immediate-mode pattern.

### Q5: 2D vs 3D coordinate spaces?
Current proposal has both 2D and 3D functions. Should we unify?
- A) Keep separate (current)
- B) All 3D, use z=0 for 2D
- C) All 2D with optional z

**Recommendation:** Option A - clearer intent.

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

## Future Enhancements

1. **Gizmos**: Interactive handles for editing values in-world
2. **Screenshot with overlays**: Export debug visualization
3. **Overlay recording**: Record overlay state for replay analysis
4. **Custom meshes**: Draw arbitrary debug meshes
5. **Overlay grouping**: Hierarchical categories
6. **Distance fade**: Automatically fade overlays at distance
