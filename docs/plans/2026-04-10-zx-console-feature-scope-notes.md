# ZX Console Feature Scope Notes

### Overview

This note captures feature ideas for Nethercore ZX while keeping a clear line between:

- console features
- console convenience APIs
- engine/game-specific systems

The goal is to avoid turning ZX into a general-purpose engine while still exposing the right hardware-like primitives for a PS1/PS2-era fantasy console.

### Scope Rule

A feature is a good fit for the console surface when at least one of these is true:

- it behaves like a hardware or graphics primitive
- it is a standardized asset/playback format that many games will need
- it is low-level enough that every game should not have to reinvent it

A feature is probably not a console feature when it is mainly:

- gameplay logic
- genre-specific framework code
- high-level orchestration that can be composed from lower-level primitives

---

## Agreed In-Scope Candidates

### 1. Render-To-Texture

This is a console-level graphics primitive, not engine bloat.

Why it belongs:

- enables mirrors, security cameras, minimaps, scoped views, distortion, and composited UI
- gives an alternative to stencil-only solutions for some split-screen or portal-style effects
- fits naturally alongside existing viewports, render passes, and stencil control

Questions:

- should the API expose named render targets, handles, or a small fixed slot model?
- should sampling a render target use normal texture binding or a dedicated path?
- should the first version support only color, or color + depth?

### 2. Sprite Sheet / 2D Animation

This is one of the clearest missing console-native features.

Current ZX already has low-level 2D sprite drawing, but not a standardized 2D animation format or playback model.

Why it belongs:

- common across many game types, not just one genre
- avoids every game inventing its own frame table and timing format
- pairs naturally with texture atlases already encouraged by the API

Possible minimum scope:

- atlas frame definitions
- named clips
- frame duration / FPS
- loop / once / ping-pong playback metadata
- optional origin / pivot per frame

Deliberately out of scope for v1:

- full state machines
- blend trees
- engine-side character controllers

### 3. UI / Text Layout Primitives

Current text and sprite APIs are strong enough for HUDs but still too low-level for menu-heavy games.

Why it belongs:

- menus, dialogue boxes, inventory screens, and save/load UIs are platform-wide needs
- text measurement already exists, so layout helpers are a natural extension
- this reduces repeated boilerplate without forcing a full UI framework

Likely good primitives:

- word wrap
- horizontal alignment
- vertical alignment within a rect
- clipped text regions
- text box helpers
- panel / frame helpers such as 9-slice
- pad-first focus navigation helpers if they can stay generic

### 4. Projector / Decal Primitives

This should be considered a console feature.

Important distinction:

- EPU already has a procedural "decal" concept for environment generation
- that is not the same thing as projected decals or projected textures onto scene geometry

Why it belongs:

- projectors and decals are graphics primitives, not gameplay systems
- useful for fake headlights, spell circles, damage marks, landing zones, signage, UI projections, and cheap environmental detail
- especially valuable in a retro 3D console where authored geometry is intentionally lean

Possible forms:

- simple world decal primitive
- box projector
- texture projector with limited blend modes
- fixed-function style projected quad/volume

Design concern:

- decals often need good depth handling and bias controls to avoid z-fighting, which makes this closely related to polygon/depth bias support

### 5. Polygon / Depth Bias

This is a good console primitive and should be tracked explicitly.

Why it belongs:

- solves z-fighting for coplanar or near-coplanar surfaces
- supports projected decals, layered roads, terrain markings, outlines, fake shadows, and overlay geometry
- feels like a rendering-state control rather than engine logic

Potential API shapes:

- simple per-draw bias value
- fixed categories such as none / decal / overlay / shadow
- full depth bias state if the simple model proves too limiting

Bias is especially relevant if ZX adds:

- projected decals
- blob or projected shadows
- layered environment meshes
- coplanar UI/world overlays

---

## Borderline / Optional

### Tilemap Support

This may be worth adding if ZX wants to support 2D and 2.5D games as comfortably as 3D games.

Why it is borderline:

- clearly useful and console-like
- but can also be built on top of sprite-region drawing and raw data loading

A good console-level version would focus on:

- tilemap asset format
- tile layers
- tileset references
- maybe flip/rotate flags

It should avoid bundling full map-editing or gameplay metadata systems.

### Simple 3D Animation Convenience

A full animation state machine is engine territory.

However, very small console helpers may still be reasonable, for example:

- clip sampling helpers
- clip-to-clip blend helper
- rigid/node animation playback primitives
- attachment helpers for bones or nodes

The line to avoid crossing:

- no full animation graph framework
- no full locomotion system
- no character controller logic

### Cheap Shadow Helpers

A complete shadow system is not required as a console feature.

But constrained shadow primitives may be reasonable if they stay low-level and cheap, such as:

- blob shadows
- projected shadows
- stencil-shadow-style helpers

These become more attractive if projector/decal support and depth bias exist.

---

## Likely Out Of Scope

These should generally be treated as engine/game features rather than console features:

- full animation graphs
- blend trees
- high-level cutscene systems
- dialogue frameworks
- full UI framework with widgets and retained tree layout
- full physics engine
- navmesh/pathfinding systems
- full shadow-mapping framework
- gameplay-specific entity/component architecture

---

## Working Priority

Current rough priority based on the console-vs-engine distinction:

1. render-to-texture
2. sprite sheet / 2D animation
3. UI / text layout primitives
4. projector / decal primitives
5. polygon / depth bias
6. tilemap support

This order is not final, but it keeps the focus on missing console primitives rather than engine frameworks.

---

## Possible Roadmap

### Phase 1: Low-Risk 2D/Data-Format Wins

Start with features that mostly extend asset formats and game-side helpers rather than renderer architecture.

Candidates:

- sprite sheet metadata format
- sprite animation clip format
- ROM loader support for sprite animation metadata
- helper bindings for drawing a named sprite frame or sampled clip frame
- text wrapping and alignment helpers
- 9-slice / panel helper built on sprite-region drawing

Why first:

- lower renderer risk
- immediately improves 2D games, UI, HUDs, dialogue, and menu work
- clarifies the data-pack story for higher-level 2D assets

### Phase 2: Render Target Primitive

Add a constrained render-to-texture path after the API shape is clear.

Candidates:

- fixed render-target slots or small handle table
- begin/end target rendering API
- bind render target as texture
- viewport-aware rendering into target
- optional fixed-size targets before arbitrary sizes

Why second:

- touches renderer lifetime, passes, texture handles, and capture assumptions
- should be designed alongside existing `begin_pass`, stencil, viewport, and texture binding behavior

### Phase 3: Projectors, Decals, And Depth Bias

Add the primitives that rely on predictable depth behavior.

Candidates:

- `depth_bias(...)` or a safer preset API
- simple projected decal primitive
- simple projector primitive
- maybe blob/projected shadow helpers if they fall out naturally

Why third:

- decals/projectors are more useful once depth bias exists
- bias needs careful interaction with render sorting, passes, and material state
- this phase can reuse render-to-texture only if texture projectors need it, but should not require RTT for the simplest version

### Phase 4: Optional Tilemaps And Small 3D Animation Helpers

Only do these if the previous phases show strong demand.

Candidates:

- tilemap asset format and drawing helper
- tile layer renderer
- simple 3D clip sampling helper
- rigid/node animation import/playback primitive
- bone/node attachment helper

Why later:

- these are useful, but closer to the boundary between console convenience and engine feature
- they should be added only if they stay small and generic

---

## API Sketches

These are exploratory shapes, not ABI commitments.

### Render-To-Texture

Small fixed-slot model:

```rust
// init-only, returns a texture-like handle or slot id.
render_target_create(slot: u32, width: u32, height: u32, flags: u32) -> u32

// render-time target switch.
render_target_bind(slot: u32)
render_target_clear(color: u32, depth: u32)
render_target_unbind()

// sample the target like a texture in later draws.
texture_bind(render_target_texture(slot))
```

Handle model:

```rust
render_target_create(width: u32, height: u32, flags: u32) -> u32
render_target_bind(handle: u32)
render_target_unbind()
texture_bind(handle)
```

Open questions:

- fixed slots are simpler and more console-like
- handles compose better with existing texture binding
- first version may only need color targets; depth textures can wait

### Sprite Sheet / 2D Animation

Asset-oriented model:

```rust
rom_spritesheet(id_ptr: *const u8, id_len: u32) -> u32
spritesheet_frame(sheet: u32, frame: u32, x: f32, y: f32, scale: f32)
spritesheet_frame_ex(sheet: u32, frame: u32, x: f32, y: f32, scale: f32, rotation: f32, color: u32)

rom_spriteanim(id_ptr: *const u8, id_len: u32) -> u32
spriteanim_frame(anim: u32, clip: u32, tick: u32) -> u32
spriteanim_frame_count(anim: u32, clip: u32) -> u32
```

Alternative: keep playback game-side and only standardize data:

```rust
rom_sprite_data(id_ptr: *const u8, id_len: u32) -> u32
sprite_frame_region(data: u32, frame: u32, out_rect_ptr: *mut f32)
sprite_clip_sample(data: u32, clip: u32, tick: u32) -> u32
```

Likely better for rollback:

- use deterministic `tick` or frame index supplied by the game
- avoid hidden host-side animation clocks

### UI / Text Layout

Text primitives:

```rust
text_wrap(ptr: *const u8, len: u32, max_width: f32, size: f32, out_ptr: *mut u8, max_out: u32) -> u32
draw_text_box(ptr: *const u8, len: u32, x: f32, y: f32, w: f32, h: f32, size: f32, align: u32)
draw_text_clipped(ptr: *const u8, len: u32, x: f32, y: f32, w: f32, h: f32, size: f32)
```

Panel primitive:

```rust
draw_panel_9slice(x: f32, y: f32, w: f32, h: f32, src_x: f32, src_y: f32, src_w: f32, src_h: f32, border: f32)
```

Design caution:

- avoid a retained UI tree
- keep focus/navigation helpers optional and generic if added at all

### Projector / Decal

Fixed-function decal model:

```rust
decal_bind(texture: u32)
decal_blend(mode: u32)
draw_decal_box(x: f32, y: f32, z: f32, sx: f32, sy: f32, sz: f32)
```

Projector model:

```rust
projector_bind(texture: u32)
projector_matrix(matrix_ptr: *const f32)
projector_blend(mode: u32)
draw_projector()
```

More explicit draw model:

```rust
draw_projected_quad(texture: u32, matrix_ptr: *const f32, w: f32, h: f32, blend: u32)
```

Design caution:

- keep it constrained
- do not expose a general material graph
- decide how it interacts with current render modes before adding it to the public ABI

### Polygon / Depth Bias

Simple raw state:

```rust
depth_bias(constant: f32, slope: f32)
depth_bias_clear()
```

Safer preset state:

```rust
depth_bias_mode(mode: u32)
```

Possible modes:

- `0`: none
- `1`: decal
- `2`: overlay
- `3`: shadow

Design caution:

- raw bias is powerful but easy to misuse
- preset bias is less flexible but more console-like
- bias should probably reset each frame like other render state expectations, or be clearly documented if persistent

### Tilemaps

If added, keep it data/rendering focused:

```rust
rom_tilemap(id_ptr: *const u8, id_len: u32) -> u32
tilemap_bind_tileset(texture: u32)
draw_tilemap(tilemap: u32, layer: u32, x: f32, y: f32)
draw_tilemap_region(tilemap: u32, layer: u32, tile_x: u32, tile_y: u32, tile_w: u32, tile_h: u32, x: f32, y: f32)
```

Avoid:

- collision objects
- triggers
- entity spawning
- editor-specific metadata

### Small 3D Animation Convenience

If pursued, keep it to sampling and attachments:

```rust
keyframe_sample_blend(a: u32, a_frame: u32, b: u32, b_frame: u32, t: f32, out_ptr: *mut u8)
bone_transform_read(anim: u32, frame: u32, bone: u32, out_ptr: *mut f32)
```

Avoid:

- graph definitions
- automatic transition rules
- locomotion parameters
- hidden animation clocks

---

## Open Questions

- Should ZX intentionally support both 3D-first and 2D/2.5D workflows, or stay primarily 3D-first?
- Should render-to-texture be a general primitive or a tightly constrained fixed-slot system?
- Should sprite animation assets live as standalone metadata, inside texture/atlas metadata, or inside ROM data packs as a distinct format?
- If decals/projectors are added, should they be material-driven, texture-driven, or a fixed-function primitive first?
- Should depth bias be exposed as a raw low-level knob, or as a small set of safer presets?

---

## Current Conclusion

The strongest console-surface candidates right now are:

- render-to-texture
- sprite sheet / 2D animation
- UI / text layout primitives
- projector / decal primitives
- polygon / depth bias

By contrast, full animation composition systems and full shadow systems should be treated as engine-level concerns unless reduced to small, reusable primitives.
