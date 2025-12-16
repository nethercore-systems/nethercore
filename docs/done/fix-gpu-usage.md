# Fix Frame Rate: Event-Driven Rendering Instead of Continuous Polling

## Problem Summary

The app runs at 240 FPS (monitor refresh rate) because:
1. Event loop uses `ControlFlow::Poll` - continuously spins without waiting
2. `about_to_wait()` unconditionally calls `request_redraw()` every cycle
3. `render_frame()` always returns `true`
4. `request_redraw_if_needed()` has `needs_redraw = true` hardcoded

The rendering architecture itself is well-optimized (command sorting, state deduplication, buffer caching). The issue is purely the event loop spinning at max speed.

## Goal

Render only when needed:
1. **Timer-based**: When game tick says it's time to render
2. **Event-based**: When user input or egui triggers a refresh

## Files to Modify

| File | Changes |
|------|---------|
| [core/src/app/event_loop.rs](../reference/ffi.md) | Add `next_frame_time()` to trait, change ControlFlow |
| [core/src/runtime.rs](../reference/ffi.md) | Add `tick_duration()` getter |
| [emberware-z/src/app/mod.rs](../reference/emberware-z.md) | Implement smart redraw logic, track next frame time |

## Implementation Plan

### Step 1: Add `tick_duration()` getter to Runtime

In `core/src/runtime.rs`, add after line ~274:

```rust
/// Get the tick duration
pub fn tick_duration(&self) -> Duration {
    self.tick_duration
}
```

### Step 2: Add `next_frame_time()` to ConsoleApp Trait

In `core/src/app/event_loop.rs`, add to the `ConsoleApp` trait:

```rust
/// When is the next frame needed? Returns None to wait for events, Some(instant) for scheduled render.
fn next_frame_time(&self) -> Option<std::time::Instant>;
```

### Step 3: Change `about_to_wait()` to use WaitUntil

In `core/src/app/event_loop.rs`, replace lines 183-187:

```rust
fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
    if let Some(app) = &self.app {
        match app.next_frame_time() {
            Some(next_time) => {
                event_loop.set_control_flow(ControlFlow::WaitUntil(next_time));
                // Still request redraw so we wake up at the right time
                app.request_redraw();
            }
            None => {
                event_loop.set_control_flow(ControlFlow::Wait);
            }
        }
    }
}
```

### Step 4: Remove Poll from `resumed()`

In `core/src/app/event_loop.rs`, line 116, remove:
```rust
event_loop.set_control_flow(ControlFlow::Poll);
```

Also remove the same line from `run()` at line 205.

### Step 5: Add state tracking to ZApp

In `emberware-z/src/app/mod.rs`, add one new field (note: `last_frame: Instant` already exists at line 67):

```rust
struct App {
    // ... existing fields ...
    // last_frame: Instant,  // Already exists!
    next_egui_repaint: Option<Instant>,  // Add this
}
```

### Step 6: Store egui's repaint_delay

In `emberware-z/src/app/mod.rs`, after line ~462 (the repaint_delay check), capture non-zero delays:

```rust
// Check 7: Viewport repaint requested
for viewport_output in full_output.viewport_output.values() {
    if viewport_output.repaint_delay.is_zero() {
        egui_dirty = true;
    } else if !viewport_output.repaint_delay.is_max() {
        // Schedule future repaint for animations
        let repaint_at = Instant::now() + viewport_output.repaint_delay;
        self.next_egui_repaint = Some(
            self.next_egui_repaint
                .map(|t| t.min(repaint_at))
                .unwrap_or(repaint_at)
        );
    }
}
```

### Step 7: Implement `next_frame_time()` in ZApp

```rust
fn next_frame_time(&self) -> Option<Instant> {
    match &self.mode {
        AppMode::Playing { .. } => {
            // Game running: schedule next tick based on tick_duration
            if let Some(session) = &self.game_session {
                let tick_duration = session.runtime.tick_duration();
                let next_tick = self.last_frame + tick_duration;
                // Also consider egui repaints (for debug overlays)
                match self.next_egui_repaint {
                    Some(egui_time) => Some(next_tick.min(egui_time)),
                    None => Some(next_tick),
                }
            } else {
                Some(Instant::now()) // Fallback: immediate
            }
        }
        AppMode::Library | AppMode::Settings => {
            // UI only: wake on events or scheduled egui repaints
            self.next_egui_repaint
        }
    }
}
```

### Step 8: Fix render_frame return value

In `emberware-z/src/app/mod.rs` line ~657:

```rust
fn render_frame(&mut self) -> anyhow::Result<bool> {
    self.last_frame = Instant::now();  // Already updated elsewhere, but ensure it's fresh
    self.next_egui_repaint = None; // Clear for next frame's collection
    self.render();

    // Return whether we need immediate redraw (for debug stepping, transitions, etc.)
    Ok(self.needs_redraw)
}
```

### Step 9: Delete unused `request_redraw_if_needed()`

Lines 624-636 can be removed or simplified since redraw logic moves to `next_frame_time()`.

## Expected Behavior After Fix

| Mode | FPS | Behavior |
|------|-----|----------|
| Library (idle) | ~0 | Only renders on mouse/keyboard events |
| Library (hover/animation) | variable | egui schedules repaints via repaint_delay |
| Playing | 60 | Fixed timestep based on tick_duration |
| Paused | ~0 | Only on debug panel interaction |

## Risks & Mitigations

1. **Input latency**: Events still wake the loop immediately via WindowEvent, so latency unaffected
2. **Animation smoothness**: egui's repaint_delay handles UI animations properly
3. **VSync**: GPU still vsyncs on present, but we skip unnecessary frames
4. **Frame pacing**: Using WaitUntil may have slight timing jitter; if problematic, could add spin-wait for last ~1ms

## Testing

1. Monitor GPU usage in Task Manager - should drop significantly when idle
2. Check FPS counter shows ~60 during gameplay, ~0 when idle in library
3. Verify UI responsiveness unchanged (button clicks, scrolling, hover effects)
4. Test that game timing/rollback still works correctly
5. Test egui animations (tooltips fading, button hover transitions)
