# XM Sample Auto-Extraction Proposal

> **Status**: Proposal
> **Author**: Claude Code
> **Date**: 2025-12-30

## Problem Statement

The current XM music import flow is tedious and error-prone:

1. **Manual sample export** - Every sample must be individually exported from the tracker (MilkyTracker/OpenMPT) and saved as a WAV file
2. **Manual manifest entries** - Each sample must be declared in `nether.toml` under `[[assets.sounds]]`
3. **Exact name matching** - XM instrument names must exactly match the sound IDs (case-sensitive)
4. **Late validation** - Errors only surface at `nether pack` time, not during composition
5. **No deduplication** - If multiple tracks use the same kick drum, it must be declared separately

**Current workflow:**
```
Full XM (with samples) → [Manual Export] → WAV files → nether.toml [[assets.sounds]]
                     ↓
              nether pack → Strips samples from XM → Validates instrument names → ROM
```

## Proposed Solution

Enable **automatic sample extraction** from XM files at pack time:

```
Full XM (with embedded samples) → nether pack → [Smart Extract] → Samples to datapack
                                           → [Strip] → Minimal XM patterns
                                           → [Match] → Instrument → sample references
```

## Key Features

### 1. Smart Sample Extraction
When packing a tracker with `extract_samples = true`:
- Parse full XM including embedded sample data
- Convert samples to 22050Hz mono i16 (nethercore standard format)
- Generate sample IDs from instrument names

### 2. Hash-Based Deduplication
- Compute content hash (SHA-256) for each extracted sample
- If same hash seen across multiple XM files → single `PackedSound` entry
- Dramatically reduces ROM size when tracks share samples (common with sample packs)

### 3. Named Access via `rom_sound()`
- XM instrument names become valid `rom_sound()` IDs
- Sanitization: `"  My Kick!  "` → `"my_kick"`
- Empty names auto-generated: `""` → `"trackername_inst0"`

### 4. Collision Detection
- If manual `[[assets.sounds]]` has same name as XM instrument → **error**
- Same instrument name across XM files with different content → **error**
- Same instrument name across XM files with same content → deduplicated, no error

### 5. Bulk Import Pattern
XM files with empty patterns can serve as sample libraries:
```toml
[[assets.trackers]]
id = "sample_bank"
path = "assets/samples.xm"
extract_samples = true
patterns = false  # Only extract samples, don't register as playable tracker
```

## Manifest Changes

```toml
# New fields for [[assets.trackers]] entries:

[[assets.trackers]]
id = "boss_theme"
path = "music/boss_theme.xm"
extract_samples = true  # NEW: Auto-extract samples from XM (default: false)

# Optional: global setting
[game]
auto_extract_xm_samples = true  # Apply to all trackers
```

## Technical Implementation

### Phase 1: Sample Extraction API (`nether-xm`)

**File: `nether-xm/src/extract.rs` (new)**

```rust
/// Extracted sample data from XM instrument
pub struct ExtractedSample {
    pub instrument_index: u8,
    pub name: String,
    pub sample_rate: u32,      // XM finetune → sample rate
    pub bit_depth: u8,         // 8 or 16
    pub loop_start: u32,
    pub loop_length: u32,
    pub loop_type: u8,         // 0=none, 1=forward, 2=pingpong
    pub data: Vec<i16>,        // Decoded and converted to i16
}

/// Extract all samples from an XM file
pub fn extract_samples(data: &[u8]) -> Result<Vec<ExtractedSample>, XmError>;
```

**Key changes to `parser.rs`:**
- Currently line 388-389 skips sample data: `cursor.seek(SeekFrom::Current(sample_length as i64))?`
- New function reads and decodes delta-encoded sample data

**XM Sample Decoding:**
- 8-bit samples: delta-encoded signed bytes → i8 → i16 (scale to 16-bit range)
- 16-bit samples: delta-encoded i16 pairs → i16

### Phase 2: Audio Conversion (`nether-cli`)

**File: `tools/nether-cli/src/audio_convert.rs` (new)**

```rust
/// Resample audio to 22050 Hz using linear interpolation
pub fn resample_to_22050(samples: &[i16], source_rate: u32) -> Vec<i16>;

/// Convert stereo to mono by averaging channels
pub fn stereo_to_mono(samples: &[i16]) -> Vec<i16>;

/// Full conversion pipeline
pub fn convert_xm_sample(sample: &ExtractedSample) -> Vec<i16> {
    let resampled = resample_to_22050(&sample.data, sample.sample_rate);
    // XM samples are mono, no stereo conversion needed
    resampled
}
```

### Phase 3: Pack Integration (`nether-cli`)

**File: `tools/nether-cli/src/manifest.rs`**

Add to `AssetEntry`:
```rust
#[derive(Debug, Deserialize)]
pub struct AssetEntry {
    pub id: String,
    pub path: String,
    // ... existing fields ...

    /// Extract samples from XM file (trackers only)
    #[serde(default)]
    pub extract_samples: Option<bool>,

    /// Only extract samples, don't register as playable tracker
    #[serde(default)]
    pub patterns: Option<bool>,  // default true
}
```

**File: `tools/nether-cli/src/pack.rs`**

Updated `load_assets()` flow:

```rust
fn load_assets(...) -> Result<ZXDataPack> {
    // 1. Load explicit sounds first
    let explicit_sounds = load_explicit_sounds(&assets.sounds)?;
    let mut sound_map: HashMap<String, PackedSound> = explicit_sounds
        .into_iter()
        .map(|s| (s.id.clone(), s))
        .collect();

    // 2. Track content hashes for deduplication
    let mut hash_to_id: HashMap<[u8; 32], String> = HashMap::new();

    // 3. Extract samples from XM files (if extract_samples = true)
    for entry in &assets.trackers {
        if entry.extract_samples.unwrap_or(false) {
            let samples = nether_xm::extract_samples(&xm_data)?;

            for sample in samples {
                let converted = audio_convert::convert_xm_sample(&sample);
                let hash = sha256(&converted);
                let name = sanitize_name(&sample.name, &entry.id, sample.instrument_index);

                // Check for collision with explicit sounds
                if sound_map.contains_key(&name) {
                    return Err(anyhow!(
                        "Collision: XM instrument '{}' conflicts with [[assets.sounds]]",
                        name
                    ));
                }

                // Check for content mismatch with same name
                if let Some(existing) = sound_map.get(&name) {
                    let existing_hash = sha256(&existing.data);
                    if existing_hash != hash {
                        return Err(anyhow!(
                            "Conflict: instrument '{}' has different content in multiple XM files",
                            name
                        ));
                    }
                    // Same content = deduplicated, continue
                    continue;
                }

                // Check for hash match (same content, different name)
                if let Some(existing_name) = hash_to_id.get(&hash) {
                    // Alias: same content already exists under different name
                    println!("  Note: '{}' is identical to '{}', deduplicating", name, existing_name);
                }

                // Add new sample
                sound_map.insert(name.clone(), PackedSound {
                    id: name.clone(),
                    data: converted,
                });
                hash_to_id.insert(hash, name);
            }
        }
    }

    // 4. Load trackers (validation against sound_map instead of explicit sounds only)
    // ...
}
```

**Name Sanitization:**
```rust
fn sanitize_name(name: &str, tracker_id: &str, index: u8) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return format!("{}_inst{}", tracker_id, index);
    }

    trimmed
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}
```

## Files to Modify

| File | Change |
|------|--------|
| `nether-xm/src/extract.rs` | NEW: Sample extraction API |
| `nether-xm/src/parser.rs` | Add sample data reading (currently skipped at line 388-389) |
| `nether-xm/src/lib.rs` | Export new extraction API |
| `tools/nether-cli/src/manifest.rs` | Add `extract_samples`, `patterns` fields to `AssetEntry` |
| `tools/nether-cli/src/pack.rs` | Add extraction, dedup, collision logic |
| `tools/nether-cli/src/audio_convert.rs` | NEW: Resampling to 22050Hz |
| `zx-common/src/formats/zx_data_pack.rs` | No changes (structs unchanged) |
| `nethercore-zx/src/ffi/audio.rs` | No changes (runtime unchanged) |

## Migration Path

### Backwards Compatibility
- `extract_samples` defaults to `false` - existing projects work unchanged
- Can mix manual `[[assets.sounds]]` with auto-extracted samples
- Gradual adoption: enable `extract_samples` on one tracker at a time

### Deprecation Timeline
1. **v1.0**: Feature released with `extract_samples = false` default
2. **v1.1**: Warning when trackers have embedded samples but `extract_samples = false`
3. **v2.0**: Consider changing default to `true`

## AI Plugin Updates

### `zx-procgen/skills/procedural-music/SKILL.md`

Update workflow documentation:
- Remove manual sample export steps
- Add `extract_samples = true` to examples
- Document bulk import pattern

### `zx-procgen/skills/xm-writer/SKILL.md`

Update Python XM writer:
- Document that generated XM can include embedded samples
- Add helper for setting instrument names that become `rom_sound()` IDs

## Example: New Workflow

**Before (current, tedious):**
```toml
# 1. Export each sample from MilkyTracker manually
# 2. Save to assets/kick.wav, assets/snare.wav, etc.
# 3. Add to nether.toml:
[[assets.sounds]]
id = "kick"
path = "assets/kick.wav"

[[assets.sounds]]
id = "snare"
path = "assets/snare.wav"

# ... repeat for every sample ...

[[assets.trackers]]
id = "boss_theme"
path = "music/boss_theme.xm"
```

**After (proposed, simple):**
```toml
# 1. Just save the XM file (samples embedded)
# 2. Add to nether.toml:
[[assets.trackers]]
id = "boss_theme"
path = "music/boss_theme.xm"
extract_samples = true

# Done! Samples auto-extracted, named by instrument names
```

## Test Cases

1. **Basic extraction**: XM with 3 instruments → 3 `PackedSound` entries
2. **Deduplication**: 2 XMs sharing same kick → 1 `PackedSound`
3. **Name collision error**: XM instrument + `[[assets.sounds]]` same name → error
4. **Content mismatch error**: Same name, different content across XM files → error
5. **Sanitization**: `"  My Kick!  "` → `"my_kick"`
6. **Empty name**: `""` → `"trackername_inst0"`
7. **Bulk import**: XM with empty patterns, `patterns = false` → samples only
8. **Backwards compat**: `extract_samples = false` (default) works as before

## Summary

This change transforms the music workflow from:
- **Manual**: Export N samples → N manifest entries → hope names match
- **Automatic**: Add XM → `extract_samples = true` → done

**Benefits:**
- Eliminates most common source of music bugs (name mismatches)
- Enables sample deduplication (smaller ROMs)
- Unlocks bulk import pattern (XM as sample organizer)
- Better DevX with faster iteration

## Implementation Checklist

- [ ] Phase 1: Sample extraction API in `nether-xm`
  - [ ] Add `ExtractedSample` struct
  - [ ] Implement `extract_samples()` function
  - [ ] Handle delta-encoded 8-bit and 16-bit samples
  - [ ] Add tests with existing XM files
- [ ] Phase 2: Audio conversion in `nether-cli`
  - [ ] Create `audio_convert.rs` module
  - [ ] Implement `resample_to_22050()`
  - [ ] Add tests for resampling quality
- [ ] Phase 3: Pack integration
  - [ ] Add `extract_samples` field to manifest
  - [ ] Add `patterns` field for bulk import mode
  - [ ] Implement hash-based deduplication
  - [ ] Implement collision detection
  - [ ] Update validation to use combined sound map
- [ ] Phase 4: Documentation & plugins
  - [ ] Update `procedural-music` skill
  - [ ] Update `xm-writer` skill
  - [ ] Update asset pipeline guide
  - [ ] Update `tracker-demo` example
