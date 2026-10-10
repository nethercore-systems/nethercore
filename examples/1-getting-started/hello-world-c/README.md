# Hello World (C)

One square teaches the complete `init` / `update` / `render` loop. This is
intentionally the **same lesson and output** as the other two languages, not
three superficially different games. Requires Clang/LLD 18 or newer on PATH (freestanding WASM; no wasi-sdk or libc needed) and the current `nether`
CLI/native `nethercore-zx` player.

```sh
nether build
nether run --no-build
```

- D-pad **Up / Down**: move 10 pixels on a new press. Holding does not repeat.
- **A**: reset to y=240. Default keyboard mappings: arrows and **Z**.
- **F3** in the native player: inspect `square_y` (read-only).
- Coordinates use the fixed 960x540 framebuffer, origin at top-left. The
  64x64 square is at x=448; y is clamped to 160..384. Lines show its bounds.

`game.c` includes the repository-generated `include/zx.h`; `set_color` precedes drawing. All gameplay state changes in `update`; `render` only reads it.
The debug pointer makes the real memory-backed state available to a replay
check. There is no allocator, game framework, WASI import or external asset.
The manifest explicitly chooses mode 0, 60 Hz, one player and offline play.

## Repeatable checks

From this directory, after building:

```sh
nether replay run ../hello-world/movement.ncrs --rom hello-world-c.nczx --headless --report movement.json
nethercore-zx hello-world-c.nczx --replay ../hello-world/capture.ncrs --scale 1
```

The first script checks press versus hold, release/repress, both limits,
opposing buttons and reset through the actual guest. The second records
native images at initial, moved, clamped and reset states. Screenshots are
written to the player's normal screenshot directory (see its output).

Run the shared three-language build/relocation/negative-control check from
any directory with `python ../hello-world/check.py --nether PATH --player PATH
--output OUTSIDE_SOURCE_DIRECTORY`. It deliberately rejects a wrong assertion;
a successful script parse or guest boot alone is not behavior evidence.
Use `--render` to also run the capture replay and a 120-frame rollback sync
smoke. Logs/reports/media are QA output, not source assets to commit. All three
compilers must be available; a missing language is a failure, never a skip.

Source is covered by the repository license. This lesson does not claim online
netplay, universal cross-platform determinism, or support for every SDK call.
