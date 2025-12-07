# Emberware Suggested Tasks - Pre-Launch Roadmap

This document contains research-backed suggestions for features and improvements needed before Emberware can ship to the public. These tasks are based on analysis of successful fantasy consoles (PICO-8, TIC-80, etc.) and industry best practices.

**Research Sources:**
- [PICO-8 vs TIC-80 Comparison](https://www.slant.co/versus/9018/22511/~pico-8_vs_tic-80)
- [Fantasy Consoles Guide](https://nerdyteachers.com/Explain/FantasyConsoles/)
- [Fantasy Console Features](https://gamefromscratch.com/fantasy-console-development/)

---
### **[CRITICAL] Getting Started Experience**

**Current State:**
- README has installation instructions
- `examples/` folder has 8 example games
- No step-by-step tutorial
- No "Your First Game" walkthrough
- No interactive playground

**Why This Matters:**
First impressions determine if developers stick with a platform. PICO-8's success is partly due to its gentle learning curve and excellent documentation.

**What Successful Consoles Do:**
- **PICO-8**: Has Zine (PDF manual), video tutorials, built-in examples you can edit
- **TIC-80**: Interactive tutorials in the console itself, "Start Here" button
- **Pygame Zero**: "Zero Boilerplate" philosophy, runnable in 5 lines of code

**What We Need:**

1. **Quick Start Tutorial (Markdown + Example):**
   ```
   docs/quick-start.md (~500 lines)
   examples/quick-start/ (~100 lines of game code)
   ```

   Content:
   - "Your First Game in 15 Minutes"
   - Step-by-step: Create a bouncing ball
   - Covers: init(), update(), render(), input, drawing
   - Screenshot at each step
   - Runnable checkpoint commits

2. **Interactive Tutorial Example:**
   ```rust
   // examples/tutorial/src/lib.rs
   // A game that teaches itself through pop-up hints
   // "Press A to jump" → "Great! Now try pressing B to shoot"
   ```

3. **Template Projects:**
   ```bash
   ember new platformer  # Creates from template
   ember new shooter
   ember new puzzle
   ember new rpg
   ```

   Each template includes:
   - Basic game loop structure
   - Common patterns (player movement, collision, score)
   - Placeholder art
   - Comments explaining what to modify

4. **Video Walkthrough:**
   - 10-minute YouTube video: "Build Your First Emberware Game"
   - Shows: Installation → Code → Run → Iterate
   - Target audience: Game dev beginners

5. **Playground (Optional but Impactful):**
   - Web-based "Try Emberware Now" page
   - Code editor + live preview
   - Pre-loaded examples you can edit and run
   - No installation required
   - Like [Rust Playground](https://play.rust-lang.org/) but for Emberware

**Success Criteria:**
- ✅ Complete beginner can make a playable game in <1 hour
- ✅ Tutorial covers 80% of common FFI functions
- ✅ Template projects for 4 common genres
- ✅ Video has <5% drop-off rate (metric: watch time)
- ✅ Documentation scores 9+/10 in user surveys

**Files to Create:**
- `docs/quick-start.md` (step-by-step tutorial)
- `docs/tutorials/` (genre-specific guides)
- `examples/tutorial/` (self-teaching game)
- `templates/` (project templates)
- Update `README.md` with "New to Emberware? Start here" section

**Estimated Complexity:** Medium (2-3 weeks)

---

### **[CRITICAL] Performance Profiling & Optimization**

**Current State:**
- FPS counter in debug overlay
- No frame time breakdown
- No way to identify bottlenecks
- Games can't self-profile

**Why This Matters:**
Performance is a fantasy console's defining characteristic. PICO-8 has a stat() function for performance monitoring. Developers MUST be able to hit 60fps.

**What Successful Consoles Do:**
- **PICO-8**: `stat(1)` returns CPU usage, `stat(7)` returns FPS
- **TIC-80**: `trace()` for debugging, performance overlay

**What We Need:**

1. **FFI Performance Functions:**
   ```rust
   // Get current FPS
   fn get_fps() -> u32

   // Get frame time in microseconds
   fn get_frame_time() -> u32

   ```

2. **Debug Overlay (Already Exists):**
   Should display:
   - CPU budget: `4ms @ 60fps` with bar
   - Draw calls / frame
   - VRAM: `2.4 MB / 4 MB`
   - Network: rollback frames, ping

3. **Profiler Integration:**
   ```rust
   // In core, use `tracing` for structured profiling
   #[instrument]
   fn execute_draw_commands() {
       // Automatically tracked
   }
   ```

   Export chrome://tracing format:
   ```bash
   RUST_LOG=trace cargo run > trace.json
   # Open in chrome://tracing
   ```

4. **Performance Budget Warnings:**
   ```rust
   // In Runtime::tick()
   if update_time > tick_budget {
       warn!("Update exceeded budget: {}ms > {}ms", update_time, tick_budget);
       // Optionally: Pause game, show warning to developer
   }
   ```

5. **Benchmark Suite:**
   ```rust
   // benchmarks/graphics.rs
   // Measure draw call overhead, triangle throughput, etc.
   ```

**Success Criteria:**
- ✅ Debug overlay shows real-time stats (toggle with F3 or similar)
- ✅ Profiler identifies bottlenecks to <100μs precision
- ✅ Performance budget warnings prevent shipping slow games
- ✅ Benchmarks track performance regressions between releases

**Files to Create:**
- Add FFI functions to `core/src/ffi.rs` or `emberware-z/src/ffi/debug.rs`
- Create `emberware-z/src/debug_overlay.rs` (if doesn't exist)
- Create `benchmarks/` directory with criterion benchmarks
- Update `docs/emberware-z.md` with performance functions

**Estimated Complexity:** Low (1 week)

---

### **[CRITICAL] Comprehensive API Documentation**

**Current State:**
- `docs/ffi.md` exists (shared FFI)
- `docs/emberware-z.md` exists (Z-specific API)
- Incomplete coverage (task exists: "Document ALL FFI Functions")
- No code examples for many functions
- No searchable API reference

**Why This Matters:**
Documentation IS the product for developers. Poor docs = abandoned platform.

**What Successful Consoles Do:**
- **PICO-8**: Complete API reference, every function documented with examples
- **TIC-80**: Wiki with API docs, community contributions
- **Godot**: Searchable class reference, inline examples

**What We Need:**

1. **Complete FFI Reference:**
   - Every function documented
   - Parameters explained (types, ranges, units)
   - Return values documented
   - Example usage for each
   - Visual examples (screenshots) where applicable
   - Links to related functions

2. **Searchable Documentation:**
   ```bash
   # Use mdBook or similar
   cd docs && mdbook build
   mdbook serve  # localhost:3000
   ```

   Features:
   - Full-text search
   - Syntax highlighting
   - Copy-paste buttons
   - Mobile-friendly
   - Dark mode

3. **Code Examples Everywhere:**
   Every FFI function should have:
   ```rust
   /// Draws a sprite at the specified position.
   ///
   /// # Parameters
   /// - `texture_id`: Texture handle from load_texture()
   /// - `x`, `y`: Screen position in pixels
   /// - `width`, `height`: Size in pixels
   ///
   /// # Example
   /// ```rust
   /// let player_tex = load_texture(PLAYER_DATA, 32, 32);
   ///
   /// fn render() {
   ///     draw_sprite(player_tex, 100, 100, 32, 32);
   /// }
   /// ```
   fn draw_sprite(texture_id: u32, x: f32, y: f32, width: f32, height: f32)
   ```

4. **Visual API Cheat Sheet:**
   - One-page PDF with all functions grouped by category
   - Printable reference
   - Like PICO-8's cheat sheet

5. **Auto-Generated Docs:**
   ```bash
   # Extract doc comments from FFI functions
   cargo doc --no-deps --document-private-items
   # Convert to markdown or mdBook format
   ```

**Success Criteria:**
- ✅ 100% FFI function coverage (67 functions)
- ✅ Every function has at least one code example
- ✅ Searchable online documentation
- ✅ Cheat sheet PDF available
- ✅ Zero "How do I...?" questions answered with "See docs"

**Files to Create/Modify:**
- Complete `docs/ffi.md` and `docs/emberware-z.md`
- Create `docs/book/` (mdBook structure)
- Create `docs/cheat-sheet.pdf` (visual reference)
- Add doc comments to all FFI functions in `emberware-z/src/ffi/mod.rs`

**Estimated Complexity:** Medium (2 weeks)

---

### **[CRITICAL] Cart/Distribution Format**

**Current State:**
- Games are compiled to `game.wasm` files
- No standardized packaging
- No metadata (title, author, description, thumbnail)
- No version information
- No dependency tracking

**Why This Matters:**
Fantasy consoles use "cartridges" as a distribution format. This enables:
- Sharing games easily (one file)
- Metadata for game browsers/stores
- Version management
- Asset bundling

**What Successful Consoles Do:**
- **PICO-8**: `.p8` text files or `.p8.png` (code hidden in PNG)
- **TIC-80**: `.tic` binary cartridges with embedded assets
- **Pixel Vision 8**: `.pv8` zip archives

**Implementation:**

1. **EmberwareZCart Format (`.ewz` ):**
   ```rust
   // Binary format
   struct EmberwareZCart {
       version: u32,             // Cart format version

       // Metadata
       title: String,
       author: String,
       description: String,
       thumbnail: Vec<u8>,       // PNG thumbnail (128x128)
       tags: Vec<String>,
       created: DateTime,

       // Content
       code: Vec<u8>,     // Compiled game
       // Embedded assets - Later
   }
   ```

2. **Cart Builder CLI:**
   ```bash
   ember cart create my-game.wasm \
     --title "Super Platformer" \
     --author "YourName" \
     --description "A platforming adventure" \
     --thumbnail icon.png \
     --tags "platformer,action" \
     --output game.embercart
   ```

3. **Cart Loading:**
   ```rust
   // In core, can use bitcode for optimal compressing and faser than serde
   fn load_cart(path: &Path) -> Result<T> {
       let bytes = fs::read(path)?;
       T::deserialize(&bytes)
   }

   // Validate and run
   ```

4. **Cart Signing (Future):**
   ```rust
   // Optional: Cryptographic signatures for verified authors
   signature: Option<Vec<u8>>,
   public_key: Option<Vec<u8>>,
   ```

5. **Human-Readable Alternative (.yaml + .wasm):**
   ```yaml
   # game.ember
   title: Super Platformer
   author: YourName
   description: A platforming adventure
   thumbnail: icon.png
   tags: [platformer, action]
   wasm: game.wasm
   ```

   Easier for version control, human editing.

6. **Include generics for Console Abstraction**
   - Reusable loading across different kinds of consoles in future

**Success Criteria:**
- ✅ Cart format specification document
- ✅ `ember cart` CLI tool for creating/extracting carts
- ✅ Emberware runtime can load .embercart files
- ✅ Platform API accepts cart uploads
- ✅ Thumbnail, metadata displayed in library UI

**Files to Create:**
- `shared/src/cart.rs` (cart format definition, one per console)
- `xtask/src/cart.rs` (cart builder CLI)
- Update `emberware-z/src/app.rs` to load .embercart files
- `docs/cart-format.md` (specification)

**Estimated Complexity:** Medium (1-2 weeks)

---

### **[HIGH] Screenshot & GIF Recording**

**Current State:**
- No way to capture gameplay
- Developers can't create marketing materials in-engine
- Social sharing requires external tools

**Why This Matters:**
"Pics or it didn't happen." Games need shareability for viral growth.

**What Successful Consoles Do:**
- **PICO-8**: F6 saves screenshot, F8 starts/stops GIF recording
- **TIC-80**: F6 screenshot, F7 GIF recording (8 seconds, 30fps)

**Implementation:**

1. **Screenshot (PNG):**
   ```rust
   // FFI function
   fn screenshot(filename_ptr: u32, filename_len: u32) {
       // Capture framebuffer
       // Save as PNG to ~/.emberware/screenshots/
   }

   // Or keyboard shortcut: F9
   ```

2. **GIF Recording:**
   ```rust
   // Record at 30fps for max 60 seconds
   // Save to ~/.emberware/gifs/

   // Use `gif` crate for encoding
   // Keyboard: F10 to start/stop
   ```

3. **FFI Control:**
   ```rust
   fn start_recording()
   fn stop_recording() -> u32  // Returns duration in frames
   fn is_recording() -> u32
   ```

4. **Watermark (Optional):**
   ```
   Bottom-right corner: "Made with Emberware"
   Configurable in settings
   ```

5. **Upload Integration (Future):**
   ```rust
   // Upload to emberware.io/share
   fn share_screenshot() -> String  // Returns URL
   ```

**Success Criteria:**
- ✅ F9 saves screenshot to local directory
- ✅ F10 records GIF (max 60s, 30fps, optimized size)
- ✅ FFI functions for programmatic capture
- ✅ GIFs <5MB for typical 10-second gameplay
- ✅ Screenshots saved as PNG with timestamp

**Files to Create/Modify:**
- Add to `emberware-z/src/app.rs` (keyboard shortcuts)
- Create `emberware-z/src/capture.rs` (screenshot/GIF logic)
- Add FFI in `emberware-z/src/ffi/debug.rs`

**Estimated Complexity:** Low (3-5 days)

---

### **[HIGH] Error Recovery & Better Panics**

**Current State:**
- Game panic = crash entire Emberware runtime
- No error boundary
- Unhelpful error messages for game developers

**Why This Matters:**
Crashing the entire console because of a game bug is user-hostile. Games should fail gracefully.

**What We Need:**

1. **Trap Handling:**
   ```rust
   // In core/src/runtime.rs
   match game.call_update() {
       Ok(_) => { /* Continue */ }
       Err(trap) => {
           // Don't crash runtime!
           self.state = RuntimeState::Error(trap);
           // Show error screen
       }
   }
   ```

2. **Error Screen UI:**
   ```
   ┌─────────────────────────────────────┐
   │  ⚠️  Game Error                      │
   │                                     │
   │  The game encountered an error:     │
   │                                     │
   │  > WASM trap: out of bounds memory │
   │    access at offset 0x1234         │
   │                                     │
   │  Stack trace:                       │
   │  - update() at offset 0x5678       │
   │  - check_collision() at 0x9ABC     │
   │                                     │
   │  [View Full Log] [Restart] [Quit]  │
   └─────────────────────────────────────┘
   ```

3. **WASM Source Maps:**
   ```bash
   # Include debug info in WASM
   cargo build --target wasm32-unknown-unknown \
     -Z build-std=std,panic_abort

   # Generate source map
   wasm-sourcemap game.wasm -o game.wasm.map
   ```

   Map WASM offsets → Rust source lines.

4. **FFI Error Injection (Debug Mode):**
   ```rust
   #[cfg(debug_assertions)]
   fn trigger_error(message_ptr: u32, message_len: u32) {
       // Developers can test error handling
   }
   ```

5. **Helpful Error Messages:**
   ```
   ❌ BAD:  "WASM trap"
   ✅ GOOD: "Memory access out of bounds at address 0x1234.
            Your game tried to read 4 bytes at offset 0x1230,
            but WASM memory is only 0x1000 bytes.

            Possible causes:
            - Array index out of bounds
            - Use after free (in unsafe code)

            See: https://emberware.io/docs/errors/oob"
   ```

**Success Criteria:**
- ✅ Game errors don't crash Emberware runtime
- ✅ Error screen shows helpful diagnostic info
- ✅ Source maps link errors to source code lines
- ✅ Developers can trigger test errors
- ✅ Error messages link to documentation

**Files to Modify:**
- `core/src/runtime.rs` (trap handling)
- `emberware-z/src/app.rs` (error screen UI)
- `core/src/wasm/mod.rs` (source map integration)

**Estimated Complexity:** Medium (1 week)

---

## Medium Priority

### **[MEDIUM] Memory Inspector / Debug Tools**

**Current State:**
- No way to inspect WASM memory at runtime
- Can't view game state variables
- No breakpoints or step debugging

**Why This Matters:**
Debugging is hard without visibility into runtime state. Must be implemented at generic level.

**What Successful Consoles Do:**
- **PICO-8**: Immediate mode console, print debugging
- **TIC-80**: `trace()` function for logging
- **Browser DevTools**: Memory inspector, breakpoints

**What We Need:**

1. **Memory Viewer UI:**
   ```
   ┌─────────────────────────────────────┐
   │  Memory Inspector      [0x0000]     │
   ├─────────────────────────────────────┤
   │  Offset  | Hex              | ASCII │
   │  0x0000  | 48 65 6C 6C 6F   | Hello │
   │  0x0005  | 00 01 02 03 04   | ..... │
   │  ...                                │
   └─────────────────────────────────────┘
   ```

2. **Watch Variables:**
   ```rust
   // FFI function
   fn watch(name_ptr: u32, name_len: u32, addr: u32, size: u32) {
       // Display variable in debug overlay
   }

   // Usage in game:
   watch("player_x", &player.x as *const _ as u32, 4);
   ```

3. **Console/REPL (Future):**
   ```
   > print memory[0x1000..0x1010]
   [48, 65, 6C, 6C, 6F, 00, 01, 02, 03, 04, 05, 06, 07, 08, 09, 0A]

   > set memory[0x2000] = 42
   Done
   ```

4. **Trace Logging:**
   ```rust
   fn trace(message_ptr: u32, message_len: u32) {
       // Logged to file: ~/.emberware/logs/game.log
       // Shown in console overlay (if enabled)
   }
   ```

**Success Criteria:**
- ✅ Memory viewer accessible via debug UI (F12?)
- ✅ Can watch specific memory addresses
- ✅ Trace logs visible in overlay and saved to file
- ✅ Performance impact <5% when debugging disabled

**Files to Create:**
- `emberware-z/src/debug/memory_viewer.rs`
- Add FFI functions in `ffi/debug.rs`

**Estimated Complexity:** Medium (1 week)

---

### **[MEDIUM] Localization Support**

**Current State:**
- No text rendering beyond basic ASCII
- No internationalization (i18n) support
- Hard-coded English strings

**Why This Matters:**
Global audience. Non-English speakers are 75% of potential users.

**What We Need:**

1. **Unicode Text Rendering:**
   ```rust
   // Already using String in Rust (UTF-8)
   // But font rendering may not support it

   // In font.rs:
   // Add Unicode glyph support
   // Use rusttype or similar for TTF rendering
   ```

2. **i18n FFI Functions:**
   ```rust
   fn set_language(lang_ptr: u32, lang_len: u32)  // "en", "ja", "es"
   fn get_text(key_ptr: u32, key_len: u32) -> u32  // Returns string pointer
   ```

3. **String Table Format:**
   ```toml
   # strings/en.toml
   [menu]
   start = "Start Game"
   quit = "Quit"

   # strings/ja.toml
   [menu]
   start = "ゲーム開始"
   quit = "終了"
   ```

4. **Font Support:**
   - Include default font with Latin, Cyrillic, CJK
   - Or allow loading custom fonts

**Success Criteria:**
- ✅ Games can render Japanese, Arabic, Emoji
- ✅ i18n helper functions available
- ✅ Example game demonstrating localization

**Files to Create:**
- Update `emberware-z/src/font.rs` (Unicode support)
- Add `ffi/i18n.rs` (localization functions)
- Create `examples/i18n-demo/`

**Estimated Complexity:** Medium (1-2 weeks)

---

### **[LOW] Save State / Replay System**

**Current State:**
- Games can save data via `save()` / `load()`
- No automatic save states
- No replay recording

**Why This Matters:**
- Quality of life for players (save anywhere)
- Useful for demos, speedruns, bug reports

**Implementation:**
```rust
// Automatic save states on F5
fn save_state(slot: u32) {
    // Serialize entire WASM memory
    // Save input history for deterministic replay
}

fn load_state(slot: u32) {
    // Restore WASM memory
    // Fast-forward with saved inputs
}
```

**Estimated Complexity:** Low (3-5 days)

---

### **[LOW] Social Features (Likes, Comments, Follows)**

**Current State:**
- Private `emberware-platform` backend exists
- No public game browser yet
- No social graph

**Why This Matters:**
Community drives adoption. PICO-8 BBS and itch.io are central to their ecosystems.

**What We Need:**
- Public game browser (emberware.io/games)
- User profiles
- Like/favorite games
- Comments/ratings
- Follow creators
- Jam hosting (game jams)

**Note:** This is platform work, not core console work.

**Estimated Complexity:** High (4-8 weeks, full-stack)

---

## Summary Table

| Task | Priority | Complexity | Est. Time | Blockers |
|------|----------|------------|-----------|----------|
| Web Player | CRITICAL | High | 4-6 weeks | wgpu WebGL, nested WASM |
| Getting Started | CRITICAL | Medium | 2-3 weeks | None |
| Performance Tools | CRITICAL | Low | 1 week | None |
| API Docs | CRITICAL | Medium | 2 weeks | None |
| Cart Format | CRITICAL | Medium | 1-2 weeks | None |
| Hot Reload | HIGH | Medium | 1 week | None |
| Screenshot/GIF | HIGH | Low | 3-5 days | None |
| Error Recovery | HIGH | Medium | 1 week | None |
| Memory Inspector | MEDIUM | Medium | 1 week | None |
| Localization | MEDIUM | Medium | 1-2 weeks | Unicode fonts |
| Save States | LOW | Low | 3-5 days | None |
| Social Features | LOW | High | 4-8 weeks | Platform backend |

---

**Last Updated:** 2025-12-06
