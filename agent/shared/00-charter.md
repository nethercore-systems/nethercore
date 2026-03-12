# Charter

## Mission

Expand `examples/3-inspectors/epu-showcase` from the current 12-preset shipping set to 20 presets and hold the result to a real showcase standard.

The shipped set must be:

- beautiful
- technically clean
- visually distinct
- useful to game developers
- deterministic
- validated by replay-driven screenshots

If screenshots have not been reviewed, the set is not done.

## Repo Facts

- Hard rules: `CLAUDE.md`, `AGENTS.md`
- Showcase root: `examples/3-inspectors/epu-showcase/`
- Shipping preset registry: `src/presets.rs`
- Preset files: `src/presets/*.rs`
- Preset helpers and opcode catalog: `src/constants.rs`
- Existing backlog: `PRESET_DESIGN_PLAN.md`
- EPU guide: `docs/book/src/guides/epu-environments.md`
- Replay schema: `core/src/replay/script/ast.rs`, `core/src/replay/script/parser.rs`
- Real player replay path: `core/src/app/player/init.rs`, `core/src/app/player/trait_impl.rs`
- Screenshot save logic: `core/src/capture.rs`

## Current Known Drift

- `src/presets.rs` currently exposes 12 presets, not the final 20.
- The full-sweep replay script is now synced to the current 12-preset roster and toggles the UI off before capture. Keep that alignment closed as the roster changes.
- `tools/nether-cli replay execute` is still not sufficient for final validation because it does not run the full real-player render path.
- The current blocker is showcase quality, not replay syntax: `Combat Lab` closes indoor proof-of-life, but outdoor proof-of-life and broader roster quality are still open.
- Authoring-surface drift is still a risk: bounds establish the world envelope, feature layers carry most readable motifs and motion, and some families are structural or static by design.

## Non-Negotiables

- Anything reachable from `update()` must remain deterministic and rollback-safe.
- Do not add wall-clock time, OS RNG, filesystem access, network access, or unordered iteration to simulation paths.
- Keep render-only behavior in `render()`.
- Keep `PRESET_COUNT`, `PRESETS`, `PRESET_NAMES`, `ANIM_SPEEDS`, and replay scripts in sync.
- Do not assume slot index enforces bounds versus feature behavior.
- Do not ship dead layers.
- Do not ship weak duplicates.
- Do not mark a preset as passing without visual screenshot review.

## Required Final Deliverables

- 20 presets wired into `src/presets.rs`
- synced names, counts, and animation speed tables
- replay scripts updated to the real preset count and capture behavior
- coverage accounting for opcode, domain, and variant usage
- capture and review logs that show the work actually happened
- doc touch-ups if showcase-facing docs become stale

## Required Mix

Across the final 20, maintain broad environmental and genre spread.

Minimum balance targets:

- at least 5 hot-leaning presets
- at least 5 cold-leaning presets
- at least 5 natural, wet, lush, or weather-driven presets
- at least 5 synthetic, industrial, corrupted, or otherworldly presets

These may overlap, but the final set must not feel samey.

## Quality Bar

Every preset must clear all of these:

- instant visual identity
- readable sky / wall / floor separation where intended
- no muddy value collapse
- no obvious seams or broken masking
- no bug-like shimmer or ugly motion
- readable reflections or ambient lighting on the hero shape
- layered effects imply a place, not random shader noise
- strong differentiation from the rest of the roster
- acceptable as a docs or marketing screenshot

Adversarial rejection prompts:

- "This looks like shader noise, not a world."
- "This is just another preset with a palette swap."
- "The horizon is unreadable."
- "The reflection read is weak."
- "The motion feels broken."
- "One or more layers are invisible in practice."

If any one of those feels true, the preset fails.
