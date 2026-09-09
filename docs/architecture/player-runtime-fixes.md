# Player input, replay and rollback-render fixes

## Origin and scope

These changes were retained in the shared checkout separately from XM/IT compatibility.
The local-controller slot mapping originated in the April 20 Codex session
`019daa37-8d66-7930-8220-8c90e2c7f786`, fixing the requested paddle-game
joiner/controller problem and mixed local/remote player arrangements.
The replay, P2P, rendering and window fixes originated in the September 5-6
Volley Fighter build, whose brief allowed minimal demonstrated sibling-engine
blockers. The owner subsequently repaired and verified them as this named set.

- Map physical local input slots to session player handles; submit replay input
  only for locally owned players.
- Poll while P2P is synchronizing; report disconnect and desynchronization.
- Execute render while restoring WASM linear memory for rollback sessions,
  retaining host draw commands. Temporarily cap the existing RAM limiter at the
  snapshot size to prevent irreversible render-time growth; restore its ordinary
  limit before restoring bytes, including after WASM traps.
- Recover Lost/Outdated surfaces and resize. Consume the previous redraw request
  before rendering so a recovery request remains a replay advancement barrier.

Local rendering without rollback is unchanged. This is not a claim that arbitrary
render-time persistent host side effects are rolled back, nor internet/NAT or
physical multi-controller certification. Fixed-size linear-memory copying is the
bounded implementation; optimize only if profiling warrants it.

## Verification

```sh
cargo fmt --all -- --check
cargo test --offline --locked -p nethercore-core -p nethercore-zx --lib -- --test-threads=1
cargo clippy --offline --locked -p nethercore-core -p nethercore-zx --all-targets -- -D warnings
cargo build --offline --locked -p nethercore-zx --bin nethercore-zx
```

On Windows, set TEMP/TMP/TMPDIR (and lowercase variants) to the existing writable
user Temp directory, as in the established repository workflow.

Owner results: core 466 passed; ZX 504 passed, one ignored. The expanded real-WASM
render regression failed before the RAM-limit repair, then passed all growth/trap
combinations, byte restoration, retained draw calls and subsequent allowed growth.
The redraw regression covers a failed acquisition followed by a successful retry.

The freshly built native player also passed the existing Volley Fighter loopback
and window harnesses using its unchanged packaged cart:

- Two peer windows, 180 ms receiver stall, 11 rollback events, matching winners,
  remote-player rematch, no audio discontinuity warnings, safe peer disconnect.
- External-directory launch, native 1280x720 resize, keyboard training/menu/CPU/
  winner/rematch, manual serve gating and normal close.

Harness adaptation only: point PLAYER/the temporary launcher to the fresh binary;
use Down twice rather than Right twice to select the current vertical online menu.
No game source or packaged assets were changed. Source review identified the memory
 growth and redraw-barrier defects above; both received focused repairs and tests.
