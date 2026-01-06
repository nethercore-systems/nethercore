# Determinism Audit Checklist (Rollback-Reachable Code)

Use this checklist for any code path that can be executed during rollback or replay.
If an item is not applicable, mark it as N/A and note why.

## Inputs and Timing
- [ ] Uses only frame/tick counters for time; no wall-clock time (`Instant`, `SystemTime`).
- [ ] No frame-rate dependent math without fixed timestep inputs.
- [ ] Any timing-derived values are derived from console specs or session config.

## Randomness
- [ ] Uses deterministic RNG seeded from session/ROM metadata.
- [ ] No OS RNG (`rand::random`, `thread_rng`, `getrandom`) in rollback paths.

## External IO
- [ ] No filesystem reads/writes during rollback paths.
- [ ] No network calls during rollback paths.
- [ ] No environment variable reads that affect simulation state.

## Threading and Concurrency
- [ ] No reliance on thread scheduling, locks, or channels for simulation state.
- [ ] No cross-thread mutation of state used in rollback.

## Data Structures and Ordering
- [ ] Iteration order is deterministic (avoid `HashMap`/`HashSet` iteration in core state).
- [ ] Sorting uses stable, explicit keys.
- [ ] Any serialization/deserialization uses deterministic ordering.

## Floating Point and Math
- [ ] Floating-point operations are consistent across platforms (avoid NaN/Inf divergence).
- [ ] Uses fixed-point or clamped math where determinism is critical.
- [ ] No reliance on CPU-specific math intrinsics.

## FFI and Host Boundaries
- [ ] FFI calls do not access non-deterministic host state.
- [ ] Host-provided data used by simulation is captured and replayed deterministically.

## Logging and Debug
- [ ] Logging does not read or mutate simulation state.
- [ ] Debug-only code does not affect simulation outcomes.

## Testing
- [ ] Add or update a determinism test where possible.
- [ ] Replay and rollback tests cover the modified path.
