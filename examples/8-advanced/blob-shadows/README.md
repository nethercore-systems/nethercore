# Blob shadows (74)

A blue jumping actor carries a radial, dithered shadow on a known flat platform.
A green foreground box must occlude it. A narrow white floor inlay is intentionally
drawn **after** the shadow: it proves the shadow did not write depth. Both common
mistakes have explicit counterexample switches, rather than passing by luck.

## Controls

- **A / Z:** center, platform edge, outside lane.
- **B / X:** deliberately disable/restore the shadow's depth test.
- **X / C:** run/pause deterministic lateral motion and the jump cycle.
- **L1 / Q:** hide/show only the shadow.
- **R1 / E:** deliberately enable/disable shadow depth writes.
- **Y / V:** reset; wins over simultaneous input. Starts paused near jump apex.

Slash labels are default keyboard mappings. Held buttons do not repeatedly
toggle. The full counter and control state are rollback-covered; render only
reads state. A triangular jump trajectory keeps this a shadow lesson, not physics.

## Recipe and limits

`state::pixels()` authors one 32x32 RGBA8 radial-alpha texture. It is loaded once
in init and reused; no per-frame retained resource creation, external asset or
private dependency is involved. Texture alpha, not RGB vertex color, supplies the
radial coverage. The tiny tile occupies 4096 RGBA8 bytes before host overhead.

1. Draw the opaque platform, actor and foreground box with depth writes.
2. Project the shadow center directly onto the known y=0 receiver, adjusting
   radius and opacity heuristically with height.
3. Clip the quad to the CPU-known floor rectangle and preserve its original UVs.
   Do not stretch the radial texture to fit the clipped portion.
4. Draw immediate POS_UV triangles with depth LESS_EQUAL, depth writes **off**,
   nearest texture sampling and explicit uniform alpha/Bayer offset. The small
   receiver-normal geometry offset is 1/1024 world unit, not native depth bias.
5. Restore the ordinary pass, white texture, full opacity and zero dither offset.
   Draw the white inlay at 1/4096 above the floor. It should remain intact; enabling
   shadow depth writes punches dithered holes into it. Disabling the depth test
   instead paints shadow onto the green box.

Only this flat receiver is known. This is not runtime depth readback, arbitrary
terrain projection, multiply blending or smooth framebuffer alpha. A fixed
orthographic camera makes depth and receiver witnesses reproducible. Dyadic view
coefficients and a binary-representable caster/blob center keep the moving texture
samples away from near-boundary ambiguity; the preflight checks every motion phase
without enlarging the frozen oracle tolerances. All boxes
and planes are actual immediate 3D triangles; no new mesh API or private renderer
repair is used. The rectangular carrier has transparent radial coverage, producing
a disk rather than a rectangular shadow.

## Reproduce

```sh
nether build -p examples/8-advanced/blob-shadows -o blob.nczx
nethercore-zx blob.nczx
rustc --edition 2021 --test examples/8-advanced/blob-shadows/src/state.rs -o blob-state
./blob-state
python examples/8-advanced/blob-shadows/check_pixels.py --preflight
python examples/1-getting-started/check_foundations.py --nether PATH_TO_NETHER --player PATH_TO_ZX --output OUTSIDE_SOURCE --projects 8-advanced/blob-shadows --render
python examples/8-advanced/blob-shadows/check_pixels.py --state OUTSIDE_SOURCE/blob-shadows.json
```

The frozen independent checker uses orthographic rays, box/plane intersections,
nearest radial texels and the exact Bayer comparison. It excludes only a declared
one-pixel analytic opaque/inlay edge band and 2e-5-texel sampling ties; more than
95% of the scene interior must still be classified. Dither/alpha coverage does
not get an adjustable error band. It checks moving frames, ground contact, edge
clipping, all controls, priority reset, foreground occlusion and the late inlay.
Actual native bad-depth and bad-write captures must fail `--clean-depth IMAGE`.
The shared runner supplies relocation, false-assertion rejection and at least 120
scripted forced-rollback frames. Read the curriculum ledger for actual results;
these reproducible commands do not by themselves claim native or human acceptance.
