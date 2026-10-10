# Transform Composition Lab

Two bars show the difference between **T*R** and **R*T**. A two-joint crane
shows the smallest useful hierarchy: retain the parent matrix in guest code,
compose the child explicitly, and set each model before drawing it.

## Controls

- A / keyboard Z: advance the parent rotation by 45 degrees (eight stops).
- B / keyboard X: flip the child-relative angle between +45 and -45 degrees.
- X / keyboard C: switch the two comparison bars between API calls and explicit guest matrices. The pictures should agree.
- Y / keyboard V: reset to 45 degrees, +45-degree elbow and API mode.
- F3: the player's state/debug overlay exposes step, flip, explicit, orbit center, elbow and tip coordinates.

Button edges change state in `update`; held A advances only once. `render` never
advances simulation. This is an offline, one-player, 60 Hz, mode-0 lesson.

## Read the labels carefully

Matrices use **column-major storage and column vectors**. The rightmost matrix
acts on the point first. After `push_identity()`, calls `push_translate(T)` then
`push_rotate_z(R)` create **T*R**, so a point rotates *then* translates. Reversing
those API calls creates **R*T**, so the translated point orbits the origin.
The left comparison has an additional -7 world-unit panel offset; subtract it
when comparing its local center (2,0) with the orbiting center. The crane's base
is at (7,0). All white crosses mark z=0 reference points, not front-face pixels.

`push_*` post-multiplies the current transform. There is no generic push/pop
hierarchy stack. The crane uses `parent = Tbase * Rparent` and
`child = parent * T(1.5,0,0) * Rchild`. The first bar's origin is at its midpoint;
crane bars add a half-length translation so their local origin stays at a joint.

`composition.rs` is a tiny, lesson-local matrix recipe, not a general math or
scene-graph library. Eight explicit sine/cosine stops keep the boundary examples
repeatable; the native image checker uses independent Python trigonometry.
A built-in cube is created once in `init` with **half-extents** (0.5,0.5,0.5).
Models add their own scale. No imported or private assets are required; all
editable source follows this repository's license.

## Build and run

From this directory with the repository tools on PATH:

```sh
rustup target add wasm32-unknown-unknown
nether build
nether run --no-build
```

Build uses the crate's explicit WASM output. No generated source or asset step
is required. Copy the resulting `.nczx` outside the checkout and run it with
`nethercore-zx path/to/transform-composition-lab.nczx`.

## Runnable checks

From this directory:

```sh
rustc --edition 2021 --test src/composition.rs -o composition-tests
./composition-tests
```

Windows appends `.exe`. The check covers column-major multiplication, identity,
noncommuting scale/translation, the two quarter-turn centers, child-relative
rotation, fixed elbow, arm length and wrap. Do not commit the test executable.

From the repository root (absolute tool paths and an empty output directory
outside the checkout):

```sh
python examples/1-getting-started/check_foundations.py --projects 2-graphics/transform-composition-lab --nether PATH/TO/nether --player PATH/TO/nethercore-zx --output OUTSIDE_CHECKOUT --render
```

`witness.ncrs` checks exact state/held-button/reset boundaries;
`capture.ncrs` produces all eight angles in both API and matrix modes, the
independent child toggle, and reset. The shared runner checks packaging,
headless state, deliberately false assertion rejection, native pixels and
scripted forced rollback. `check_pixels.py` compares projected box silhouettes
with independent scalar trigonometry, checks fixed joints, API/matrix agreement,
and rejects repeated idle frames. Predeclared raster tolerance is **3 pixels**
per silhouette bound; matched API/matrix coverage may differ at **0.5%** of scene
pixels due to floating-point edge rasterization. World-state tolerance is
**0.0001 units**. Reset/wrap are exact pixel comparisons. These checks do not
establish human understanding, taste, actual network play, or G5 delivery.
