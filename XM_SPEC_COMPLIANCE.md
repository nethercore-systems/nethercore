# XM Spec Compliance Report

> Reference: [libxm](https://github.com/Artefact2/libxm), [OpenMPT Wiki](https://wiki.openmpt.org/Manual:_Effect_Reference)
>
> File: `nethercore-zx/src/tracker.rs`

## Priority Legend

- 🟡 **MEDIUM** - Subtle difference, affects some songs
- 🟢 **LOW** - Minor deviation, edge cases only

---

## Remaining Issues

### 1. Pattern Delay (EEx) - Row Advancement 🟡 MEDIUM

**Status:** Stored but not applied during playback

The `pattern_delay` and `pattern_delay_count` fields exist (lines 87-90) and tick-0 handling stores the value (lines 916-921), but `render_sample_and_advance()` doesn't check these fields during row advancement (lines 1346-1398).

**Fix:** In row advancement logic, check if `pattern_delay_count < pattern_delay` before advancing to next row. Increment `pattern_delay_count` each time the row would advance, only actually advance when counter reaches delay value.

---

### 2. High Sample Offset (SAx) 🟡 MEDIUM

**Status:** Field exists but not used

The `sample_offset_high` field exists (line 227) but the 9xx sample offset effect (lines 814-818) only uses the low byte:

```rust
channel.sample_pos = (channel.last_sample_offset as u32 * 256) as f64;
```

**Fix:** Combine high and low bytes:
```rust
let offset = ((channel.sample_offset_high as u32) << 16) | ((channel.last_sample_offset as u32) << 8);
channel.sample_pos = offset as f64;
```

Also need to implement the SAx extended command to set `sample_offset_high`.

---

### 3. Multi-Retrigger Multiplicative Volume (Rxy) 🟡 MEDIUM

**Status:** Multiplicative options not supported

Lines 965-973 use additive approximation (0) for multiplicative volume changes:

```rust
6 => 0,  // Should be * 2/3
7 => 0,  // Should be * 1/2
14 => 0, // Should be * 3/2
15 => 0, // Should be * 2
```

**Fix:** Apply multiplicative volume changes in retrigger processing:
```rust
6 => { channel.volume *= 2.0 / 3.0; }
7 => { channel.volume *= 0.5; }
14 => { channel.volume *= 1.5; }
15 => { channel.volume *= 2.0; }
```

---

### 4. Fine vs Extra-Fine Portamento 🟢 LOW

**Status:** Both use same scaling

Current implementation uses `* 4.0` for both fine portamento (E1x/E2x) and regular portamento.

libxm distinguishes:
- Fine portamento: `* 4`
- Extra-fine portamento (X1x/X2x): `* 1`

**Impact:** Minor - extra-fine portamento commands are rarely used.

---

## References

- [libxm source](https://github.com/Artefact2/libxm)
- [ft2play (8bitbubsy)](https://github.com/8bitbubsy/ft2play) - Direct FT2 port
- [OpenMPT Effect Reference](https://wiki.openmpt.org/Manual:_Effect_Reference)
- [OpenMPT Test Cases](https://wiki.openmpt.org/Development:_Test_Cases/XM)
