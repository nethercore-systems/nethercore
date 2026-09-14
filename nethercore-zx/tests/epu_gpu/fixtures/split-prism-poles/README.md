# Frozen PRISM pole diagnostic inputs

Only split_prism_poles.rs consumes these fixtures. Diagnostic capture, NOT continuity acceptance or an adopted repair. Production shader assembly, packed instructions, assertions, tolerances and dispatch dimensions are unchanged.

The three WGSL bodies are byte-exact copies of tmp/epu-review inputs. The paths body exercises procedural pole directions; stored replays exact direction bits through helper/eval/dispatch and packed controls; axis diagnoses constant/runtime axis and basis evaluation using the same packed instructions.

split-prism-poles-input.bin is a minimal immutable TEST INPUT for the existing stored probe, not a GPU output dump. It contains 432 rows x 58 samples x 32 bytes = 801792 bytes. Each row-major sample is two WGSL vec4u records, little-endian <8I: three IEEE-754 f32 direction bit patterns, one zero padding word, then four original packed EPU instruction words. Stored reads every sample; axis reads the first sample of each of its first 72 rows. Keeping one exact shared input avoids a second derived axis fixture or changing shader indexing/dimensions.

Historical generator: tmp/epu-review/split-prism-poles-input.py. It reconstructs direction bits from channels of paths 17..19 in split-prism-poles-paths.csv (GPU readback), then packs original instructions. It is not a standalone deterministic CPU generator: reproducing directions would require that historical GPU CSV or a backend-sensitive GPU rerun. No cheap standalone equivalence proof is available; freeze the exact consumed input instead. Neither the CSV nor generator nor build outputs are copied here. Original ignored inputs remain untouched.

Output: PRISM_POLES_OUTPUT_DIR optionally selects a caller-owned unique directory (absolute recommended); missing parents are created. Unset writes no diagnostic artifacts. Existing receipts are refused; do not reuse an occupied directory for the same test. Choose a unique directory for each rerun. No runtime input override is provided.

## Exact original = fixture SHA-256

- split-prism-poles.wgsl: 2969 bytes; 8a85dbf292050f93e476b11e8aea58a667be2af1ff44812d42b1e8c189b30645
- split-prism-poles-stored.wgsl: 2562 bytes; 1d3431e815166f4ec7c53eb86cee758b283fa28636172ea78eb611caa945166a
- split-prism-poles-axis.wgsl: 1914 bytes; bf3816bfd806f0fdb4651302ebec2e5efd3420c5b6a3fab363bf1c2dd7ca2b67
- split-prism-poles-input.bin: 801792 bytes; 958ca25a29423265bd1a3b0be42c0a680a4639614381e8d3021f404834749893
