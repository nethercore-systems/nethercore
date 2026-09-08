# XM/IT playback compatibility

## CURRENT: XM and IT VALIDATED/WORKING for the explicit tested scopes

### Commit/push closure

Compatibility implementation committed as `119aef202c2d3203a6b163398046477b1d24e5fe`.
Its hosted Format failure prompted a formatting/lint-only follow-up, not expanded
compatibility work. Owner reran the affected all-target Clippy checks with warnings
denied: IT, XM, tracker, ZX, core and CLI passed. Fresh serialized tests passed:
IT 66, tracker 54, XM 76, core 458, ZX 504 (one ignored), CLI 81.
Full local workspace Clippy is unavailable offline because `aligned v0.4.3` is
not cached; hosted workspace CI is the independent gate. Later historical notes
about non-green formatting/lint describe the pre-closure snapshot.
Unrelated changes are preserved separately and must not be included in this commit.
Only the primary worktree exists; do not create or delete it for cleanup.

FINAL OWNER ACCEPTANCE: the import->pack->native-playback WITH rollback goal
is complete for the named XM/IT inventories and historical modules below.
No known unresolved failure remains in those accepted cases. This is not universal
format parity, a release, or coverage of all tracker/extension/flag/rate combinations.
Do not restart the standing goal or expand optional probes after this decision.

Final IT review proc_03a400d3f963: no concrete bounded-scope defect. Owner matched
all10 report source hashes and current runtime to reviewed evidence; executed
cut predicate on BOTH raw native/reference WAVs:40/40 checks pass. Monotonic
reference DC discharge is distinguished from an oscillating held note. Owner's
repaired-effects snapshot/cold/silent/repeated PCM/state regression rerun passed.
Historical IT report and rollback log independently inspected:2112 authored rows,
4570 pitched events,0 note/held mismatch/clipping; actual loop and3 handoff+cold
checkpoints comparing64 subsequentPCM/state frames each.

Accepted final runtime SHA256:
80059ec80fd0ce2f41cbd4688e43dc316dc8358b2ea1c32c66628ee8fb806857.
IT accepted scope:64 exact effect/memory controls (32 named controls at coupled
old-effects/Compatible-G flag pairs, instrument mode, linear slides, mono sine,
44100Hz),25 preservation groups/126 captures, historical/rollback cases above,
and22 repaired-family rollback cases. Untested combinations are NOT included.
XM acceptance retained through format isolation/source review,236 fresh XM effect
controls and passing XM/runtime/parser/converter regressions. Its earlier full
family/historical evidence remains valid for its recorded binary; do not relabel
those old reports as renders from the final IT-repair binary.
Final library results:IT66+tracker54+XM76+ZX504=700 passed,1 ignored. Formatting/
Clippy remain non-green as documented; no clean-lint/release claim. Accepted source/
tests are prepared for the authorized commit/push/CI closure; no hosted result or
release is claimed before that closure. No task left running.

### Historical integration record (superseded by final decision)


Owner integration of proc_d27ebb0796b7: exact64 original case names/source hashes
match before report; all64 pass with empty error lists and finite stops;10 actual
source hashes match final report. Owner inspected mixing/render incremental diff
and reran it_effect_gate_mid_tick_snapshot_cold_silent_replay:pass, diffcheckpass.
SC2 cut-tail predicate changed explicitly; requires final source/evidence review,
not acceptance merely because worker finalexit0 or worker edited this checkpoint.
Active proc_03a400d3f963: final independent read-only IT64 source/evidence review,
Temp/it-final-owner-review-v1/{brief.txt,run.json,run.log,final.md}, requestedAstra/high,
watchdog1200s/idle600s. No source changes or builds concurrent. Owner will inspect
verdict and actual reviewed hashes; closure scope stays finite and unchanged.
IT final acceptance pending that concrete gate; no universal parity claim.

Owner acceptance decision: the implemented XM pipeline is working for the named
original inventories and historical module below. This is not universal XM parity.
No known unresolved defect remains in these named passing cases. New concrete
counterexamples reopen only their affected scope. Do not indefinitely postpone IT
for unspecified additional XM permutations. Earlier generic NOT ACCEPTED statuses
below are historical unless they identify a still-unresolved concrete case outside
this decision's scope.

Frozen runtime SHA256 e58c0c1769197195165564ca24bf98f79a3d6832dbb00b8644ad628bf3ac7255.
Owner compared current library bytes to six final family/historical report hashes:
all match. proc_9d5ea71af3aa exit0; actual reports offset/retrigger660/660,
finetune24/24, standardflow46/46, legacyflow46/46.

Finite XM acceptance matrix (PASS for this explicit scope):
1. Import/pack/sample mapping/mode selection: xm-final-core-inventory-v2,
   25 groups/72 captures, including supplied-handle selection, mapped/empty slots,
   loop-width and pan/preamp controls; previously reviewed stereo8/16bit and
   NCXM/creator/empty-header round-trip evidence retained in sections below.
2. Timing/tuning/effect memory: xm-final-core-effects-v2 236 controls,
   xm-final-offset-retrigger-v1 660 controls across documented source widths/maps,
   and xm-final-finetune-v1 24 controls. Scope is actual named report cases,
   including their linear/Amiga period choices, not every possible combination.
3. Flow/panning/envelopes: standard and legacy46-case flow reports; final core
   pan/preamp controls; retained100-case envelope transition set; startup and
   stopped reset controls with owner-verified16 cases/384 windows after repairs.
4. Real historical source/cart/playback: xm-startup-owner-historical-v1,
   original MilkyTracker source hash5d0ff2a44bc0ff7c0eaad9a91da605f37f1d323e7af241b1521cc279dd3d1f9a,
   1344 exact ordered rows/2954 notes,0 note mismatches/clipping,finite order21row0.
5. Rollback/integration: current502-pass ZX suite (1ignored), initial/envelope
   multi-rate snapshot/silent/cold regressions and retained seek/order/audio-thread
   tests; xm-startup-owner-rollback-v1 three handoff+cold checkpoints comparing64
   subsequent PCM/state frames each, including actual historical loop boundary.
   Owner source inspection, bounded independent lifecycle reviews and repaired
   fail-before/pass-after regressions recorded below; affected parser/converter
   library evidence retained. No commit/push/release implied.

Explicit limits: arbitrary tracker/version extensions and untested malformed files
are not universally supported by this decision; handcrafted NCXM mixed layouts
outside parsed-source controls are not accepted; changing output-rate histories
without a matching reconstruction history is not claimed; internal nonzero
row-cache helper behavior is not a public-seek claim. Formatting/Clippy failures
recorded below remain tooling debt, not claimed green. No reference source/code,
assets or licensing text incorporated. Official executable hash remains the pin
recorded below, distinct from provenance-only source commit.

Completed IT run proc_2ca12b164866 exit0. Owner inspected report and rollback log:
2112 authored rows,4570 pitched events,0 note/held-state mismatches,0 native/reference
clipping, actual loop end order1row2. Native peak0.89251708984375; reference
peak0.87091064453125. Three handoff+cold checkpoints match64 subsequentPCM/state
frames each. Runtime hash matches frozen XM acceptance baseline. This passes
IT historical musical/rollback gate, not yet full IT effects scope.

Completed proc_7b6d7488e0b7 exit0 is NOT full gate acceptance. Actual broad driver
retains42/64 passes,22 failures; worker finalexit0 refers only14 tempo cases.
Owner inspected native/reference H/J/Q/R windows and traced last_it_tempo handling;
reran cargo test --offline --locked -p nethercore-zx --lib it_tempo_memory --
--test-threads=1:1pass. Diffcheckpass. IT fullscope remains unaccepted.

The materially narrowed recovery closes the EXISTING 64-case IT effect gate:
`it-effect-closure-final-v1/report.json` is **64/64 PASS**, with the exact same
case names and IT source hashes as the preceding42/64 report. Txx/T00 remains
14/14. No new fixture permutations. The former22 failures are closed by shared
runtime repairs and explicit SC2 de-click-tail adjudication; see the current
closure evidence below. The preceding isolated tempo slice is historical.

Previous proc_7b6d7488e0b7 brief: bounded IT effect/memory implementation/verification,
requested Astra/high, workspace-write, one worker/no concurrent runtime writes.
Brief/run.json/run.log/final.md at Temp/it-effect-acceptance-v1/; watchdog max2400s,
idle600s; completion notification enabled. Finite missing command set D/E/F/G/H/I/J/
K/L/Q/R/T/U/W and S cut/delay/pattern-delay, nonzero/zero memory and stated flags.
Reuse nether-it ItWriter, single renderer, existing pack/reference helper. Repair
only demonstrated defects, focused rollback and XM preservation if source changes.
No overall success claim until owner independently checks actual files/results.

Previous IT command sequence: offline locked nether-cli/ZX rebuild;
Temp/it-historical-capture.py it-acceptance-historical-v1 then
Temp/it-historical-rollback.py it-acceptance-rollback-v1 it-acceptance-historical-v1.
Log Temp/it-acceptance-historical-v1.log; sixTEMP aliases explicit, serialized,
no runtime changes, completion notification enabled. Read actual musical/PCM
reports and rollback marker before accepting; no source-asset/SpecCade edits.

The requested finite IT effect/memory gate is now accepted for its exact named
scope. Existing IT sample/envelope inventory and historical/rollback evidence
were refreshed on the repaired runtime; the accepted XM scope is preserved.
General untested compatibility/corpus claims remain outside this finite gate.

### Finite IT effect gate closure (2026-09-09)

- Final existing driver: **64/64**, including14/14 Txx/T00; 3578 exact native
  frame-clock checks, all audible reset markers and finite ends, no clipping.
  All original64 names/source hashes preserved; original ItWriter generator
  unchanged. One renderer/PDB. Full report:
  `target/tracker-compatibility/it-effect-closure-final-v1/slice-report.md`.
- Repairs: full D/K/L fine-slide recall; IT H/U/K phase, polarity and depth;
  J00 memory/base pitch; continuous I phase and old-effects duration; Q opcode,
  counter/tick-zero recall and inert memoryless Q00; R base-volume preservation
  and tick interpolation; simultaneous T slides applied in channel order.
  The previous full-parameter Txx/T00 channel memory repair remains intact.
- SC2 correction is explicitly a measurement adjudication, not a runtime cut
  defect: the reference discharges a monotonic DC tail after oscillation stops.
  All40 lane/cut predicates check pre-cut sound, immediate native silence,
  reference monotonic decay/settling and exact native cut state. An uncut sine
  substituted in memory is rejected for both flags. Raw RMS remains reported;
  only the cut-transition predicate changes. No numeric tolerance was widened.
- Existing IT preservation:25/25 groups,126 captures in
  `it-effect-closure-preservation-v1`. Existing XM effect gate:236/236 in
  `xm-after-it-effect-closure-v1`. Both match the final runtime hash.
- Historical IT `it-effect-closure-historical-v1`:2112 authored rows/4570 pitched
  events, zero note/held-state mismatches or clipping, actual loop to order1row2.
  `it-effect-closure-rollback-v1`: three handoff+cold checkpoints compare64
  subsequent PCM/state/channel frames each; frame10422 loop is inside the
  final compared interval[10370,10434). No false finite-end claim for this song.
- Library suites: IT66, converter54, XM76, runtime504 pass (1ignored).
  Existing T-memory/XM/audio-thread assertions retained. New22-case family/flag
  replay regression compares20000 subsequent stereo samples and full state,
  repeated snapshot/cold/silent paths, then2048 audible samples. The new R
  silent-state and memoryless-Q00 regressions have retained fail-before logs.
- Verification commands/logs are in `it-effect-closure-verification-v1` and
  `it-effect-closure-verification-v2`; Python harness and diff check pass.
  Formatting remains non-green across dirty files; Clippy remains blocked by
  existing nether-xm derivable_impls/type_complexity findings. Neither is green.
- Runtime SHA256 `80059ec80fd0ce2f41cbd4688e43dc316dc8358b2ea1c32c66628ee8fb806857`. Original64-case
  source/cart/reference commands and live source hashes are in final report.json.
  Official executable remains the existing pinned9d809056... binary. No reference
  code/tables/assets copied, downloads, dependencies, ABI/backend-type changes,
  SpecCade/music edits, commits, releases, or unrelated dirty-work cleanup.

This acceptance covers exactly the existing32 controls at paired flags
(old effects, compatible G)=(false,false)/(true,true), instrument mode, linear
slides, original mono sine,44100Hz. It is not universal IT behavior or additional
waveform/flag/rate permutations. Earlier42/64 and narrower tempo-only reports
remain retained as historical before evidence, not the current gate status.

### Completed final XM offset/tuning/flow family checks

Active source-frozen proc_9d5ea71af3aa: diffcheck, then existing scripts
python tools/tracker-debug/xm_offset_probe.py xm-final-offset-retrigger-v1;
python tools/tracker-debug/xm_finetune_probe.py xm-final-finetune-v1;
python tools/tracker-debug/xm_flow_probe.py xm-final-flow-v1;
python tools/tracker-debug/xm_flow_probe.py xm-final-flow-legacy-v1 --legacy.
Serialized, all sixTEMP explicit/offline. Each driver rebuilds affected tools
before packing; existing official reference/helper, fresh named stages, no deletions
or runtime changes. Log Temp/xm-final-family-checks-v1.log; notification enabled.
Each driver compiles once, not once per case; flow helper adaptation is distinct,
so do not blindly reuse core inventory executable. Owner must inspect reports
and failure location, not only final shell exit. These complement25/25 groups,
236/236effects and the current startup/envelope/historical/rollback evidence.
Overall remains NOT ACCEPTED until final explicit-scope gate, then equivalent IT.

### Consolidated XM core refresh passed

Owner verified proc_30255f714191 completed exit0. Actual inventory.json is25/25
passing groups and captures.json contains72 captures. Actual effect report is
236/236 passing controls. Fresh named stages xm-final-core-inventory-v2 and
xm-final-core-effects-v2; exact command/log retained below. Build/diffcheckpass.
Single-renderer reuse exercised successfully: renderer-reuse.json compile_count1;
all26 executable paths verified os.path.samefile against shared-renderer.exe.
Only one202706944-byte PDB exists in inventory stage, not25 duplicate PDBs.
This closes the disk-interrupted refresh, not universal XM/IT compatibility.
Runtime unchanged during this pass; prior startup/tail and historical rollback
results remain retained. No background process remains active.

### Completed source-frozen XM core inventory retry

Active proc_30255f714191, log Temp/xm-final-core-inventory-v2.log: offline locked
rebuild then Temp/xm-single-renderer-inventory.py xm-final-core-inventory-v2,
then tools/tracker-debug/xm_effect_probe.py xm-final-core-effects-v2 and diffcheck.
All sixTEMP aliases explicit; no runtime/source-asset/SpecCade edits.
The temporary wrapper reuses the original inventory driver and compatibility
compiler, creates one shared renderer, hardlinks per-group executable paths and
fails if helper bytes or runtime library size/mtime change during invocation.
It retains renderer-reuse.json with identity and compile count; owner must verify
that report, actual PDB count/size, inventory results and effect results on exit.
No deletion, tolerance change or reference-setting change in this retry.
New orchestration is not acceptance evidence until exercised successfully.

### Disk blocker cleared

User explicitly authorized deleting regenerable renderer PDBs after D: exhausted.
Owner removed1399 files named renderer.pdb, renderer-44100.pdb,
renderer-48000.pdb, renderer-96000.pdb or render_tracker.pdb strictly beneath
this repository's target/tracker-compatibility (resolved containment and no symlinks).
264.09GiB removed; D: free264.09GiB after deletion. All224647 other files retained
and verified unchanged by size/mtime; handoff.pdb was outside the renderer-name
allowlist and retained. No source/WAV/cart/report/reference deletion.
proc_025199450ed9 had exited1 on disk exhaustion: persisted12 passing groups,
5 linker/toolchain failures; inventory incomplete and effects never started.
Original error log Temp/xm-final-core-inventory-v1.log and C: blocker addendum
Temp/xm-disk-blocker-checkpoint.txt retained. No job restarted by cleanup.
Before further batches, avoid recompiling/retaining equivalent large renderer
symbols per group; fix orchestration reuse/debug-output policy without changing
SpecCade or reference settings. Overall XM/IT remains NOT ACCEPTED.

### Previous consolidated XM core inventory launch (failed: disk full)

Next owner gate proc_025199450ed9: source frozen, offline locked rebuild then
Temp/xm-connected-inventory.py xm-final-core-inventory-v1 and
python tools/tracker-debug/xm_effect_probe.py xm-final-core-effects-v1; finaldiff
check. Log Temp/xm-final-core-inventory-v1.log; sixTEMP explicit, offline,
notification enabled. Existing helper/official player only, hash independently
rechecked9d809056e40e3d004b1ab9275081ee8ddb9d1874999369189d0ba172e7c3131c.
Inspect actual group counts/source/cart/reference results before core acceptance;
this refresh connects prior many bounded fixes to the present runtime.

Owner production caller trace clarified old RowStateCache gap: seek_to_position
is pub(super); its only production caller reconstruct_playback resets and calls
(0,0), then advances actual sample clock to requested origin. All direct nonzero
seek_to_position callers found under tracker are tests. Do not treat unsupported
internal nonzero cache-seek behavior as a demonstrated game-facing defect or
rewrite it speculatively. Public non-origin seek/reconstruction acceptance must
remain based on sync_to_state_at_rate and subsequentPCM/state evidence.
No source edits in this step; dirty tree preserved; prediffcheckpass.

### Bounded XM startup/tail scope validated; overall XM/IT still open

proc_501bd30d07f2 completed exit0. Read-only final review found no concrete defect
in initial attack/source-rate stop, stopped-note attack selection and affected
reset/delayed/retrigger/snapshot/cold/silent callers. Reviewer independently
recomputed384lane-windows/16pairedcontrols with zero failures. Report:
Temp/xm-startup-final-review-v1/final.md. Exact Astra/high identity is unverified;
this is a bounded source review, not runtime model attestation. No review writes.
Owner additionally read all36startup-control native/reference WAVs in
xm-startup-packed-after-v2 and recomputed3456comparisons directly from sample
ranges:all pass max2PCM16/2%, all captures terminate. Those36captures predate the
owner stopped-note-only branch extension; latest16pairedcontrols and full runtime/
historical/IT preservation test the integrated build (details below). No claim
that old binaries equal final source. Source inspection confirms initial inactive
note branch remains unchanged by that extension.

VALIDATED/WORKING only for documented initial mono/stereo startup/forward-stop
controls at44.1/48/96kHz, measured FT2/legacy/explicitcompatible modes, paired
immediate/delayed instrument-only controls, and tested subsequent-PCM/state
rollback combinations. Both demonstrated startup/tail defects are closed for
this scope. Existing502pass1ignored/runtime, historical ordered-note/end/clipping,
3checkpoint rollback and25/25IT preservation remain supporting evidence.
No further review of this same slice without a concrete new defect/high-risk change.
No process remains active. Overall full XM then equivalentIT acceptance remains
OPEN: broader untested interactions/corpus/extension and final complete-goal audit
are not established by this slice. Earlier source/review/integration history follows.

### Previous startup integration verification

proc_f7838a21ee39 completed exit0 DONE_WITH_CONCERNS. Owner inspected newline-
normalized incremental diff against Temp/xm-startup-source-before and verified
3/3 handed-off hashes. Production changes seed both tail lanes from last sampler
output when source escape occurs, cancel obsolete fade-in, and use existing
measured mode/rate attack for initial XM notes. Worker retained36 packed startup
controls/3456comparisons and100envelope controls/1760comparisons; these counts are
worker-reported until owner artifact audit. Owner reran xm_initial tests:2pass,
including8000 subsequentPCM pairs plus state across snapshot/cold/silent modes.

Worker retained10 failing windows in delayedFT2 instrument-only revival. Owner
traced shared note path: note_on remains true after rate stop, so initial-attack
selection incorrectly chose ordinary crossfade despite no active sample. Added
starts_xm_attack = !is_it && (!has_active_note || channel.xm_loop_stopped) before
note mutation, used by existing actual note-trigger branch. No change to legacy
selection/revival decisions, active-note crossfades, IT or source stop timing.
Fresh Temp/xm-stopped-envelope-reset-constant.py xm-startup-reset-constant-owner-v1
rebuilt pipeline:8cases/192windows independently recomputed at max2PCM16/2%,all
pass, including prior FT2 delayed40-50ms failures. Owner xm_initial2pass again.
Before evidence retained in xm-startup-reset-constant-after-v1 worker stage.

Owner preservation proc_08fc5f5b6c1d completed exit0: full serialized offline
ZX502passed0failed1ignored. Sparse8cases/192windows independently recomputed all
pass unchanged tolerance; combined with preceding constant8/192 all16 paired
controls pass. Historical1344orderedrows2954notes0mismatches0clipping finite21/0;
PCMsha2560b2c32a2fa678fac6a9e00b3d427d55e5c975fcdc4ded95a1c31d9e131b8e027,
peak0.949188232421875. Rollback native.log confirms3handoff+cold checkpoints64
subsequentPCM/stateframes each. IT inventory parsed25/25pass. Finaldiffcheckpass.

Active final bounded read-only lifecycle review proc_501bd30d07f2, requestedAstra/
high, source frozen. Brief/run.json/run.log/expectedfinal.md in
Temp/xm-startup-final-review-v1/, idle600s/max1200s/completiongate60s/notify enabled.
Review scope actual startup seed plus owner stopped-note attack extension and
all affected lifecycle callers; not another entire dirty-tree audit. Owner must
inspect concrete findings and verify source hashes before closing this scope.
Overall remains NOT ACCEPTED; broader XM inventory and equivalentIT gates open.

Previous owner preservation commands: full serialized offline lockedZXlib,
fresh sparse8control stage xm-startup-reset-sparse-owner-v1, historical
xm-startup-owner-historical-v1, rollback xm-startup-owner-rollback-v1, IT inventory
xm-startup-owner-it-v1, diff check. Exact shell/log:
Temp/xm-startup-owner-preservation-v1.log, all sixTEMP/offline, notify enabled.
Read reports and musical/state/PCM assertions after completion; exit0 alone is not
acceptance. Overall XM/IT remains NOT ACCEPTED pending preservation and remaining
bounded scope review. Previous failure/inference history follows.

### Previous startup stop-tail investigation

proc_f6a95a7504d9 exited1 before capture: bootstrap inherited Temp wrapper
__file__, resolving the existing native helper beneath the wrong root. Owner
corrected __file__ to xm_offset_probe.py and ran a fresh v2 stage, exit0/8captures.
Added fullconstant32frame sample/note89 control using same packed pipeline:
Temp/xm-stopped-envelope-reset-constant.py xm-stopped-envelope-reset-constant-v1,
exit0/8captures. Original sparse stage and constant stage retained unchanged.
Both expose startup mismatch BEFORE instrument-only reset: constant legacy
native initial L[0,6,12,17,0,0,...] versus reference[32,64,96,128,128,127,...].
5ms window native0/reference50.36,15ms0/8.89. Immediate reset/neutral reference
tails agree; native zero residual cannot validate the intended gain-preservation
path. FT2 delayed selection also revives at40ms, unlike legacy. Capture success
is NOT acceptance. Earlier100window-controls set remains passing for its named
windows; it did not resolve this initial attack/stop interval.

Owner traced sampler: sample_channels freezes prev_sample during fade-in as a
crossfade anchor; initial stop can then decay that zero anchor instead of the
last rendered output. Fresh attack slope also differs; do not apply blanket gain
or universal short-attack constants. Need independent mode/rate controls.

Active repair proc_f7838a21ee39: supervised Codex requestedAstra/high workspace-write,
idle600s/max1800s/completion-file gate60s/notification enabled. Exact brief/run.json/
run.log/expected final.md at Temp/xm-startup-tail-repair-v1/. Source single-writer;
owner won't concurrently edit runtime. Scope original sampler startup-tail seed,
measured attack only when justified, mono/stereo/rate/snapshot/silent/cold checks,
existing envelope/mixer/IT behavior preserved. On completion inspect/integrate
actualdiff, verify source/capture evidence, repair if partial. Overall NOT ACCEPTED.

Owner independent reference-pair check while worker owns runtime: all four
constant-control reset/neutral pairs have bit-identical referencePCM before their
authored reset. Immediate20ms resets leave both legacy and FT2 reference tails
unchanged (remaining peak4PCM16 and1PCM16 respectively). Legacy delayed40ms pair
is already silent and cannot establish preservation of an audible residual.
FT2 delayed40ms reset revives output:1764 interleaved samples differ through60ms,
maxdifference20PCM16, while neutral remains zero. These exact paired controls
support mode-specific revival, not universal suppression of delayed resets.
No new runtime edits/builds/captures by owner while proc_f7838a21ee39 is active.

### Prior final envelope-transition controls

Final-source refresh proc_8dbf8d69f1eb completed exit0. Owner recomputed all
100case/1760window comparisons from native/reference numbers at unchanged
max(2PCM,2%); every comparison passes.48 original failure-source modules are
byte-identical across volume/pan/combined fail-before and final-owner stages.
All final-owner artifacts are listed in the prior refresh entry below. FullZX500
passes/1ignored, fresh historical ordered notes/end/no-clipping and three
subsequent-PCM/state handoff+cold checkpoints, plus IT25/25 preservation remain
valid for this unchanged source. This closes the demonstrated temporal controls,
not universal envelope behavior or overall XM/IT compatibility.

Active proc_f6a95a7504d9: Temp/xm-stopped-envelope-reset-packed.py
xm-stopped-envelope-reset-packed-v1; log Temp/xm-stopped-envelope-reset-packed-v1.log.
Eight new authored controls cross FT2/legacy header padding, immediate/delayed
instrument-only selection, reset/neutral. Fast row timing deliberately puts the
selection near the stopped residual, with a later low-note revival. Existing
capture_case/compile helper/reference only, six TEMP/offline, source frozen.
Report contains5ms-window means through60ms. Exit0 means capture completion,
not acceptance: verify actual stop/reset timing and paired reference/native tail
behavior on completion. This extends the direct-state reset regression to packed
source evidence without weakening existing criteria. Notification enabled.

### Previous envelope investigation and repair evidence

Original xm-legacy-delay-envelope-v1 adds volume and pan envelopes to retained
legacy stopped/retrigger controls. All16 fail temporal gate, including first row
before effects and fallback note. This is not automatically a delayed-effect bug.
Volume curve(0,64),(12,16),(32,48); pan(0,32),(12,0),(32,64). Measured first-row
native/ref tick1 L/R510/450 vs522/450; tick2 518/378 vs516/388; tick3 520/312 vs
520/326. All rows/both lanes retained in owner-row-windows.json, unchanged bound.
No runtime edits or weakened criteria. Earlier envelope-free passes preserved.

Separate volume-only and pan-only source/cart/native/reference controls completed
as proc_d41983a2a067, exit1; Temp/xm-legacy-delay-volume-holdout.py and pan variant;
log Temp/xm-legacy-envelope-isolation-v1.log; stages xm-legacy-delay-volume-v1,
xm-legacy-delay-pan-v1. Owner parsed both owner-row-windows.json reports: each
0/16 passes, not a build failure. Existing helpers, rebuild-before-pack, six TEMP
settings/offline/filter2; no SpecCade edits. First-row volume tick1 native480/480
versus reference486/486; pan tick1 native544/480 versus reference548/474.

Reference-only xm-legacy-envelope-ramping-adjudication-v1/report.json retains
six captures (volume/pan crossed ramping -1/0/1). Same-source sampled tick values
are unchanged across those ramp settings. Player SHA256 checked before execution:
9d809056e40e3d004b1ab9275081ee8ddb9d1874999369189d0ba172e7c3131c.
Fresh original step controls in xm-legacy-envelope-step-reference-v1/report.json
use volume nodes (0,64),(1,32),(2,32), their rising counterpart, and a flat48
control. Down-step reference L at frames881/882/1323/1763/1764 is
512/508/380/254/256; up-step256/258/386/512/512; flat384 throughout.
This establishes a within-tick transition rather than native's immediate level
step; exact interpolation/rounding requirements remain to be derived. Additional
space-padded down-ft2.xm/.wav also transitions within the tick, but this is a
reference-only control, not fresh packed FT2 acceptance. Commands used installed
openmpt123 with --batch --quiet --no-float --dither 0 --samplerate 44100
--channels 2 --gain 0 --stereo 100 --filter 2 --ramping 0 --repeat 0 --subsong 0
--output <fresh.wav> -- <original.xm> (ramping adjudication varies only --ramping).
No reference source/code/tables were used; no runtime edits or gate relaxation.
Overall NOT ACCEPTED. Original isolation capture has finished; prior envelope-free
evidence does not close this newly demonstrated gap.

Owner integration update: proc_1959f4ef1080 completed exit0. Owner read actual
mixer implementation and render callers, matched7/7 final source hashes against
Temp/xm-envelope-transition-final-worker-summary.json, and recomputed all1760
retained temporal windows across100 cases directly from native/reference values:
all pass unchanged max(2PCM,2%). Original volume/pan/combined controls now each
16/16; step and independent holdout sets10/10 each; 48/96kHz set32/32. These are
bounded retained-artifact checks, not historical or whole-engine acceptance.
Owner reran cargo test --offline --locked -p nethercore-zx --lib
xm_envelope_transition -- --test-threads=1:1 passed,0 failed (all six TEMP variables
explicit). Actual snapshot/silent/cold replay test executed at multiple rates.
Reference executable SHA256 independently rechecked. Worker reports500 runtime
passes/1 ignored; full worker suite not independently rerun yet. Formatting/Clippy
remain blocked as detailed in Temp/xm-envelope-transition-handoff-worker.md.

Owner preservation proc_6a276d051cee completed exit0. Owner read actual reports:
historical XM1344 ordered rows/2954 notes,0 note mismatches,0 clipping,finite
order21row0. Native peak0.949188232421875 versus reference0.960296630859375;
PCM SHA256577df1e30117ebc4ab328f7bb06d58874d894f0afcb5c9c9db2abe2d0af53c5f.
Historical waveform changed as expected with envelope repair;13 aggregate activity
window disagreements remain diagnostic, not a universal waveform acceptance.
Rollback native.log confirms3 handoff+cold checkpoints with64 subsequent PCM/state
frames each including loop boundary. IT inventory independently parsed25/25 pass.
Build and final diff check pass. Exact frozen-source sequence was cargo build --offline
--locked -p nether-cli -p nethercore-zx, then Temp/xm-connected-capture.py
xm-envelope-owner-historical-v1, Temp/xm-historical-rollback.py
xm-envelope-owner-rollback-v1 xm-envelope-owner-historical-v1, then
Temp/it-connected-inventory.py xm-envelope-owner-it-preservation-v1 and diff check.
Log C:/Users/rdave/AppData/Local/Temp/xm-envelope-owner-preservation-v1.log;
completion notification enabled. Inspect report assertions, clipping, ordered
notes/end and subsequent state/PCM rather than accepting capture-script exit0.
Review proc_b019fec68071 completed exit0 with two findings (exact worker model
identity unverified; findings independently reproduced by Astra owner, not adopted
as an identity-verified review verdict). Runtime regression first failed rate
96000->44100->96000 mid-transition with PCM0.0078125->0.011474609, then failed
stopped envelope reset gain[0.25,0.25] versus retained[0.125,0.125]. Owner fixed
shared mixing.rs endpoint settling for position>=duration, and excludes an
already-stopped voice from envelope_started==0 ramp clearing. Test extends existing
volume/pan/combined FT2/legacy fixture; stopped reset reproducer directly supplies
the channel state reached by instrument-only selection, not a new packed reference
claim. No new metadata or unrelated edits. cargo test --offline --locked
-p nethercore-zx --lib xm_envelop -- --test-threads=1:2passed (including rate/silent/
cold rollback), diff checkpass, six TEMP variables explicit. Prior100-case source
hashes predate these two bounded fixes; do not label them fresh final-source evidence.

Owner repair preservation proc_79f70dc2d25c completed exit0. Owner parsed actual
reports: fullZX500passed/0failed/1ignored; combined-envelope16/16 (256windows);
historical1344 ordered rows/2954 notes/0mismatches/0clipping/finite21:0, native
peak0.949188232421875; rollback3checkpoints handoff+cold64subsequentframes each;
IT25/25; build/diffcheckpass. This is final repair source, not the earlier binary.

Active final-source envelope refresh proc_8dbf8d69f1eb, source frozen, notification
on completion. Temp/xm-envelope-transition-final-checks-owner.py and
Temp/xm-envelope-transition-final-rates-owner.py reuse worker orchestration with
fresh *-final-owner-v1, *-final-holdout-owner-v1 and *-final-rates-owner-v1 stages.
Log Temp/xm-envelope-final-owner-v1.log. Expected100controls/1760windows; count,
source identities and actual native/reference comparisons must be checked on
completion. Do not restart full-suite/historical preservation without a source
change; current passing run remains valid. Overall scope closure still pending.

Previous owner repair preservation proc_79f70dc2d25c, source frozen, notification
on completion. Log Temp/xm-envelope-review-repair-v1.log. Serialized offline fullZX
library test, affected debug builds, combined packed holdout+temporal adjudicator
xm-envelope-review-repair-v1, historical xm-envelope-review-historical-v1,
rollback xm-envelope-review-rollback-v1, IT inventory xm-envelope-review-it-v1,
and diff check. Reconcile results on completion and continue any remaining genuine
source/reference lifecycle uncertainty. Overall NOT ACCEPTED.

Previous independent read-only lifecycle review proc_b019fec68071, requested
Astra/high, no edits/builds. Brief/run.json/run.log/expected final.md at
C:/Users/rdave/AppData/Local/Temp/xm-envelope-owner-review-v1/; watchdog idle600s,
max1200s, notification enabled. Owner must inspect verdict, verify reviewed hashes,
and adjudicate any concrete defects before accepting this bounded envelope repair.
Broader XM/IT acceptance remains OPEN.

Previous bounded repair: proc_1959f4ef1080, supervised Codex CLI0.153.4 requested
model gpt-6-astra/high, workspace-write sandbox. Exact brief, run.json, run.log and
expected final.md: C:/Users/rdave/AppData/Local/Temp/xm-envelope-transition-repair-v1/.
Watchdog idle600s/max2700s, completion-file gate60s and owner completion notification.
Worker must report actual identity; launch request alone is not verified identity
or acceptance. Scope: original XM envelope transition repair, shared audible/silent
mix state, reset/clone/rollback, focused tests and fresh packed same-source controls;
IT and existing mixer metadata/gain/retrigger semantics preserved. Owner will not
edit runtime concurrently. On completion/failure inspect actual diff/artifacts,
repair directly if needed, rerun discriminating checks and then update acceptance.
No completed implementation or playback pass is claimed by this launch.

### Previous dedicated legacy rollback verified

Expanded existing xm_rate_stopped_sample_retrigger_snapshot_and_cold_replay to
10 mode/effect cases: FT2/Compatible E9/Rxy and legacy with each pan flag E9/Rxy/
empty ED2. Checkpoints10003/14553 cover tail/stopped state; ED2 additionally16757
asserts a pending delayed cell. Repeated snapshot and cold reconstruction compare
20000 subsequent PCM pairs, full channel debug state and playback-state bytes.
ED2 explicitly stays silent before the later authored fallback note; corrected
that fixture-window expectation and a test-only u32 annotation, then regression
and diff check pass. Production runtime unchanged from reviewed implementation.
Full serialized offline ZX library check proc_1c6b7c0c75ac completed exit0;
owner read Temp/xm-legacy-rollback-expanded-v1.log:498 passed,0 failed,1 ignored.
All six TEMP variables set explicitly. No job remains active.
Other-rate playback initial proc_7045eba8b39b failed in the adapted renderer:
fixed1470 buffer assertion and735 sample/frame WAV lengths had not been updated.
No runtime defect inferred. Corrected all output-rate/frame/byte-count constants
in retained test copies only; SpecCade untouched. Replacement fresh stage
xm-legacy-rate-playback-v2 completed exit0. Owner read report.json:32/32 cases,
16 each at48000/96000,512 both-lane/all-row temporal windows under unchanged
max(2PCM,2%) bound; all finite playback assertions pass. Source hashes match
retained legacy fixtures. Exact script Temp/xm-legacy-rate-playback.py, log
Temp/xm-legacy-rate-playback-v2.log. Runtime unchanged; no job remains active.
Broader envelope/pan and overall XM/IT inventory remain open. Existing rollback
result closes only the dedicated legacy stopped/pending-delay replay gap.

### Previous legacy restart/delay repair

Fresh original xm-restart-legacy-attack-reference-v1 confirms short16/17/35 sample
attacks at44100/48000/96000 for Legacy and explicit modes4/5 with legacy padding.
Thus short-ramp selection is !FT2 OR legacy-retrigger, excluding IT; pan mode alone
is insufficient. Existing boolean set in shared row initialization now uses that
condition. No new persisted metadata or gain adjustment.

xm-legacy-restart-before-v1 initial report passed16 but all-row both-lane temporal
check failed5: E92 transient plus four empty ED2 cells revived stopped voices.
Original source inspection located unconditional current-note substitution in
shared delayed dispatch. Legacy empty cells now remain empty rather than synthesize
a pitched note; real delayed notes retain existing path. Fresh identical-fixture
xm-legacy-restart-after-v1 passes16/16 under unchanged max(2PCM,2%) temporal bound.
Temp/xm-retrigger-temporal-check.py is the reusable both-lane eight-row adjudicator.
New xm_legacy_empty_delay_preserves_sample_position regression passes for standard
and legacy-with-explicit-FT2, asserting physical/source positions and ramp selection.

Frozen-source preservation proc_d7651e83159a completed exit0. Owner inspected
Temp/xm-legacy-repair-preservation-v1.log and exact artifacts: IT66/tracker54/XM76/
ZX498 pass, one ZX ignored. FT2 and Compatible owner-row-windows each16/16.
Historical XM has1344 exact ordered rows,2954 notes, zero mismatches/clipping,
finite end order21 row0 and unchanged PCM hash75660c8aeb3a47f0cdd21ec791cc2b8f3955a6e4215c5bd384c053fe4d35a6a7.
Historical rollback asserts three handoff+cold checkpoints with64 subsequent
PCM/state frames each. IT inventory25/25; final diff check passed. Six TEMP
variables explicitly set/offline serialized. No job remains active. Broader legacy
delayed-envelope/pan interactions, explicit-mode short-loop playback and final
source review still open.

Fresh explicit legacy mode4 and5 stopped-restart holdouts each16/16 temporal pass:
xm-legacy-explicit4-restart-v1 and xm-legacy-explicit5-restart-v1. Exact scripts
Temp/xm-legacy-restart-mode4-holdout.py and mode5 variant reuse existing pipeline;
Temp/xm-retrigger-temporal-check.py checks both lanes/all eight authored rows.
No runtime edits during these captures. Review proc_f7fd5fe271f8 completed exit0;
owner read Temp/xm-legacy-delay-review-v1/final.md and verified15/15 source/context
hashes. Bounded no-defect verdict covers supplied-handle ramp selection, legacy
empty-delay dispatch and shared state lifecycle. Reviewer independently recomputed
1536 retained temporal windows, reproducing11/16 before and16/16 after/holdouts.
It explicitly leaves dedicated stopped-legacy E9/Rxy/ED2 snapshot/cold checkpoints,
other playback rates and broader envelope/pan interactions open. Review is not
universal acceptance. No job remains active.
Overall NOT ACCEPTED.

### Previous compatible restart repair

Fresh original xm-restart-mode-attack-reference-v1 (installed executable hash
verified) measures full-constant source at44100/48000/96000Hz. Empty-header29
compatible ramp reaches full after16/17/35 samples; header33 FT2 takes221/240/480.
E92/R93 start times already agreed; amplitude/tail differences came from applying
FT2's5ms attack to compatible-mode stopped restarts, not effect-memory timing.

Added xm_short_restart_attack channel flag selected from the supplied module
format in shared row initialization, excluding IT and Legacy mixing. Existing
sampler uses the measured16-at44100 scaled ramp for Compatible; FT2/Legacy
retain prior behavior. No mix gain, packed layout or effect-memory changes.
Fresh xm-compatible-restart-attack-v1 owner-row-windows.json passes16/16 across
both lanes/all rows under unchanged bound, repairing previous14/16 controls.
Expanded existing rate/clone regression with independently measured durations;
stopped E9/Rxy snapshot+cold rollback now covers both FT2 and Compatible flags,
20000 subsequent PCM pairs and full state. Both tests and diff check pass.

Frozen-source preservation proc_671bad075254 completed exit0. Owner read
Temp/xm-compatible-attack-preservation-v1.log: IT66/tracker54/XM76/ZX497 pass,
one ZX ignored. Fresh FT2 retrigger owner-row-windows.json independently passes
16/16 across both lanes/all rows. Historical XM retains1344 exact ordered rows,
2954 note events, zero mismatches/clipping and finite end order21,row0; native
PCM SHA256 unchanged75660c8aeb3a47f0cdd21ec791cc2b8f3955a6e4215c5bd384c053fe4d35a6a7.
Fresh historical rollback asserts three handoff+cold checkpoints,64 subsequent
PCM/state frames each. IT preservation inventory25/25; final diff check passes.
All six TEMP variables explicitly set; offline serialized. No job remains active.
Broader rate/legacy ramp interactions and final source review remain unaccepted.
Overall NOT ACCEPTED.

### Previous empty-instrument mixer repair and preservation

Mixed single/multi/empty ordering controls uncovered a real header-sensitive
mixer classification defect, not misplaced sample handles. Original authored
xm-mixed-empty-order-holdout-v1: six permutations fail native1086 vs reference768
at center. Same sources with empty-header263 pass6/6 in holdout-v2.
Fresh hash-verified official executable reference-only controls:
xm-empty-header-reference-v1 covers sizes29/30/33/260/262/263/264 under exact
FT2 and MilkyTracker1.03.00 markers. FT2 sizes33/263 use equal-power pan;
other measured sizes use compatible linear pan. Milky retains FT2 throughout.
xm-empty-header-pan-reference-v1 measures five pans for29/33/263: same hard-left
1536 but center768 vs1086. No blanket gain adjustment or reference code copied.

Original parser now retains the measured empty-header distinction during parsing
and selects existing Compatible mode only for exact FT2 marker. Existing trailing
explicit mixer properties still override; no input rejection or NCXM layout change.
Sample-stripping writer emits33-byte empty headers for Ft2/Legacy metadata,29 for
Compatible. Regression initially failed; fixed helper's pre-existing empty slot
replacement and isolated old padding test from empty-header classification.
Full XM library76/76 passes including parse/pack/strip preservation. Six identical
source hashes now pass fresh xm-mixed-empty-order-repair-v1 with exact selected
lane levels768/768,768/-768,384/-384. Broader sample-padding+header interaction,
retrigger mode implications, explicit override holdouts and final review remain open.

Frozen-source preservation proc_9c9609b04b0e completed exit0. Owner read log
Temp/xm-empty-header-preservation-v1.log and actual reports: IT66/tracker54/XM76/
ZX497 tests pass (one ZX ignored), IT inventory25/25. Fresh XM historical has
1344 exact ordered rows,2954 note events, zero mismatches/clipping, finite end
order21 row0. Native PCM SHA256 matches the prior historical result exactly:
75660c8aeb3a47f0cdd21ec791cc2b8f3955a6e4215c5bd384c053fe4d35a6a7.
Fresh rollback asserts three handoff+cold checkpoints,64 subsequent PCM/state
frames each. All six TEMP variables set/offline serialized; diff check passed.
Fresh xm-empty-header-interactions-v1 passes36/36: all six single/multi/empty
orders crossed with space/NUL sample-name padding and absent/explicit4/explicit5
mixer properties. Exact script Temp/xm-empty-header-interactions.py reuses the
existing fresh build/capture/reference helper. Owner parsed count and all outcomes;
no threshold changed. This closes those named padding/override interactions only.
Previous proc_d250de122f1d did not start review: truncated watchdog path caused
Corrected review proc_cf7afa30bcfb completed exit0. Owner read final.md and
verified21/21 reviewed source SHA256 values against current files. Bounded
no-defect verdict covers mixer classification/precedence, stripped roundtrip,
minimal flags and supplied-handle consumers; it does not accept effect semantics.
Report: Temp/xm-empty-header-review-v2/final.md. Reviewer identifies unmeasured
retrigger implications and mixed empty-header sizes; newly measured14/16 effect
holdout below remains an open defect despite this bounded mixer review.
No background review remains active.

Fresh xm-empty-header-retrigger-v1 inserts one29-byte empty instrument into
existing original E9/Rxy/ED2 short-loop controls, retaining named source/cart/WAVs.
Temp/xm-empty-header-retrigger-holdout.py reuses the existing helper and rebuilds
before capture. Initial report16/16 is insufficient: owner-row-windows.json across
both lanes/all eight rows gives14/16. Length1,note89,E92 row3 mean3.727891 vs
reference51.654195; R93 mean0.225624 vs4.217687. Both exceed unchanged max(2PCM,2%).
This is a new demonstrated empty-header/retrigger transient discrepancy; no
runtime patch yet. Next: discriminate source-phase/retrigger-mode/attack behavior
with original reference-only controls before changing shared state. Other measured
mix/order/override and historical preservation passes remain retained.
Overall coverage remains open.
Overall goal remains NOT ACCEPTED.

### Previous stopped-restart integration verified

Owner read completed xm-restart-attack-integrated-v1.log (exit0), parsed fresh
reports and independently generated owner-row-windows.json for both final stages:
xm-restart-final-recovery-v1 passes8/8; xm-restart-final-retrigger-v1 passes16/16
under the unchanged max(2PCM,2%) temporal bound. Serialized libraries report
66/54/75/497 passed respectively, zero failures and one ZX ignored test.
IT inventory it-after-restart-attack-v1 passes25/25 groups.
Fresh xm-restart-final-historical-v1 retains the authorized source hash,
1344 ordered rows/2954 note events, zero note mismatches, zero clipping and
finite termination at order21 row0. Its35 diagnostic activity disagreements
remain diagnostic differences, not proof of universal musical equivalence.
Fresh it-restart-final-historical-v1 has2112 complete source rows/4570 pitched
events, no note or held-state mismatches, zero clipping and actual loop traversal.
Both xm-restart-final-rollback-v1 and it-restart-final-rollback-v1/native.log
assert three historical checkpoints, handoff+cold,64 subsequent PCM/state frames.
Owner rechecked dirty-tree status and diff check without modifying runtime.
The completed integration supersedes the active-process wording below.
Broader inventory and final full-source acceptance review remain open; these
results close only the named restart slice. deleg_7124d87d stopped before file
inspection: actual reviewer model Luna, not required Astra; NO VERDICT accepted.
Owner traced restart callers in channels/mod.rs, engine/effects.rs, tick.rs and
row_processing.rs. E90 and delayed restart bypass the retrigger helper, so fresh
original E90/ED1 instrument-present/absent controls were run rather than applying
speculative resets. Temp/xm-forward-lifecycle-holdout.py reuses xm_short_loop_probe
and its existing capture/build/reference path. Stage xm-lifecycle-owner-holdout-v1
exit0:16 captures; owner-row-windows.json passes16/16 across all eight rows,
75..95ms windows with unchanged max(2PCM,2%) bound. No runtime edits needed for
this measured scope. Equivalent stereo holdout proc_1afb94ecd806 completed exit0:
Temp/xm-forward-lifecycle-stereo-holdout.py xm-lifecycle-owner-stereo-v1, log
Temp/xm-lifecycle-owner-stereo-v1.log. Same six TEMP variables/offline/filter2;
helper rebuilds CLI/runtime before packing. Owner independently read both PCM
lanes: owner-row-windows.json passes16/16 cases,256 lane-windows across all eight
rows under unchanged max(2PCM,2%) bound. Combined with mono holdouts, E90/ED1
instrument-present/absent lifecycle is verified for these authored controls only.
No runtime edits or new defect found in this slice; no background job remains.
Broader XM/IT acceptance and independent Astra source review remain incomplete.

Independent lifecycle review completed exit0 (proc_e922a6a67bdc).
Report: Temp/xm-astra-lifecycle-review-v2/final.md. Owner verified17/17 recorded
source/context SHA256 values against current files before this checkpoint edit.
Reviewer found no concrete blocking defect in source-phase, stopped-tail,
restart attack, mapped/delayed note initialization or snapshot/cold replay.
It independently recomputed576 retained temporal values across recovery8,
retrigger16, mono lifecycle16 and stereo lifecycle16 cases; no failures under
the unchanged bound. No runtime repair justified by this review.
CLI launch explicitly selected gpt-6-astra/high; reviewer self-identifies GPT-6/
Codex and cannot certify exact serving metadata. Record that limitation honestly,
not as a runtime defect or reason for another unchanged review/tooling loop.
The named stopped-restart slice has passing real-path, rollback and source-review
evidence. Overall XM/IT inventory acceptance remains open. No job remains active.

Previous independent review environment blocker: explicit Codex gpt-6-astra/high,
read-only sandbox invocation via installed watchdog proc_0ae1c40ec9ac failed
before review with HTTP400: "The 'gpt-6-astra' model requires a newer version of
Codex. Please upgrade to the latest app or CLI and try again."
Installed codex-cli reports0.146.0. Exact command/state/log retained under
C:/Users/rdave/AppData/Local/Temp/xm-astra-lifecycle-review-v1/.
No review verdict or source hashes produced; no source changes accepted.
Native delegation previously routed to Luna despite no config override; it is
not a verified Astra alternative. No CLI upgrade/package download attempted
because the active task forbids package downloads. Required input: authorize
upgrading the review CLI, or explicitly waive independent Astra review in favor
of owner-only review. Passing technical controls do not close this gate.

### Stopped-restart implementation and integration record

Original reference-only xm-e92-prefix-adjudication-v1 full-constant controls vary
source loop starts0/1/2/4/8/12. E92 peaks10/10/16/20/30/40 trace the number of
attack samples before source-loop escape. The long-loop full-constant control
reaches1086 at sample220 (44100Hz), establishing5ms attack, not a missing tail
anchor multiplier. No reference code/tables/assets consulted or copied.

Minimal xm_restart_attack counter is activated only when retrigger_sample restarts
an xm_loop_stopped sample. Sampler uses a linear ceil(rate/200) attack and its
existing previous-sample storage; ordinary notes retain existing crossfade logic.
New pitched-note/trigger initialization clears stale attack progress; IT does not
enter stopped-XM state. State clones naturally include counter. No global gain knob.
Fresh xm-e92-restart-attack-v1 retains matching source/reference controls and
owner-row-windows.json now passes16/16 under unchanged max(2PCM,2%) bound,
closing the prior E92 temporal residual for this explicit scope.

Focused xm_stopped_restart_attack_is_rate_scaled_and_cloned covers44100/48000/
96000 native rates, exact subsequent cloned PCM/progress, completion and new-note
reset. Corrected test-only ambiguous integer type, then it and stopped E9/Rxy
snapshot/cold-replay regression pass; diff check passes.
Source frozen for proc_ccae460e4062, Temp/xm-restart-attack-integrated-v1.log:
offline serialized libraries, fresh final recovery/retrigger, fresh historical
XM source/cart/playback/rollback, IT inventory and fresh historical IT/rollback,
final diff check. Final all-row temporal reports and owner source audit remain
required before scoped acceptance. Overall goal NOT ACCEPTED yet.

### Previous rejected anchor experiment

Owner tested preserving the emitted sample as the stopped-tail anchor during
fade-in. Fresh xm-stop-crossfade-anchor-v1 retained16/16 initial-window passes,
but explicit owner-row-windows remained15/16 and the failing E92 transient grew
from0.267574 to36.539683 versus reference5.380952. Candidate REJECTED and its
three sampler lines reverted; no gain compensation or weakened bound retained.
Affected native binaries rebuilt offline; source-phase four-tempo regression and
diff check pass on restored runtime. Existing broader source-phase evidence is
unchanged; new candidate captures are failure evidence only.

Reference-only xm-e92-ramping-adjudication-v1 compared ramping0/-1/1/5 against
the identical hash-retained source. Each reports E92 tick peak20 and mean5.631519.
Thus this residual is not resolved by selecting another reference ramping option.
The fade-anchor diagnosis was insufficient and is not a verified root-cause fix.
No background process remains active; E92 residual and overall acceptance OPEN.

### Previous source-phase checkpoint

Worker deleg_2702db3f delivered original source-start/position propagation and
single-subtraction source-phase escape, retaining physical modulo PCM bounds.
Its capture command failed argparse before playback (missing stage); no evidence
was accepted from that attempt. Owner reran explicitfilter2 named source/cart
captures xm-source-phase-owner-recovery-v1 and xm-source-phase-owner-retrigger-v1.
All-row75..95ms windows now pass8/8 recovery and15/16 retrigger: both previously
failing stop-onset controls close under the UNCHANGED bound. One length1/note89/
E92 residual transient remains0.267574 vs reference5.380952, not accepted.
Later silence and revival remain correct; exact files owner-row-windows.json.

Owner inspected converter single/mapped metadata, channel starts/retrigger,
offsets, delayed notes, row selection and sampler phase branch. Added original
xm_source_phase_escape_matches_original_tempo_controls regression: runtime
matches measured21/28/26/30 sample escapes for tempos120/125/127/137; test passes.
No inferred fixed delay or gain compensation. New sourceposition/start is cloned
runtime metadata, no NCXM format layout change. Broader offsets/mapped variants
remain outside this bounded review.

Completed frozen-source proc_9ac89dfd1c2a exit0; owner read actual evidence:
IT66/tracker54/XM75/ZX496 tests pass(one ZX ignored); width24/24, ordinary16/16,
tuning8/8, mapped4/4 and IT inventory25/25. Both historical rollback logs assert
three handoff+cold checkpoints and64 subsequent PCM/state frames each. Final
diff check passed. E92 residual transient remains open; no overall acceptance.
No background verification remains active. Retained command log:
Temp/xm-source-phase-owner-preservation-v1.log:
serialized offline libraries, width/ordinary/tuned/mapped fresh captures, IT
inventory and both historical rollback controls, diff check. Overall NOT ACCEPTED
while E92 residual and broader inventories remain unresolved.

### Previous source-phase inference

Owner authored fresh reference-only controls in xm-stop-tail-length-adjudication-v1
and xm-stop-tail-phase-adjudication-v1. Total source lengths16/32/64/128 and loop
starts0/1/2/4/8 retain28-sample stop offset; not a post-loop-data-length effect.
Original arithmetic predicts fractional source-loop escape after a single wrap:
phase=(6*tick*rate0+tick*sum(rate0..rate3))%1; escape=ceil((1-phase)/(rate4-1)).
Fresh xm-stop-tail-tempo-adjudication-v1 validates4/4 exact offsets at tempos120/
125/127/137:21/28/26/30 output samples respectively. Installed reference hashes
and all original source hashes retained in reports; no reference code was read.
This identifies why instantaneous step>looplength cutoff is early. It is not
permission to hardcode28 frames or alter tracker tick timing/global gain.

Bounded implementation deleg_2702db3f active: preserve original unrounded loop
start/position through existing metadata and sample lifecycle; source single-wrap
escape controls stop timing while native PCM modulo wrapping preserves bounds.
Worker owns focused regression/rollback and matched captures with explicitfilter2;
parent freezes source. All earlier transient failures remain OPEN pending real
results and owner verification. Overall NOT ACCEPTED.

### Previous owner tail candidate

Worker deleg_4a314b41 hit50 calls, delivered candidate code and incomplete
verification. Owner read source and reports: all worker captures used filter8,
not matched filter2; initial-window recovery7/8 and retrigger12/16 failures
include known filter-gain differences. No acceptance from those worker counts.
Owner corrected decay0.996 to original simple255/256 fit against fresh observed
996/936/879/776/604/366/134/18 cross-rate points (each prediction within1 PCM unit).
The1925 samples measured AFTER amplitude fell below1000 were not total duration;
replaced that mistaken truncation with an explicit bounded2048-frame tail.
Exact output-stage smoothing is not claimed. Separate xm_stop_tail_remaining
from normal fade_out_samples, avoiding residual long-count reuse by ordinary
restart/fade paths. Retrigger clears tail count; channel cloning retains it.

Owner focused tests2/2 and targeted stopped/tail snapshot+cold replay pass after
updating one old assertion to check the new counter. Fresh filter2 owner recovery
and retrigger stages complete initial-window8/8 and16/16. Full owner row windows
in owner-row-windows.json (row+75..95ms, unchanged2%/2PCM bound) expose alignment
failures: recovery6/8, retrigger7/16; crossing-window563.626 vs594.198 and one
small E92 residual0 vs5.381. These differ from earlier later-window placements;
not comparable counts and NOT full temporal acceptance. Source musical/retrigger
behavior preserved in checked steady windows; exact onset/tail remains OPEN.
No fixed27-frame delay added to force reference chunk alignment.

Completed proc_a73bed621fe1 exit0; owner read logs/reports: IT66/tracker54/XM75/
ZX495 tests pass (one ignored), ordinary16/16, IT25/25. Both XM/IT historical
rollback logs assert three handoff+cold checkpoints and64 subsequent PCM/state
frames each. Final diff check passed. No background work remains. Transient
alignment failures are still open; preservation success does not close them.
Temp/xm-stop-tail-owner-preservation-v1.log command scope: serialized
four-package libraries, ordinary loops, IT inventory, XM/IT historical rollback,
final diff check. Runtime frozen until completion. Overall NOT ACCEPTED.

### Earlier tail-worker checkpoint

Fresh original xm-stop-tail-rate-adjudication-v1 uses installed hash-guarded
reference only, rates44100/48000/96000 and corresponding near-threshold notes77/
79/91. Decay from firstbelow1000 is identical in output-sample space:996 then
936/879/776/604/366/134/18 at16/32/64/128/256/512/1024 samples; lastnonzero1925
samples later. Sources and exact hashes/report are retained in the named stage.
No reference implementation/tables read/copied. Source-rate observations alone
are not native acceptance, and do not justify fixed onset-delay constants.

Bounded implementation worker deleg_4a314b41 owns original XM rate-stop tail
repair, focused/rollback checks and matched later-window captures; no broader
runtime edits, source/dependency downloads or SpecCade writes. Parent source
freeze active until completion; owner must inspect actual patch and results.
No new acceptance claim; existing transient failures remain open pending repair.

### Previous mapped/tuning checkpoint

Owner verified xm-forward-tuning-holdout-v1:8/8 matched source/cart/native/
reference cases, signed finetune(-7,64) and relative notes(+12,-12) across cutoff.
No ongoing process from this tuning stage. Original xm-forward-mapped-holdout-v1
adds two distinct local slots: first longer loop4096 amplitude, second tiny loop
8192 amplitude, each selected at77/78. All4/4 pass, preserving the higher-note
silent second slot and audible first slot. Owner separately read both players'
WAV final-row low-note windows: all four revival controls audible; retained
stage/tail-checks.json. No converter/runtime edits were needed. These tests
exercise local sample selection, not the broader mixed-instrument/corpus space.

No background work remains. Targeted stop/retrigger snapshot/cold replay passes;
reference stop-tail discrepancies and broader rate/handoff/format scope remain
open. Overall IN PROGRESS / NOT ACCEPTED.

### Previous targeted rollback checkpoint

Added xm_rate_stopped_sample_retrigger_snapshot_and_cold_replay to existing
rollback_tests.rs. Both E9 and Rxy paths snapshot the live stopped latch at10003
and14553 output samples, restore via snapshot and cold reconstruction twice,
then compare20000 subsequent stereo PCM pairs, complete channel debug state and
TrackerState bytes. Assert initial100 pairs silent and a later audible window
BEFORE the low-note fallback: a silent replay cannot pass. Focused test passes.
Serialized four-package suite (Temp/xm-stop-retrigger-rollback-suite-v1.log)
passes IT66/tracker54/XM75/ZX494 (one existing ignored); diff check passes.
This closes the named stopped-latch replay gap, not all mapped/rate/handoff scope.

Active proc_aa5f2f50d045: original Temp/xm-forward-tuning-holdout.py selects
8 source finetune/relative-note threshold cases through existing matched packed
pipeline. Stage/log xm-forward-tuning-holdout-v1; no runtime edits during build.
Remaining reference tails are still open. Overall IN PROGRESS / NOT ACCEPTED.

### Previous sample-stop checkpoint

Review retry deleg_f0ed19d9 returned no stale-limit defect and a delayed-instrument
revival hypothesis. Owner authored xm-forward-retrigger-adjudication-v1:16
matched source/cart/reference controls. All ED2 with/without instrument behavior
matches, so do not patch that intentional revival. All-row checks instead exposed
four genuine E9/Rxy audible-revival failures plus one small reference-only tail:
11/16 pass despite the initial-window report passing16/16.

Root cause was new cutoff clearing note_on, disabling tick effects and retrigger.
Introduced xm_loop_stopped (default false, cloned channel state) to silence only
sampling while retaining note/effect processing. Existing sample restart/reset
assignments in channels, row processing, effects and tick clear the latch.
After matched rebuild, xm-forward-retrigger-repair-v1 all-row checks improve to
15/16; all four sustained revival failures pass. Only length1/note89/E92 retains
reference transient mean5.631519 vs native0, with later silence and low-note
revival matching. Original two stop-tail comparisons remain open. No full parity.
Focused regression now asserts stopped PCM remains silent after pitch lowering,
clones stop state, and resumes audibly through retrigger_sample. It and diff check
pass. Verification proc_2ea2bd3d5222 completed exit0. Owner inspected actual
artifacts: IT66/tracker54/XM75/ZX493 tests pass (one ZX ignored), width24/24,
IT inventory25/25; both XM/IT native rollback logs confirm three historical
handoff+cold checkpoints and64 subsequent PCM/state frames each. Affected builds
and final diff check passed. No background verification remains active. These
historical carts do not specifically exercise the new stopped-sample latch;
new stop/retrigger snapshot/cold-replay coverage and transient gates remain open.
Temp/xm-sample-stop-preservation-v1.log command scope:
serialized four-package tests/builds, fresh width24, IT inventory and both
historical retained-cart rollback checks. No source edits until chain completes.

### Previous source-loop limit checkpoint

Owner adjudication xm-forward-stop-ramp-adjudication-v1 rendered both original
slide-crossing sources through the hash-guarded installed reference with ramping0
and-1. All four captures retain the same tail: first change at24724, final
nonzero26670 (44100Hz), versus native stop after24695. Disabling reference
ramping does not remove it. Original nonloop past-end-offset control has a
shorter, distinct tail; do not generalize one tail formula to every voice stop.
These are output-shape observations, not evidence of continued pitched playback.
No runtime edit made from this calibration; two transient checks stay open.
Review deleg_726dbc78 exhausted50 calls and returned no verdict (transport
failure at completion). Owner inspected its live trace; it contains reads, not
a completed finding or acceptance report. One materially narrowed retry
deleg_f0ed19d9 is active: maximum12 calls, selection/delayed-note/E9/Rxy stale
limit and stopped-state lifecycle only. No broader mixer/parser audit. Reviewer
cannot mutate/build; runtime source remains frozen. No acceptance from failed review.


Fresh xm-forward-recovery-baseline-v1 original slide controls establish terminal
voice stop, not temporary mute: downward pitch after stop remains silent; new
low note revives playback. Added xm_forward_loop_limit to TrackerInstrument,
TrackerSample, TrackerChannel, derived from original XM loop length and sample
rate before rounded PCM conversion. Mapped and single-sample conversion and
note selection propagate it; IT explicitly defaults0. No NCXM layout change:
existing source loop metadata remains authoritative. Shared sampler stops only
forward loops with this nonzero limit when output increment exceeds it. Channel
cloning retains the limit; focused test covers disabled/pingpong isolation and
cloned PCM, position, note state. Focused test and diff check pass after fixing
a test-only invalid whole-TrackerChannel PartialEq assertion.

Initial limit-v1 probes stopped early at renderer's audible-song guard because
correctly silent controls have zero peak. Guard was NOT weakened: original probe
now authors an explicit low-note final-row revival control. Fresh matching-source
reference/native limit-v2 runs complete16/16 threshold and8/8 initial-window
recovery captures; both players' low-note tails are audible. Independent all-row
windows recorded in each recovery-windows.json pass16/16 threshold but6/8 recovery.
Two slide-crossing controls retain a short reference stop ramp (window mean25.492
versus native0); later silent rows and revival match. This transient is OPEN,
not hidden by aggregate initial-window pass. Broader mapped/tuned/rate coverage,
stop/retrigger rollback, final source audit and full acceptance remain open.

Verification proc_86eabb244800 completed exit0; owner read the actual reports
and logs: IT66/tracker54/XM75/ZX493 library tests pass (one ZX ignored),
short-loop width24/24, ordinary loops16/16, IT inventory25/25, and both XM/IT
historical rollback native logs assert three handoff+cold checkpoints with64
subsequent PCM/state frames each. Build and final diff check passed.
Temp/xm-forward-limit-preservation-v1.log retains the chain below. No process
from this checkpoint remains active; two transient windows and broader new
metadata/stop/retrigger rollback/source audit gates remain open.
Verification command scope:
serialized four-package offline tests, affected builds, fresh width24 and ordinary
loop16 controls, IT inventory, XM/IT historical rollback on retained carts, diff
check. All six Temp variables explicit; reference filter2 only in the named
short-loop controls, normal defaults preserved. Previous proc_d273b65b7691 was
for the earlier modulo-only surface and cannot validate these new metadata edits.
Owner verified its exit1: IT inventory builds overlapped insertion of the new
field and encountered a missing xm_forward_loop_limit initializer. The completed
initializer subsequently built and passed the focused test. Discard this
mixed-source run as acceptance evidence. Freeze runtime edits until the current
replacement rebuilding verification chain finishes.
No source/licensing downloads, source-asset edits, commits or cleanup performed.

### Earlier forward-wrap candidate


Threshold baseline xm-forward-threshold-baseline-v1 completed16 cases,5 pass,
exit1 at retained-failures assertion. Reference length1 changes from1086 at77
 to0 at78; length2 changes from1086 at89 to0 at90. These are source-loop-dependent
observations, not permission to silence all high notes or change global gain.
Owner traced shared sample_channels and mixer/silent advancement, reproduced
forward out-of-loop position with forward_multiple_crossings_stay_inside_loop
(Temp/xm-forward-wrap-before-v1.log exit101), then replaced single forward
subtraction with modulo loop length. Both mono/stereo PCM and position now pass;
both forward/ping-pong multiple-crossing tests pass together. No new state.

After affected rebuilds, xm-forward-threshold-wrap-v1 retains all16 source hashes
and passes6/16. The audible length2/note89 improves542.972562->1086 matching
reference1086. Remaining10 controls are reference-silent, native1086; unchanged
2% gate fails explicitly. Candidate fixes native bounds, NOT full reference
silence semantics. Earlier matched silence can change under proper wrapping;
do not borrow previous short-loop acceptance for this changed surface.

ACTIVE proc_d273b65b7691: Temp/xm-forward-wrap-preservation-v1.log;
offline serialized four-package libraries, xm-forward-wrap-ordinary-v1,
it-after-forward-wrap-v1, XM/IT forward-wrap rollback stages against retained
pingpong historical carts, final diff check. No source/cart freshness claim for
those reused historical carts. Overall NOT ACCEPTED. Next repair must establish
source-rate/loop-length suppression and recovery from actual source controls,
including pitch changes; resampled rounded loop metadata alone may shift the
threshold and must not be treated as source metadata without verification.

### Previous multiple-boundary checkpoint

Owner reproduced the high-rate state failure in the real native sampler:
ping_pong_multiple_crossings_preserve_position_and_direction failed before
repair (Temp/xm-multicross-before-v1.log, exit101). A four-frame increment over
loop[1,3] did not preserve position/direction across a complete round trip.
The shared ping-pong sampler now folds any residual out-of-range position modulo
the double loop length AFTER the existing first reflection. Ordinary crossings
and forward-loop behavior are unchanged. The same mono/stereo, both-direction
state regression passes. Extended existing stereo snapshot/cold rollback to
include note85 with a two-frame loop; both old and high-rate cases match2048
subsequent PCM pairs and final state/channels.

Original tools/tracker-debug/xm_short_loop_probe.py uses existing matched build,
source/cart/native/reference helpers. Frozen xm-short-loop-before-v2 versus
xm-short-loop-after-v1 retain24 identical source hashes. Six high-rate ping-pong
captures improve native steady magnitude543 ->1086 (both channel layouts), but
reference remains1054, outside the unchanged2% bound. Full gate remains10/24.
Forward tiny loops also have reference-silent/high-note cases; source volume/gain
must not be patched speculatively. Initial probe assumed reference audibility and
stopped early in before-v1; before-v2 explicitly records reference_audible without
labeling silent controls proof of preserved audible looping.

Reference adjudication completed: owner verified both report SHA256 values and
read all72 WAVs in xm-short-loop-high-rate-adjudication-v1 and
xm-short-loop-high-rate-constant-controls-v2. Medium/wide neutral controls confirm
1086 for filters1/2 versus1054 for filter8 at high notes, across loop types.
This is reference-filter-dependent, not evidence for a runtime gain correction.
Tiny forward-loop silence remains unresolved; the first adjudication's constant
controls also silence at note49, so do not infer universal note thresholds from
them. Fresh matched packed run with explicit XM_REFERENCE_FILTER=2 completed:
proc_dd4bbfd2c4bb, xm-short-loop-linear-after-v1, exit1; Temp log of the same name.
Owner verified the exit is the final retained-failures assertion after all24
captures, not a build or orchestration failure. Historical rollback verification
completed as proc_f36f3c02fdca, exit0; Temp/xm-multicross-historical-rollback-v1.log.
Owner read both native.log files: each reports three historical checkpoints,
handoff+cold,64 subsequent frames with PCM/state/channel equality. Builds passed.
Boundary diagnostic proc_989c72adcda3 failed before capture (missing compatibility
import in the temporary wrapper); adding tools/tracker-debug to sys.path fixed
orchestration. Owner reran xm-forward-boundary-baseline-v2 synchronously:18 cases,
16 pass, final retained-failures assertion exit1. All lengths3/4/8/16 pass at
notes73/85/96; lengths1/2 fail only at96. Length1/note85 is matched silence, not
proof of audible looping. No runtime change from these mean-only findings.
Active threshold run: proc_93e16f1b7a00, xm-forward-threshold-baseline-v1;
Temp/xm-forward-loop-boundary-probe.py now selects lengths1/2 and notes77/78/79/80/
89/90/91/92, still matched packed playback with reference filter2. It checks
whether the silence transition tracks a playback step exceeding source-loop
length before selecting any source-versus-resampled-rate runtime semantics.
No gain changes or broad creator exclusions.
It rebuilds affected binaries then replays the existing pingpong historical carts
through fresh XM and IT rollback renderers; this is not fresh repacking evidence.
Default filter8 and existing2% gate remain unchanged. No silent gate relaxation.
Preservation process proc_a46d3fef2e90 completed exit0. Owner read test log:
IT66/tracker54/XM75/ZX491 passed (one ZX ignored); ordinary loop report16/16;
IT inventory25/25; final diff check passed. These are bounded preservation
results, not broad compatibility acceptance. Linear-reference packed report is
complete at20/24: all ping-pong cases pass; four remaining failures are forward
loops of lengths1/2 at note96 in mono/stereo (native magnitude542.740816 versus
reference0). Retain those failures; do not infer silence semantics from aggregate
means alone. Historical PCM/state rollback refresh follows before new repairs.
Log: C:/Users/rdave/AppData/Local/Temp/xm-multicross-preservation-v1.log.
Command chain: offline serialized nether-xm/nether-it/nether-tracker/nethercore-zx
library tests; xm_loop_probe.py xm-multicross-ordinary-loops-v1;
Temp/it-connected-inventory.py it-after-multicross-v1; git diff --check.
All six Temp variants explicitly set; no downloads/assets/license/dependency
changes. New high-rate scope remains NOT ACCEPTED while oracle gaps are open.

### Previously verified ping-pong endpoint scope

Review deleg_2a870cfe found no introduced defect for valid parsed XM in the
bounded endpoint repair. Owner read integration artifacts: IT66/tracker54/XM75/
ZX490 tests pass (one existing ignored); XM historical1344 rows/2954 events and
IT historical2112 complete rows/4570 events retain zero note/held-state mismatches
and zero clipping. IT preservation inventory25/25 passes. Both fresh historical
snapshot-handoff/cold rollback logs assert three checkpoints and64 subsequent
PCM/state frames each. Integration log: xm-pingpong-integrated-v1.log in Temp.
This supports the named loop/endpoint scope, not arbitrary rates or all formats.

Concrete remaining review lead: sample_channels reflects only one crossed loop
boundary. At loop[1,3], position2.5 and increment4, the current reflection produces
position-0.5, outside the loop. This pre-existing high-rate/short-loop gap is a
repair obligation: reproduce in original packed sources, fix shared folding with
forward/ping-pong and stereo controls, then compare subsequent PCM/state rollback.
Reviewer also noted possible stale IT sustain metadata under a reused XM channel,
but found no normal parsed XM failure; do not patch that hypothetical state without
real-path reproduction. Overall XM/IT remains IN PROGRESS / NOT ACCEPTED.

### Endpoint repair evidence

Owner continued the open XM sample-loop inventory with original
`tools/tracker-debug/xm_loop_probe.py`: source -> fresh pack -> native -> guarded
installed official reference. Cases cover8/16-bit mono/stereo, forward/ping-pong,
source loop lengths8/64, source8363Hz/output44100Hz and finite termination.
`xm-loop-width-baseline-v2` passes8/16; ping-pong endpoint behavior is a real defect.
An initial edge-group period estimator failed on smooth ping-pong waves, and its
replacement initially picked a near-zero-lag correlation peak. The final probe
uses bounded nonzero-lag normalized autocorrelation with peak interpolation;
original retained WAVs were remeasured, not rewritten. No runtime gate was relaxed.

Owner repaired two connected causes: convert_xm_sample now resamples the ping-pong
loop region with aligned phase and held endpoint; sample_channels holds the last
loop frame at reversal instead of reading post-loop PCM or interpolating to the
opposite end. Stereo lanes share the same frame coordinate. Forward behavior is
preserved; IT shares the sampler but not the XM conversion, so IT regression and
historical checks are explicitly required below. No gain compensation/asset edits.

`python tools/tracker-debug/xm_loop_probe.py xm-loop-width-repair-v2` passes16/16.
Same-source before/after remeasurement in that stage's adjudication.json retains
8/16 ->16/16 with period0.6%, mean2%, polarity, finite playback/no-clipping checks,
plus a ping-pong adjacent-step bound against reference (one PCM unit allowance).
Short ping-pong native largest step improves72 ->7, versus reference8. New original
unit ping_pong_interpolation_holds_endpoint_without_tail_or_wrap passes mono/stereo
positions2.5/3.0 with a sentinel tail; all9 CLI audio-conversion tests pass.
No wider pitch/offset/loop-length or universal stereo acceptance is claimed.

ACTIVE: proc_bda8d1c99f4b, completion notification enabled, serialized offline
suite and rebuilt historical XM+rollback then IT inventory+historical+rollback.
Log: C:/Users/rdave/AppData/Local/Temp/xm-pingpong-integrated-v1.log.
Named stages: xm-pingpong-historical-v1, xm-pingpong-rollback-v1,
it-after-pingpong-v1, it-pingpong-historical-v1, it-pingpong-rollback-v1.
Commands reuse existing Temp/xm-connected-capture.py (--require-pass),
xm-historical-rollback.py, it-connected-inventory.py, it-historical-capture.py,
it-historical-rollback.py; each rollback receives its corresponding fresh stage.
All six Temp variants set explicitly; installed reference guard/no downloads.
Read-only review deleg_2a870cfe covers the shared endpoint/conversion change.
These results are pending; overall XM/IT remains IN PROGRESS / NOT ACCEPTED.

### Accepted raw supplied-handle API scope

Review deleg_31c2fed4 passed the scoped ABI and trust boundaries and identified
one bounded allocation-order hardening item. Owner confirmed and repaired it:
parse directly from the checked WASM module slice; reject sample_count unequal
to expected_sound_handle_count before materializing the handle vector. This also
removes the unnecessary raw XM copy. All memory arithmetic/range checks and
explicit-silence semantics remain intact. Added an in-bounds16000-handle wrong-
count case to the existing real-WASM regression. No further review wave was
needed for this narrow validation reorder; owner inspected the final source.

Post-repair real-WASM test passes playback, invalid inputs, init guard, supplied
mono/stereo mapped handles, legacy sample-less loading, ROM loading, and snapshot/
cold subsequent PCM/state parity. Full serialized offline ZX library suite passes
489 tests,0 failures,1 existing ignored. Six Temp variants explicitly set.
Log: C:/Users/rdave/AppData/Local/Temp/xm-raw-api-hardened-suite-v1.log.
`git diff --check` passes after removing introduced trailing whitespace.

Scoped raw API repair is complete; no worker or review remains active. This does
not close the overall XM/IT inventory. Stereo ping-pong/offset/envelope breadth,
malformed/corpus inputs, and broader equivalent IT sample-format/interaction
validation remain open; IT stereo conversion still folds to mono and is not
accepted as stereo preservation. Earlier pending-review notes are historical.

### Raw API implementation and verification history

Implementation worker deleg_1bf5d91c exhausted its iteration cap after source and
cargo-check only. Owner integrated directly rather than retrying its broad brief.
Added original in-tree ffi/audio/tracker/tests.rs: actual Wasmtime imports call
load_tracker_with_samples, legacy load_tracker, music_play, and rom_tracker.
The real WASM regression passes invalid counts/pointers/nonexistent sound handles,
explicit zero silence, init-only rejection, legacy sample-less acceptance, mixed
mono/two-slot stereo selection, snapshot+cold reconstruction with2048 subsequent
PCM pairs and state equality, and audible existing ROM loading. Initial fixture
creator padding was too long; fixed bounded padding without runtime changes.

Owner command (six explicit Temp variants, offline, serialized):
`cargo test --offline -p nethercore-zx --lib raw_tracker_supplied_handles_real_wasm_and_rollback -- --test-threads=1`
passes. Full affected library suite also passes IT66/tracker54/XM75/ZX489 with
one existing ignored. Log: C:/Users/rdave/AppData/Local/Temp/xm-raw-api-owner-suite-v1.log.

Binding correctness was NOT accepted from worker timestamp changes. Owner used
installed target/debug/xtask.exe ffi check --console zx: initially out of sync.
Installed ffi generate/check then passed with212 declarations; no downloads or
uncached package installs. Owner restored pre-existing unrelated EPU comments and
verified all non-comment generated C/Zig content remained identical. Current
byte-exact generator check may therefore still report preserved comment drift;
this is code/ABI equivalence, not a claim of full byte-for-byte freshness.
Legacy sample-bearing failure is documented in Rust/C/Zig declarations; old ABI
signature remains unchanged. No existing engine/mapped-module repair was invented.

Read-only bounded review deleg_31c2fed4 checks this new WASM trust boundary and ABI;
its result is pending. No implementation worker/build remains running. Overall
XM/IT acceptance is still open; earlier active-process descriptions are historical.

### Prior stereo review adjudication

Read-only review deleg_ab400d3f returned three findings; owner checked actual
source and reference playback rather than applying them blindly:
- Short forward loop beyond rounded PCM: confirmed and repaired in
  tools/nether-cli/src/audio_convert.rs. A valid independently rounded loop now
  extends output as necessary before filling its cycle. Added mono/stereo source
  rate44100, two-frame sample, start1/length1 regression. Serialized CLI81tests
  pass; rebuilt connected xm-stereo-owner-loops-v3 passes4/4 unchanged thresholds.
- Alleged normal mixed one-/multi-sample packing/mapping defect: NOT REPRODUCED.
  Current parser retains96-entry map and sample metadata for single-sample
  instruments too (parser/read.rs365-376,533). Owner verified parser/packer/
  converter SHA256 match reviewer inputs. Fresh self-authored source/cart/native/
  official-reference controls in xm-mixed-review-adjudication-v1 pass2/2 for
  single-first and multi-first instrument order. Both preserve three selection
  segments: [1086,1086], [1086,-1086], [543,-543], exactly matching reference.
  Command: python C:/Users/rdave/AppData/Local/Temp/xm-mixed-review-probe.py.
  No speculative mapping patch was made. Hand-built inconsistent XmModule values
  are distinct from the review's disproved normal parsed-source scenario.
- Raw load_tracker API: current source confirmed missing supplied-handle input
  and all-zero handle vector. Bounded implementation worker deleg_1bf5d91c is
  now sole source writer/build owner. It must preserve the old ABI, add an
  explicit load_tracker_with_samples entry point with bounds/handle/count
  validation, fail explicitly for old raw loads missing required mapping, retain
  legitimate no-sample/explicit silent slots, and verify actual FFI playback and
  rollback without regressing rom_tracker. Worker must not edit this checkpoint.

All owner commands used six explicit Temp variants, offline serialized builds,
existing helpers, guarded installed reference; no downloads or source-asset edits.
Reviewer mixing.rs hash was malformed (not64 hex chars) and was not treated as
valid provenance. Overall XM/IT remains IN PROGRESS / NOT ACCEPTED.

### Previous integrated stereo evidence

Owner integration proc_75b7a765396f completed exit0. Owner read actual reports and
log, not only the completion status: CLI81, IT66, tracker54, XM75, ZX488 tests
passed (ZX1 existing ignored). Fresh stereo/mono identity4/4 passed. Historical
XM retained1344/1344 authored rows and2954 note events with zero note mismatches,
exact row order, native peak0.950225830078125 and zero clipping. Historical
snapshot handoff and cold reconstruction passed three checkpoints with64
subsequent PCM/state frames each. IT preservation inventory passed25/25 groups
across126 captures. Commands and artifact paths are recorded below; all six Temp
variants and offline serialized execution were used. git diff --check passed.
Independent read-only review deleg_ab400d3f remains pending; this is not overall
XM/IT acceptance. Earlier running-process text below is now historical.

### Stereo implementation and owner repair evidence

The implementation retry proc_d347ba2d049b completed child_exit_0 and delivered
source changes (not merely a proposal). Owner read the actual identity report:
all four 8/16-bit mono/stereo cases pass with native/reference means matching.
The old worker-running notes below are historical and superseded by this section.

Owner rebuilt and packed four independently authored forward-loop holdouts:
`python C:/Users/rdave/AppData/Local/Temp/xm-stereo-owner-loops.py xm-stereo-owner-loops-v1`
failed2/4. Short-loop native mean270.3095 versus reference254.2478 exposed
absolute-position resampling reading past the authored loop and shifting phase.
In tools/nether-cli/src/audio_convert.rs, convert_xm_sample now resamples the
complete forward-loop cycle using the same independently rounded start/length
as converter metadata and wraps interpolation within the source loop. The
shared mono/stereo fix leaves IT conversion and ping-pong handling unchanged.

`...xm-stereo-owner-loops.py xm-stereo-owner-loops-v2` passes4/4, with unchanged
mean2% and period0.6% bounds, channel polarity, termination and no clipping.
Short-loop native means now[254,-254]; longer-loop means[524.3435,-524.3435].
Reference means respectively[254.2478,-254.2472] and[526.5138,-526.5136].
Reports/carts/WAVs are retained in target/tracker-compatibility/{v1,v2 stage names}.
Focused offline serialized nether-cli xm_stereo_resampling and nethercore-zx
xm_stereo_snapshot_and_cold_rollback tests both passed. Snapshot/cold test compares
2048 subsequent PCM pairs and final state/channels. Initial new unit assertion
incorrectly assumed exact continuous ramp mean after discrete resampling; replaced
with exact authored start and wrap-interpolation values (1024,1707), not a relaxed
connected threshold. Mono output equality to the stereo left lane is asserted.

ACTIVE owner integration process: proc_75b7a765396f; completion notifications on.
Log: C:/Users/rdave/AppData/Local/Temp/xm-stereo-owner-integrated-v1.log.
Serialized command chain (all six Temp variables explicitly set; offline):
- cargo test --offline -p nether-cli --bin nether -- --test-threads=1
- cargo test --offline -p nether-xm -p nether-tracker -p nethercore-zx -p nether-it --lib -- --test-threads=1
- python tools/tracker-debug/xm_stereo_probe.py xm-stereo-owner-identity-v2
- python C:/Users/rdave/AppData/Local/Temp/xm-connected-capture.py xm-stereo-owner-historical-v1 --require-pass
- python C:/Users/rdave/AppData/Local/Temp/xm-historical-rollback.py xm-stereo-owner-historical-rollback-v1 xm-stereo-owner-historical-v1
- python C:/Users/rdave/AppData/Local/Temp/it-connected-inventory.py it-after-stereo-owner-v1
- git diff --check
No parallel source writers/builds. Read-only review deleg_ab400d3f inspects actual
stereo metadata/runtime/rollback and the owner's resampling repair. Its result and
the active integration results remain pending; no overall XM/IT acceptance claim.

## Historical stereo repair launch (superseded status)

Fresh original `tools/tracker-debug/xm_stereo_probe.py` closes the prior stereo
inventory uncertainty with a demonstrated failure, not an assumed exclusion.
`python tools/tracker-debug/xm_stereo_probe.py xm-stereo-identity-before-v1`
rebuilds the packer/runtime and exercises 8/16-bit mono and stereo source/cart
playback. Mono native/reference means [1086,1086] match; both stereo native means
remain [1086,1086], while reference means are [1086,-1086]. Result 2/4: source
channel identity is lost. Artifacts: target/tracker-compatibility/
xm-stereo-identity-before-v1/report.json and retained source/carts/WAVs.

Owner traced missing stereo metadata through XM parser/extractor/model and the
mono runtime mixer; no runtime repair is claimed yet. A single supervised Codex
hermes-deep/workspace-write worker owns this bounded end-to-end repair in-place.
Do NOT start another source writer or parallel build while it is active.
- Original handle proc_3299293f530c exited0 with a proposal only; no repair was
  delivered. Original run.json/run.log/final.md remain preserved, not acceptance.
- User explicitly approved the brief update and supervised implementation retry
  after the tool-approval timeout. The updated brief approves the design and
  requires implementation plus real verification, not another proposal.
- Active retry handle: proc_d347ba2d049b (completion notification enabled).
- Brief/state/log/final: C:/Users/rdave/AppData/Local/Temp/
  xm-stereo-repair-20260908/{brief.md,run-implementation.json,
  run-implementation.log,final-implementation.md}.
- Retry watchdog read-back confirmed status running/reason started.
- This is the one materially changed implementation retry. If it fails, owner
  integrates/repairs directly where feasible; do not repeat the unchanged brief.
- Constraints: preserve dirty tree, existing mono/XM/IT behavior and old artifacts;
  no dependencies, downloads, source assets, reference-code copying, commits,
  pushes, license changes, cleanup/deletion or SpecCade edits. Worker must build,
  run connected controls and add stereo rollback/regressions; owner must inspect
  and independently verify actual artifacts on completion. Worker summaries
  alone never establish acceptance. This is not a stereo acceptance claim.

On completion or recovery, read the state/final once, integrate/repair directly
as needed, and update this existing checkpoint. Overall goal remains
IN PROGRESS / NOT ACCEPTED. Prior bounded mono claims do not cover this defect.




## Independent stereo loop-unit holdout (reference-only)

While the supervised implementation retry owns source/build writes, the owner ran
four original reference-only controls; no native build or runtime edit was made.
`python C:/Users/rdave/AppData/Local/Temp/xm-stereo-loop-oracle.py` exited0;
`python C:/Users/rdave/AppData/Local/Temp/xm-stereo-loop-oracle.py --check` passed4/4.
Both commands explicitly set all six Temp environment variants as above.

Retained stage: `target/tracker-compatibility/xm-stereo-loop-layout-oracle-v1/`.
Its report.json records source hashes and reference SHA256
`9d809056e40e3d004b1ab9275081ee8ddb9d1874999369189d0ba172e7c3131c`.
Existing filter_reference() verified the installed executable/version after an
existence guard; no download fallback, external source reading, or asset edits.

The authored sample has two separately delta-encoded 64-frame ramp planes, with
opposite channel polarity. For both8/16-bit stereo, header loop lengths of
16*bytes_per_sample produce approximately8 source-frame loops; doubling the
header loop byte fields produces approximately16 source-frame loops. Measured
output periods:42.184466 and84.384615 frames at44100Hz (source8363Hz).
Thus the tested stereo loop byte fields include both channels: divide by sample
width AND channel count when converting to per-channel frame coordinates.
The left/right means preserve opposite polarity, with identical measurements
between8-bit and16-bit encodings. These finite controls cover forward loops only.

This resolves a concrete metadata-unit requirement, NOT native stereo acceptance.
Owner must run these sources through the rebuilt packed/native path and verify
loop timing/channel identity plus rollback after the worker returns. Wider
ping-pong, offset/tuning, mapped-sample and IT stereo coverage remains open.

## E9 + volume-column portamento review closure (2026-09-08)

Review deleg_d7eb1b1a identified a stale-target concern, not a valid universal
repair prescription. E5+20 is not directly encodable in XM; fresh original E5x
controls use representable values. Reference playback requires different standard
and legacy handling, so the suggested unconditional target correction was not used.

- Every E9 restarts the latest pitched key, including an unfinished porta target.
  `xm_retrigger_note` preserves that key independently of the active sample note.
- Standard FT2 preserves its target. An upward arrival latches a subsequent
  downward step with target clamp until a new target. `xm_porta_target_reached`
  records that asymmetric state; this is not a general “either direction reached”
  flag. Each retrigger resets the current/base pitch, not a guessed accumulator.
- Legacy E9 refreshes the active target and clears the latch. IT stays separate.
- Channel clones retain both fields. Rollback scenarios42/43 add slow/fast
  E5x+volume-porta+E92 across standard/legacy, both period modes and four tunings.

Original probe: `tools/tracker-debug/xm_finetune_probe.py NEW_STAGE retrigger-porta
--semitone --extremes`, optionally `--legacy`, `--slow-porta`, `--porta-down`,
`--near-porta`. Per-tick carrier windows avoid E9 sample-position discontinuities;
whole-row crossing estimates were not valid for that interaction. Pitch tolerance
remains .006. Failed intermediate v1-v9 artifacts are retained, not accepted.

Fresh accepted artifacts under target/tracker-compatibility/:
- xm-e9-target-{standard,legacy}-{up,down}-after-v10: 24/24 each.
- xm-e9-target-{standard,legacy}-fast-final-v10: 24/24 each.
- xm-e9-target-near-v10: 24/24. Total current target holdout: 168/168.
- xm-e9-target-standard-up-after-v10/before-window-audit.json: original source
  hashes identical; retained before WAVs re-evaluated using the final tick windows.
  Matched comparison improves from 24/96 before to 96/96 after.
- xm-e9-target-effects-v10: 236/236.
- xm-e9-target-historical-v10: source SHA unchanged, 1,344/1,344 rows,
  2,954 events, zero musical-state mismatches/clipping; finite order21/row0 end.
- xm-e9-target-rollback-v10: snapshot handoff+cold, three checkpoints including
  loop boundary, 64 subsequent PCM/state-equal frames per checkpoint.
- it-e9-target-preserved-v10: 25/25 existing groups, 126 retained captures.

Commands used the existing reviewed Temp/xm-connected-capture.py (with
--require-pass), xm-historical-rollback.py (explicit fresh historical stage),
and it-connected-inventory.py, plus the existing xm_effect_probe.py. All use six
Temp environment variants, offline serialized builds/tests, and the installed
hash-verified official reference; no download fallback or generic --case.
Temp/xm-e9-target-tests-v1.log: IT66, tracker54, XM73, ZX487 pass (one existing
ignored ZX test). No later runtime edits occurred after that test run.

Bounded fresh review deleg_8a9387f2 found no scoped defect. Owner inspected actual
reports and same-source before/after windows. Cached OpenMPT Snd_fx.cpp retrigger,
note-change and tone-portamento caller flow was read as behavioral context under
the existing source pin f83cedb0cd5446e4dfaa83ac97e3087107e26767; executable evidence
remains separately pinned in the reports. No reference code, tables, tests, or
assets were copied, ported or added. Earlier provisional accumulator experiments
were discarded. No engine replacement, source-asset edits or licensing changes.

This closes the reviewed interaction for the tested scope. Overall wider
compatibility remains IN PROGRESS / NOT ACCEPTED; no universal compatibility claim.

## FT2 sample-finetune boundary repair (2026-09-08)

Extended the existing original E5x probe with --semitone (single-semitone baked
transpose) and --quantization (13 signed finetune boundaries, no E5 override).
The ordinary semitone scope/gliss/delay/retrigger stages pass24/24 each. Extreme
standard gliss/delay/retrigger initially fail4/24 each (source fine127, linear
mode); legacy extreme controls pass. The boundary grid also exposes standard
Amiga fine-9. Failed stages retained; the0.006 pitch tolerance is unchanged.

Fresh official reference measurements show standard FT2 default sample-finetune
plateaus at lower multiples of8 in both period modes. The linear reference grid
fits the independently stated rate formula using that quantization with maximum
relative error5.9241664090992074e-6. A trial legacy lower-multiple-of2 model failed
its0.0001 explanatory check (max0.0008998504441326549): it was NOT implemented.
Legacy behavior remains untouched, rather than inferred from the standard model.

Original native repair: `xm_sample_finetune` is shared by fresh note initialization
and E9 restoration. Both retain the difference from raw tuning already baked into
PCM in existing `xm_finetune_delta`; E5 replaces that delta instead of applying it
twice. E9 restores the quantized default, not the raw source fine. The existing
porta/gliss and snapshot paths consume/preserve the same scalar. IT initialization
keeps delta zero and its E9 path stays gated. No packed format, source asset,
reference implementation, dependency, licensing or mixer gain change was made.
Rollback matrix now includes source fine-9 and127; a direct boundary/legacy-isolation
unit regression covers the measured grid.

Verified retained reports under target/tracker-compatibility:
- xm-fine-grid-standard-v3:26/26; xm-fine-grid-legacy-preserved-v2:26/26.
- xm-e5-semitone-extreme-standard-{gliss,delay,retrigger}-v2:24/24 each.
- xm-fine-quantization-effects-v1:236/236.
- xm-fine-quantization-octave-gliss-v1 and xm-fine-quantization-selection-v1:24/24 each (extreme finetune holdouts).
- xm-fine-quantization-historical-v1:1344 rows,2954 events, exact tested row/order
  and note state, finite termination, zero native/reference clipping.
- xm-fine-quantization-rollback-v1:three checkpoints, snapshot handoff+cold
  reconstruction,64 subsequent PCM/state/channel frames each including loop boundary.
- it-fine-quantization-preserved-v1:25/25 groups,126 captures.
- xm-fine-quantization-tests-v2.log in Temp:IT66,tracker54,XM73,ZX487 pass,
  one pre-existing ZX ignored.

Commands used all six TEMP/TMP/TMPDIR and lowercase variants explicitly set to
C:/Users/rdave/AppData/Local/Temp; fresh stage names and rebuilt offline packer/runtime:
```sh
python tools/tracker-debug/xm_finetune_probe.py xm-fine-grid-standard-v3 --semitone --quantization
python tools/tracker-debug/xm_finetune_probe.py xm-fine-grid-legacy-preserved-v2 --semitone --quantization --legacy
# Each named mode was run separately; replace MODE with gliss, delay, retrigger.
python tools/tracker-debug/xm_finetune_probe.py xm-e5-semitone-extreme-standard-MODE-v2 MODE --semitone --extremes
cargo test --offline -p nether-xm -p nether-tracker -p nethercore-zx -p nether-it --lib -- --test-threads=1
python tools/tracker-debug/xm_effect_probe.py xm-fine-quantization-effects-v1
python C:/Users/rdave/AppData/Local/Temp/xm-connected-capture.py xm-fine-quantization-historical-v1 --require-pass
python C:/Users/rdave/AppData/Local/Temp/xm-historical-rollback.py xm-fine-quantization-rollback-v1 xm-fine-quantization-historical-v1
python C:/Users/rdave/AppData/Local/Temp/it-connected-inventory.py it-fine-quantization-preserved-v1
```
Installed executable hash verification remains in existing filter_reference(); no
download fallback ran. Source pin provenance remains separate and unchanged.
Bounded source review deleg_d7eb1b1a pending. This repair is measured, built and
exercised, but not a claim of complete XM/IT compatibility. Overall IN PROGRESS.

## XM/IT seek and generation-rate matrix (2026-09-08)

Expanded the existing XM rate-change regression into a shared XM/IT check at
44.1/48/96 kHz with cold and previously-used receivers:12 combinations, five
subsequent audio frames each. Every frame asserts audible PCM, exact PCM equality,
TrackerState bytes and complete channel state against uninterrupted native execution.
Receiver warm-up includes playback at44.1kHz, a recent rollback-cache entry and
private row-cache seeking. The target is order1,row2 at tick zero after an explicit
sample-clock reset. IT includes its existing swing/NNA/envelope sample fixture.
This closes this cross-format rate/seek holdout, not arbitrary rates or all seeks.

Owner traced all `seek_to_position` call sites. Its only non-test caller is
`reconstruct_playback`, after reset, requesting origin(0,0); public recovery then
uses actual sample/tick playback. The earlier non-origin private-cache limitations
remain documented, but are not a demonstrated defect on that production path.
No engine repair or cache redesign was needed. Existing playback/packing evidence
remains current because only the regression changed.

All six TEMP/TMP/TMPDIR and lowercase variants explicitly set to
`C:/Users/rdave/AppData/Local/Temp`:
```sh
cargo test --offline -p nether-xm -p nether-tracker -p nethercore-zx -p nether-it --lib -- --test-threads=1
```
Exit0, log `C:/Users/rdave/AppData/Local/Temp/tracker-rate-seek-matrix-v2.log`:
IT66, tracker54, XM73, ZX486 passed; one pre-existing ZX ignored.
Broader extension/corpus and untested interactions remain IN PROGRESS / NOT ACCEPTED.

## Recognized extension boundary adjudication (2026-09-08)

Closed the earlier ambiguous truncated-container review lead without changing
runtime/parser behavior. `extension_containers_distinguish_empty_from_truncated_fields`
uses the existing original multisample source: empty STPM/XTPM containers parse and
pack; every nonempty incomplete field-header/payload prefix returns UnexpectedEof.
XTPM payload lengths include the instrument multiplier. Complete opaque fields,
marker-looking payload bytes, consecutive empty containers, zero-length unknown
fields, and unrelated opaque trailing data remain accepted. Recognized tags are
not scanned inside payloads. This supersedes earlier statements that this specific
empty-versus-incomplete-field boundary still needs adjudication; it does not close
other extension behavior or the measured empty-container onset-ramping difference.

No speculative parser restriction or engine change was needed. Existing connected
empty-container reference measurements remain the behavioral evidence for accepting
empty containers. The added test is a native malformed-input contract check, not a
claim that the external reference rejects every malformed prefix identically.

Verified command, all six TEMP/TMP/TMPDIR plus lowercase variants explicitly set to
`C:/Users/rdave/AppData/Local/Temp`:
```sh
cargo test --offline -p nether-xm -p nether-tracker -p nethercore-zx -p nether-it --lib -- --test-threads=1
```
Exit0; retained `C:/Users/rdave/AppData/Local/Temp/xm-extension-boundaries-v1.log`:
IT66, tracker54, XM73, ZX485 passed; one pre-existing ZX ignored. Runtime source and
packed formats are unchanged, so this test-only addition does not invalidate the
immediately preceding fresh XM/IT playback and rollback evidence. Overall broader
acceptance remains IN PROGRESS / NOT ACCEPTED.

## IT preservation after XM E5x repair (2026-09-08)

Following the explicit XM acceptance immediately below, equivalent existing IT
coverage was rerun against the current build. No IT engine/source assets changed.
`it-after-xm-e5-inventory-v1/{inventory,captures}.json` passes25/25 groups and126
fresh captures. `it-after-xm-e5-timing-v1` passes speed/tempo, octave, cut/restart,
native end and separately asserted reference end-tail checks. Swing controls in
`it-after-xm-e5-swing-v1` pass bounded volume/pan variation and summed-gain checks;
random sequences are not required to match. `it-after-xm-e5-balance-v1` passes all11
mode3/mode4, five-position pan and intermediate-preamp controls within1 PCM16 unit.

Fresh unchanged historical source in `it-after-xm-e5-historical-v1/report.json`:
2,112 authored rows,4,570 pitched events, exact source order, zero note and held
state mismatches, zero clipping. Native peak0.89251708984375; reference
0.883575439453125 on this run. Random/reference variation means peak proximity
is not an acceptance predicate. The authored loop remains playing at order1,row2,
not falsely called a finite stop.

`it-after-xm-e5-rollback-v1` explicitly uses that newly packed historical cart,
not an earlier final cart. The retained renderer/log verifies three checkpoints
7/4000/10370, fresh snapshot recipient plus cold reconstruction, each comparing64
subsequent PCM/state/channel frames. Actual loop transition10422 falls within
[10370,10434). Current four-package serialized tests remain66/54/72/485 passed,
one pre-existing ZX ignored; no runtime source changed after that test run.

**IT VALIDATED/WORKING for these named scopes after the XM changes.** Whole-format,
broader extension/corpus and untested interaction acceptance remains open, not
silently inferred from this preservation run. The overall standing goal remains
IN PROGRESS / NOT ACCEPTED beyond the explicit tested scope.

Inspected existing Temp helpers were reused. The reference is now explicitly
subsong0; rollback helper accepts an optional freshly packed source-stage argument.
No download fallback, generic --case, SpecCade edit, historical-source edit,
license change, dependency, commit, push, cleanup or deletion was used.
All commands below exited0, with all six TEMP/TMP/TMPDIR and lowercase variants
set to `C:/Users/rdave/AppData/Local/Temp`; affected binaries rebuilt offline:
```sh
python C:/Users/rdave/AppData/Local/Temp/it-connected-inventory.py it-after-xm-e5-inventory-v1
cargo build --offline -p nether-cli -p nethercore-zx
python C:/Users/rdave/AppData/Local/Temp/it-historical-capture.py it-after-xm-e5-historical-v1
python C:/Users/rdave/AppData/Local/Temp/it-historical-rollback.py it-after-xm-e5-rollback-v1 it-after-xm-e5-historical-v1
python C:/Users/rdave/AppData/Local/Temp/it-timing-capture.py it-after-xm-e5-timing-v1
python C:/Users/rdave/AppData/Local/Temp/it-swing-capture.py it-after-xm-e5-swing-v1
python C:/Users/rdave/AppData/Local/Temp/it-balance-capture.py it-after-xm-e5-balance-v1
```
Official executable SHA256 remains
`9d809056e40e3d004b1ab9275081ee8ddb9d1874999369189d0ba172e7c3131c`;
external behavioral source pin remains `f83cedb0cd5446e4dfaa83ac97e3087107e26767`.
They are independent provenance, not a claim the executable matches that commit.

## Delayed zero-pan and E5x interactions (2026-09-08)

Expanded the existing pan-edge probe to zero steps, not only 1/15. Standard
baseline `xm-pan-zero-delay-standard-v1` passes34/40: all six D00+ED1/3/5 controls
fail early application and trigger reset. Legacy40/40 already passes. Standard
D00 now follows the same measured early/skip-trigger timing as nonzero pan slides;
legacy recall remains unchanged. `xm-pan-zero-delay-{standard,legacy}-v2` and
`xm-pan-zero-delay-legacy-ft2-v2` each pass40/40. Rollback scenario39 covers this
pending delayed command. No new packed metadata for this repair.

The documented E5x interaction obligation now has fresh controls in the existing
`xm_finetune_probe.py`: optional `gliss`, `delay`, `retrigger`, `--legacy`,
`--extremes`. Reference rendering now explicitly selects subsong0. Original
source fine -64/0/+64 (with baked octave transpositions), E50/E58/E5F and neutral
controls show gliss8/24 and retrigger8/24 before repair; delay24/24 already passes.
Porta/gliss incorrectly lost E5's correction; E9 incorrectly retained it.

A channel i16 `xm_finetune_delta` records override minus baked sample finetune;
the difference can exceed i8. Initialization/trigger reset it, E5 sets it, XM
porta targets and gliss quantization preserve it, E9 removes it from base/current
periods and restores source fine. Existing snapshots copy this channel scalar;
rollback scenarios40/41 cover active gliss and pending E9 with E5. IT note
initialization resets it and IT porta keeps its existing branch; no packed ABI
or source-asset changes. Corrected the new test's volume setup rather than
weakening its existing volume assertion; the initial enum-field/method-call
fixture compile errors were repaired before final execution.

`xm-e5-{gliss,retrigger}-repair-v1`:24/24 each. Legacy equivalents
`xm-e5-{gliss,retrigger}-legacy-v1`:24/24 each. Existing scope and delay preservation
(`xm-e5-{scope,delay}-preserve-v1`):24/24 each. Extreme fine -128/-1/+127 scope and
retrigger (`xm-e5-extreme-{scope,retrigger}-v1`):24/24 each.

Extreme gliss initially23/24 exposed an existing distinct bug: the XM porta
branch advanced even when current period equalled its target. Gating diff0
prevents stepping away again. Added `xm_portamento_at_target_does_not_step_away`.
The gliss probe now checks active rows2/4 as well as subsequent sustained rows;
`xm-e5-extreme-gliss-v2`, `xm-e5-active-gliss-v2`, and
`xm-e5-active-gliss-legacy-v2` pass24/24 each without changing pitch tolerance0.006.

Serialized offline four-package suites in `xm-e5-interaction-tests-v5.log`
pass66/54/72/485 (one existing ZX ignored). Fresh `xm-e5-connected-historical-v1`
passes1,344 rows/2,954 events, exact order, zero note mismatches/clipping;
`xm-e5-connected-rollback-v1` passes three handoff+cold checkpoints and64 subsequent
PCM/state frames each. Drivers rebuild affected packages before packing; all six
Temp variants explicit. Completed bounded review `deleg_97af0559` reported no
scoped finding in delta initialization, target/gliss use, E9 reversal, IT isolation
and channel snapshot copying. Its retained completion transcript was recovered
following context loss; it supplies static review, not additional executed tests.
Owner separately checked live initialization and rollback output.

Fresh `xm-e5-broad-preservation-v1/report.json` passes236/236 with the final runtime.
`xm-e5-selection-v1` and `xm-e5-selection-legacy-v1` pass24/24 each: E5 followed by
instrument-only selection, note-only restart, then explicit note/instrument.
No engine change was needed for these selection holdouts.
`xm-e5-connected-inventory-v1/{inventory,captures}.json` verifies25/25 existing
inventory groups and72 fresh source/cart/native/official-reference captures.

**XM VALIDATED/WORKING for this explicit connected inventory, the named E5/pan/
effect holdouts, unchanged historical source, and tested native rollback paths.**
No known blocking discrepancy remains in that scope. Empty-extension onset-ramping,
arbitrary non-origin row-cache seeks, untested extension/creator combinations and
a diverse historical corpus remain unaccepted; the scoped verdict does not erase
these limits. Equivalent IT inventory refresh is the next acceptance phase.
The broader whole XM/IT goal remains IN PROGRESS / NOT ACCEPTED.

Additional commands (same six Temp variables/offline configuration):
```sh
python tools/tracker-debug/xm_effect_probe.py xm-e5-broad-preservation-v1
python tools/tracker-debug/xm_finetune_probe.py xm-e5-selection-v1 selection
python tools/tracker-debug/xm_finetune_probe.py xm-e5-selection-legacy-v1 selection --legacy
python C:/Users/rdave/AppData/Local/Temp/xm-connected-inventory.py xm-e5-connected-inventory-v1
```
Each exited0; counts above were read from the retained JSON, not inferred from exit0.

## Pan-review adjudication and repair (2026-09-08)

`deleg_b440c808` found no scoped parser defect. Its two slide concerns were
adjudicated against fresh original-authored source→pack→native/reference playback,
not applied by analogy. `tools/tracker-debug/xm_pan_edge_probe.py` covers delays
1/3/5, left/right steps 1/15, both XM period modes, and P01/P10 → D00 → P00 → E00
→ P00 memory chains.

Measured standard XM applies the pan column before an EDx trigger, resets sample
pan at the trigger without taking a slide step, then resumes on following ticks.
Legacy XM correctly defers the column and **does** slide on the trigger tick.
The repair keeps that legacy behavior. Standard delayed-pan setup now runs early,
with the instrument-trigger tick gated. Legacy D00/E00 recall shared pan memory:
D00 was actually converted to `PanningLeftOnTicks`, and E00 discarded as `None`,
so the review's suggested zero-valued-column path was not the imported path.
The converter now preserves E00; XM volume-column dispatch selects measured
legacy recall while retaining standard D00's left-on-ticks behavior. IT remains
outside these XM-only branches. No new packed flag or snapshot field was added.

Before/after evidence uses identical final predicates: before-standard 4/28,
before-legacy 24/28; after-standard **28/28**, after-legacy **28/28**, and legacy
with explicit FT2 mixing **28/28**. The initial standard 0.2% level comparison
also flagged normal oracle ramp/quantization differences. Final criteria use
0.6% standard / 1.8% legacy endpoint tolerances plus an **exact native pan-reset
endpoint assertion on the standard trigger tick**. Original before artifacts
were re-evaluated with these same criteria in `owner-final-predicate-evaluation.json`;
they still reproduce every delayed-pan defect. No baseline WAV/report was replaced.

Stages: `xm-pan-review-{standard,legacy}-before-v1`,
`xm-pan-review-{standard,legacy}-after-v1`, `xm-pan-review-legacy-ft2-v1`.
Broader preservation: `xm-pan-review-{standard,legacy}-holdout-v1` passes **120/120
each**. Serialized offline package tests in `xm-pan-review-tests-v2.log` pass
66/54/72/484, with one existing ignored ZX test. Rollback scenarios 37/38 add
pending delayed pan and zero-pan memory across standard/legacy, period modes,
snapshot, backward reconstruction and subsequent PCM comparisons. The first
converter-test run failed an obsolete E00=None expectation; the updated regression
now asserts retained E00 identity required by the measured legacy behavior.

Fresh `xm-pan-review-historical-v1` retains exact row order, 1,344 rows, 2,954
note events, zero note mismatches and zero clipping. `xm-pan-review-rollback-v1`
passes all three historical handoff+cold checkpoints and 64 subsequent PCM/state
frames each. Existing helpers rebuilt affected packages before captures; all six
Temp environment variants remained explicit, offline tests serialized, installed
hash-verified oracle only. Commands used `xm_pan_edge_probe.py <stage>` with
`--legacy` / `--mix-mode 5`, the existing global-pan probe and connected historical/
rollback helpers, and the same four-package cargo test command as above.
This closes the measured review findings, not the remaining whole XM/IT goal.

## Channel-count guard and empty-container adjudication (2026-09-08)

`source_channel_count_is_validated_before_narrowing` reproduced acceptance of
zero channels; it also covers 256/257/65535, which must not wrap during u16→u8
conversion. Source parsing now checks conversion and rejects zero. Existing
representable over-limit handling is unchanged. Before: `xm-channel-count-before-v1.log`
failed. After: `xm-channel-count-after-v1.log` passed all four serialized offline
library suites (66/54/72/484; one ZX test ignored). Rebuilt CLI and ZX libraries;
`xm-parser-integrated-flow-v1` passes 46/46. Fresh
`xm-parser-integrated-historical-v1` passes 1,344 rows / 2,954 events, no note
mismatches, no clipping; `xm-parser-integrated-rollback-v1` passes handoff+cold
state and subsequent PCM at all three historical checkpoints.

The earlier marker-only STPM/XTPM rejection proposal is **not applied**.
Original-authored controls through the installed reference accept both empty
containers; `xm-empty-extension-standard-v2` and `xm-empty-extension-legacy-v2`
pass 7/7 each for loading, termination and identical end-of-tick levels relative
to each player's own unextended control. Unknown complete fields also remain
accepted. The v1 all-PCM-equality hypothesis failed: STPM changes reference onset
ramping (standard control: 884 interleaved samples differ, maximum 1257 integer
units; tick endpoints unchanged). V2 records `whole_pcm_equal` separately and
only claims endpoint preservation—not complete waveform equivalence or ramping
parity. This is evidence against treating an empty marker as necessarily invalid;
nonempty truncated fields remain a separate malformed-boundary obligation.
No marker/creator blanket rejection was introduced. The onset-ramping distinction
remains explicitly outside this new bounded endpoint result.

Commands: the six TEMP variants remain explicit. `cargo test --offline -p nether-xm
-p nether-tracker -p nethercore-zx -p nether-it --lib -- --test-threads=1`;
`cargo build --offline -p nether-cli -p nethercore-zx`; existing flow and connected
historical/rollback helpers with the exact stage names above. Empty-container
checks: `python tools/tracker-debug/xm_empty_extension_probe.py
xm-empty-extension-standard-v2` and corresponding legacy stage with `--legacy`.
Logs reside in the configured Temp root; stage reports and generated artifacts
remain under `target/tracker-compatibility/`. Source pin/executable oracle
provenance remains unchanged. Overall goal still NOT ACCEPTED.

## XM pattern extents: fail-before/pass-after parser boundary repair

`parse_pattern` previously accepted a header shorter than its fixed fields, could
seek beyond the source, and decoded notes from the entire source rather than the
declared packed payload. It could consume following bytes as missing cells.
Original `pattern_reads_stay_inside_declared_header_and_payload` fails before
(`xm-pattern-bounds-before-v1.log`, exit101) and passes after validating header>=9,
validating declared extent and decoding through a bounded payload Cursor.
Padded headers and opaque bytes AFTER a complete payload remain accepted.
Only the already-supported XM1.04 parser is changed; no creator/extension exclusion.

`xm-pattern-bounds-after-v2.log`: serialized offline IT66+tracker54+XM71+ZX484
passed, one existing ignored ZX test. Build command:
`cargo build --offline -p nether-cli -p nethercore-zx` (exit0).
Fresh `xm-pattern-bounds-historical-v2` passes1344rows/2954notes with exact row
order, zero state mismatches/clipping and finite termination. Fresh
`xm-pattern-bounds-rollback-v2` passes3 handoff+cold checkpoints/64 subsequent
PCM and state frames including loop wrap. Same installed reference and six Temp
environment variables as above. v1 capture failed E0463 while linking stale native
rlibs after the parser rebuild; explicitly rebuilding BOTH tools recovered it.
That failed stage remains retained and is not acceptance evidence.
Read-only bounded parser/slide review `deleg_b440c808` is pending.
Overall broad XM/IT acceptance remains OPEN; this closes only the measured
pattern-extent boundary cases, not all malformed inputs or extension semantics.

## XM global/panning slides: integrated bounded repair, overall NOT ACCEPTED

Original generated Hxy/Pxy controls identified standard panning step/memory,
independent volume-column panning, legacy four-unit steps/shared column memory,
legacy cross-channel global-slide memory, and the legacy internal upper pan bound.
Repairs remain XM-gated in the existing effect/row/tick path. They reuse the
existing global-memory snapshot field and add one per-channel column-pan value.
Legacy behavior is selected by supplied-handle metadata, independently of mixing.
Rollback matrix now includes global/pan slides, both period modes and legacy mode.

Retained fresh source/cart/reference reports (each 120/120):
- `xm-slide-integrated-standard-v1`
- `xm-slide-integrated-legacy-v1`
- `xm-global-pan-cross-channel-v1`
- `xm-global-pan-cross-channel-legacy-v2` (v1:90/120 before shared legacy H memory)
- `xm-global-pan-legacy-ft2-v2` (v1:112/120 before legacy upper-bound repair)
- `xm-slide-integrated-ft2-cross-v1`

Reproduce with `python tools/tracker-debug/xm_global_pan_slide_probe.py FRESH_STAGE
[--legacy] [--mix-mode 5] [--cross-channel]`. The probe reuses the existing
xm_offset_probe bootstrap and installed official player; it does not replace
packing/playback. It compares absolute stereo levels and clipping for 48 ticks,
both period modes, six main parameters and five column variants. DC samples and
the last six pre-boundary PCM samples isolate endpoint gain from waveform phase
and reference volume ramps; the 0.018 baseline-relative tolerance is unchanged.
The first sine/RMS measurement was phase-sensitive and is NOT acceptance evidence.
This endpoint test does NOT establish ramp-transient fidelity or all slide interactions.

Final serialized offline suites: `xm-pan-slide-tests-v3.log`, exit0,
IT66 + tracker54 + XM70 + ZX484 passed, one existing ignored runtime test.
Fresh current-build `xm-slide-integrated-historical-v1 --require-pass`:1344 rows,
2954 notes, zero state mismatches, exact row order, finite termination, zero clipping,
peak0.95074462890625. `xm-slide-integrated-rollback-v1`: three handoff+cold
checkpoints,64 subsequent PCM/state frames each, including actual loop wrap.
Commands: Temp/xm-connected-capture.py STAGE --require-pass; then
Temp/xm-historical-rollback.py ROLLBACK_STAGE CAPTURE_STAGE.
All six TEMP/TMP/TMPDIR/temp/tmp/tmpdir explicitly use the authorized Temp path;
no downloads, source-music edits, SpecCade changes, commits or deletions.
Executable SHA256:9d809056e40e3d004b1ab9275081ee8ddb9d1874999369189d0ba172e7c3131c.
Source historical SHA256 remains5d0ff2a44bc0ff7c0eaad9a91da605f37f1d323e7af241b1521cc279dd3d1f9a.
No reference implementation/code/tables were used for these measured repairs.
Broader XM malformed/extension/corpus scope, final source review and equivalent IT
validation remain open. The empty recognized-container reviewer concern is not yet
adjudicated; do not assume an empty field list is malformed without establishing its contract.

## XM cross-channel flow: repaired bounded scope, overall still NOT ACCEPTED

Fresh original-source playback exposed XM Bxx/Dxx channel precedence, competing E6x loops, pattern-boundary loop carry and EEx/E6x interactions. Runtime repairs use the existing effect/row pipeline; IT branches remain isolated. Standard completed loops carry their marker into the next natural pattern; a marker WITHOUT an actual loop does not (boundary holdout caught and repaired that overbroad first implementation). Legacy loops serialize the active channel owner, without that carry. A delayed loop advances its target by one row. Later XM Bxx clears an earlier Dxx destination.

- `xm-cross-channel-flow-v2/report.json`: 36/36 standard controls; `xm-cross-channel-flow-legacy-v2/report.json`: 36/36 legacy controls. Both period modes, audible per-row pitch identity, native order/row trace and finite termination against reference PCM.
- `xm-flow-boundary-holdout-v1`: 44/46; marker-only failed in both period modes. After moving carry activation from E60 to E6x, `xm-flow-boundary-holdout-v2` and `xm-flow-boundary-legacy-holdout-v2`: **46/46 each**. Adds marker-only, marker/break, marker/jump, final-row loop and longer delay controls.
- Existing pipeline reused in durable `tools/tracker-debug/xm_flow_probe.py NEW_STAGE [--legacy]`, rebuilding nether-cli/nethercore-zx and the retained renderer before pack/capture. Standard generated modules use space-padded FT2/sample markers; legacy variant uses NUL sample-name padding. Source modules, packed captures, traces, source SHA256, reference SHA256 and both WAVs retained per named stage.
- Probe correction: initial v1 reference `--subsong -1` also played unvisited subsongs after Bxx, producing spurious duration failures. Finite controls now explicitly select `--subsong 0`; actual per-row pitch discrepancies remained and motivated repairs. No reference sources, code, tables or fixtures copied.
- `xm_next_pattern_row` and `xm_loop_owner` are retained through engine snapshot, cache save/restore and reset; replay visit identity includes both. `xm_competing_flow_state_survives_snapshot_and_cold_replay` exercises both modes/period tables at sample clocks 10003/30017/60031 and compares 16000 subsequent stereo samples, channel state, flow fields and serialized tracker state. Snapshot, cold and deliberately warmed-cache reconstruction pass (`xm-flow-warm-cache-v1.log`, exit 0).
- Early new-test failures were fixture/integration errors: missing opening brace, comparing boxed/non-PartialEq channels, assuming module slot zero, and inheriting 137 BPM while starting state at 125 BPM. Corrected using supplied handle, existing debug-state comparison convention and matched 125 BPM inputs. `xm-flow-tests-v7.log`: serialized offline library suites **66 IT + 54 tracker + 70 XM + 484 ZX passed**, one existing ignored test. No test failure was labeled playback success.
- Fresh `xm-flow-historical-v2` and `xm-flow-rollback-v2` both exit 0: original historical XM 1344 rows/2954 notes, zero note mismatches or clipping, and three handoff/cold checkpoints with 64 subsequent PCM/state frames including actual loop boundary.
- Read-only review `deleg_6223f228` raised a coordinate-keyed row-cache traversal concern. Owner traced production callers: reconstruction clears cache and uses the private row seeker only at origin; arbitrary coordinate seeks currently appear in tests. The warmed-cache real sync regression above passes. This is not evidence of a present runtime failure, but non-origin cache-backed seeking must not be exposed as traversal-safe without additional design/validation.

Commands used all six `TEMP TMP TMPDIR temp tmp tmpdir=C:/Users/rdave/AppData/Local/Temp`, offline serialized Cargo tests, installed hash-verified official reference only. No source-music/SpecCade/license changes, commits, downloads or deletions. Broader XM effects/extension/corpus/malformed coverage and equivalent IT validation remain open; these bounded flow results do not establish universal compatibility.

## Legacy sample restart: connected repair (supersedes 0/105 below)

Original playback isolated E90 nonzero-tick retriggers, Rxy pre-increment timing,
new-note Rxy count, and empty-row 9xx memory. No reference source/code/tables read
or copied. Mixer choice alone cannot encode this: legacy sample form plus MMP5
retains legacy effects; standard form plus MMP4 selects legacy effects.
`XmModule::legacy_retrigger` is preserved independently in NCXM bit0x40 and
FormatFlags bit0x0200. Missing bits retain prior packed behavior. Supplied module
row dispatch initializes channel state; snapshot/cold reconstruction preserves it.
IT guards remain separate. No broad creator/trailing-data rejection was added.

Fresh reports under target/tracker-compatibility:
- xm-retrigger-mode-isolation-v1:2/8; v2:5/8; v3:8/8.
- xm-legacy-envelope-v2:105/105 (v1 was0/105).
- xm-legacy-offset-holdout-v1:600/660, exposing empty-row memory and new-note
  counter differences; v2:660/660 after repair, including all volume transforms,
  sample widths/tuning/maps, envelope/pan/retrigger phases and absolute gain.
- xm-standard-retrigger-preserved-v2:660/660. v1 was interrupted by the outer
  420-second tool deadline at273 cases; no live worker remained. Evidence retained.
- xm-legacy-explicit-ft2-holdout-v1:660/660 through the durable probe options.

Repro: `python tools/tracker-debug/xm_offset_probe.py FRESH_STAGE [VOLUME]
[--legacy] [--mix-mode 4|5]`. Every stage rebuilds existing tools and retains
source/cart/native/installed-official-reference artifacts with hash provenance.
No downloads, source music changes, gain compensation or threshold relaxation.
Temp isolation helpers are xm-retrigger-mode-isolation.py and
xm-retrigger-header-isolation.py; earlier legacy helpers remain retained.

Full serialized offline tests command:
`cargo test --offline -p nether-xm -p nether-tracker -p nethercore-zx -p nether-it --lib -- --test-threads=1`.
Latest log xm-legacy-retrigger-tests-v5.log: IT66, converter54, XM70, runtime483
passed;1 runtime ignored. Extended existing rollback matrix with seven legacy
scenarios in both period modes and signed tuning: arbitrary mid-tick snapshots,
repeated snapshot/cold replay compare16000 subsequent PCM pairs and full state.
Parser tests cover independent mix overrides, parse/pack/strip and absent old bits.
Supplied-handle and IT-isolation tests assert the new channel flag after row
processing (an initial premature assertion immediately after sync failed in v4;
corrected to observe actual dispatched state, not change playback).
All commands explicitly set TEMP TMP TMPDIR temp tmp tmpdir to
C:/Users/rdave/AppData/Local/Temp; run offline and serialized.

Fresh xm-legacy-restart-historical-v1 --require-pass:1344 rows,2954 note events,
zero mismatches/clipping, exact row sequence and finite termination. Fresh
xm-legacy-restart-rollback-v1 (same capture base):3 checkpoints, handoff+cold,
64 subsequent frames each, full channel/state+PCM equality including loop.
A first historical command collided with an older stage name and correctly
refused to overwrite artifacts; unique legacy-restart names were then used.

Read-only review deleg_3b4940dc found no concrete defect in the bounded restart
metadata/runtime path; did not run playback. Owner independently inspected live
source and actual report cases. This does NOT close the overall inventory.

Additional original header-isolation-v1 passed12/14: effect phases passed, but
OpenMPT 1.32.00.00 space-padded header failed pan/gain (native/reference ratios
0.70467 envelope and0.86308 pan64). Narrow measured marker now selects FT2 pan;
header-isolation-v2 passes14/14, with a parse/pack regression. This is evidence
for that exact header only, not an all-OpenMPT creator heuristic or corpus verdict.
Broader mode/header variants, simultaneous flow/effects and equivalent IT gates
remain OPEN. OVERALL GOAL: IN PROGRESS / NOT ACCEPTED.

## Legacy sample-header mode: connected repair, review pending

Independent external-player measurements from original authored sources:
`xm-sample-structure-measure-v2` (36 variants), `xm-sample-override-measure-v1`
(6 overrides), `xm-mixed-sample-measure-v1` (4 mixed-slot forms). No reference
implementation or creator heuristics were read/copied/adapted. FT2 space marker
plus a non-all-space sample name selects the measured legacy linear pan and
2/3 output scale; Milky/unknown controls retain their defaults. This is a bounded
behavioral inference, not a general creator-identity claim.

Baking preamp32 FAILED .APS96: replay-v2 passed45/46, native1152/384 versus
reference768/256. Added XmMixMode::Legacy, NCXM flag0x20 and unified flag0x0100;
default preamp remains48. .MMP4/5 replaces mode; .APS alone preserves legacy scale.
Supplied-handle mixer gates scaling off for IT and preserves fractional preamp97.
Old noflag artifacts remain Compatible; contradictory FT2+Legacy tags reject.
Parse/pack/strip, opaque-tail, old-artifact, supplied-handle/snapshot tests cover
this metadata, including4096 subsequent PCM comparisons and deliberate IT isolation.

VERIFIED `xm-sample-structure-replay-v3/report.json`:46/46 fresh packed-native
captures match retained hash-bound reference measurements. Initial replay-v1
failed a report-key lookup; v2 retains the genuine gain failure. Measurement-v1
selected an explicit-mode baseline and failed header validation, so only v2 is
default-mode evidence. Helpers are retained in authorized Temp with prefixes
xm-sample-structure-measure, xm-sample-override-measure, xm-mixed-sample-measure,
xm-sample-structure-replay (Python, named-stage argument).

Actual commands set all six variables with:
`for k in TEMP TMP TMPDIR temp tmp tmpdir; do export "$k=C:/Users/rdave/AppData/Local/Temp"; done;`
then serialized offline library tests across nether-it/nether-tracker/nether-xm/
nethercore-zx:66/54/68/483 passed,1 runtime ignored (`xm-legacy-integrated-tests.log`).
Fresh capture helper `xm-legacy-historical-v1 --require-pass` preserves1344 rows,
2954 events, zero note mismatch/clipping, finite order21 termination (native peak
0.95074462890625). Rollback helper `xm-legacy-rollback-v1 xm-legacy-historical-v1`
passes three64-frame handoff/cold PCM+state checkpoints including loop.
A premature spoken46-pass claim was corrected after missing-artifact detection;
complete commands were then rerun and actual reports/logs inspected. No reliance
on the earlier empty exit0. Historical report keys are musical_state/native_pcm.

Fresh `xm-legacy-pan-v1` passes35 pan captures across seven header/mode variants;
`xm-legacy-inventory-v1` passes25/25 existing groups. HOWEVER original legacy
pan-envelope/retrigger family now re-exercised with NUL sample names:
`python C:/Users/rdave/AppData/Local/Temp/xm-legacy-envelope-replay.py xm-legacy-envelope-v1`
passes0/105. Initial gain ratios approximately0.996 match within the1% gate, but
subsequent envelope/phase controls fail. The helper reuses xm_offset_probe with
only NUL sample names and pan-family filtering; no explicit mode masking. Do NOT
mark this broader legacy family accepted: gain repair reveals remaining playback
semantics/transition discrepancies. Retained source/native/reference WAVs and
report.json are the next concrete repair evidence.

Review deleg_fc70f607 reports no defect in the new legacy metadata chain, but
raises whether an empty STPM/XTPM marker must be rejected. This is not yet a
proven defect: zero-field containers versus truncated fields need adjudication;
do not broaden rejection of trailing data from that assertion alone. Earlier sample-padding defect is repaired
for measured scope; broader headers, flow/effect interactions, corpus and equivalent
IT acceptance remain OPEN. Overall NOT ACCEPTED. No commits/deletions/SpecCade edits.


## Independently measured tracker-header padding (current follow-up)

`xm-header-padding-before-v2` isolates a20-byte FT2 tracker field with three
NUL padding bytes. Fresh native/reference pan positions0/64/128/192/255:
reference [[768,0],[576,192],[384,384],[192,576],[3,765]], native
[[768,0],[665,384],[543,543],[384,665],[48,766]]. Text normalization incorrectly
collapsed that raw marker into the space-padded FT2 marker. Original measurement
only; no reference implementation/creator-detection code was copied or adapted.
`XmMixMode::from_tracker_name` now requires the measured exact space-padded
FastTracker marker for that branch. Unknown creators remain accepted, not rejected.
A parse→NCXM→parse and stripped-XM regression includes both padding variants and
opaque trailing bytes; existing explicit metadata and old-artifact tests remain.

Fresh `xm-header-padding-after-v1` passes30 pan captures across six modes:
FT2 NUL, FT2 spaces, MilkyTracker1.03.00, unversioned MilkyTracker, explicit
compatible/preamp96 and explicit FT2/preamp24. Tool:
`python C:/Users/rdave/AppData/Local/Temp/xm-header-padding-probe.py <stage> [mode]`.
The helper reuses existing xm_global_volume_probe/capture_case/compiler and the
installed hash-verified reference without downloads. `xm-header-build.log`
records offline nether-cli/nethercore-zx rebuild. All commands set all six
TEMP/TMP/TMPDIR/temp/tmp/tmpdir to C:/Users/rdave/AppData/Local/Temp.
Initial `before-v1` accidentally authored21 tracker bytes, failed packing, and
is NOT playback evidence; helper now asserts header length<=20.

Serialized offline library tests (`xm-header-tests.log`) pass IT66,
converter54, XM68, runtime483 (1 ignored). `xm-header-inventory-v1` passes25/25
existing probe groups. Fresh `xm-header-historical-v1 --require-pass` preserves
1344 rows/2954 note events, no note-state mismatch or clipping; fresh
`xm-header-rollback-v1 xm-header-historical-v1` passes all three64-frame
handoff/cold state+PCM checkpoints. No commits/deletion/SpecCade edits.

IMPORTANT: this does NOT resolve the earlier implicit envelope-fixture gap.
Inspection confirms those retained fixtures already had space-padded tracker
headers. A fresh orthogonal `FT2-sample-NUL` control in
`xm-sample-padding-before-v1` changes only the22 sample-name padding bytes:
reference [[512,0],[384,128],[256,256],[128,384],[2,510]], native
[[768,0],[665,384],[543,543],[384,665],[48,766]]. Thus sample-header structure can
also select different legacy mode AND preamp. This concrete defect remains OPEN;
no speculative all-empty-name classifier or global gain adjustment was applied.
The helper now includes this seventh default case, so an unfiltered rerun is
expected to fail until repaired. Retain failing original modules/WAVs unchanged.
Overall XM/IT remains IN PROGRESS / NOT ACCEPTED.


## Acceptance contract

**OVERALL GOAL: IN PROGRESS / NOT ACCEPTED.** The user's goal is comprehensive
XM/IT playback parity, including rollback—not completion of a selected probe set.
The earlier scoped-completion claim was too narrow and must not close this goal.
Retain the passing named probes and historical evidence below as regression
coverage, not proof of comprehensive parity. Resume the outstanding inventory;
repair newly demonstrated standard defects rather than excluding them.

**General compatibility/corpus status: NOT ACCEPTED.** The named original probes
and two historical arrangements do not establish every XM/IT behavior or a diverse
historical corpus. Untested cases are not passes; newly demonstrated standard
defects remain repair obligations, not automatic exclusions.

Outcome: an ordinary historical XM/IT imported into a game plays as authored,
without compensating music edits or silent standard-feature loss. Bit-identical
PCM is not required. Keep the existing engine and packing architecture.

1. Establish source-backed coverage for parsing/packing, instruments/sample maps,
   simultaneous columns, effect memory/order, timing/tuning, envelopes/panning,
   sample/sustain loops, NNA/DCT/DCA, filters, flow and compatibility flags.
   Missing standard behavior is a repair obligation, not an exclusion.
2. Use pinned reference players and established compatibility fixtures plus a
   diverse legally usable historical corpus. Record per-fixture provenance and
   rights; resolve player disagreements against original tracker semantics.
3. Repair root causes with before/after regressions. Preserve determinism,
   rollback reconstruction and audio-thread correctness; test interactions.
4. Rebuild and repack before real-mixer tests. Cover complete finite songs,
   deliberate loops, seeking and rollback. Reuse existing capture tooling.
5. Deliver repeatable checks and honest coverage, passing relevant tests and
   independent source-bound review. No known standard defect at completion;
   untested behavior is not a pass, nor is a corpus universal mathematical proof.

Authorized: necessary original Nethercore source/tests/docs, public reference reading,
already-authorized local fixtures/tools and local builds, subject to the hard licensing boundary below. No engine replacement, commits, pushes, releases, destructive
cleanup, unrelated refactoring, or SpecCade/music-asset modifications. Preserve
pre-existing dirty files/worktrees and the IT panning repair. Continue in small
complete slices through recoverable failures; stop only at acceptance, user
pause, or a genuine unavailable-information/authorization/environment blocker.
Any necessary listening judgment is separate from technical acceptance.

## Instrument auto-vibrato follow-up (scope still open)

Retained `xm-auto-vibrato-baseline-v1/report.json`: 201/210 pass, nine
new instrument auto-vibrato failures. Owner traced instrument conversion,
note initialization, row reset, nonzero ticks and render clock. The XM path
incorrectly reused effect waveform numbering, phase scaling, depth and a
sweep denominator256 times too large, and omitted tick zero. An original
XM-only per-tick helper now uses the authored measurements' phase coordinate,
source-period increments and instrument waveforms, preserving the IT branch.
Row reset restores base pitch before modulation. No reference code/table copied.

`xm-auto-vibrato-repair-v1` passes208/210; the falling-ramp wrap endpoint
remained wrong. After correcting that discrete endpoint, fresh disjoint
rate13/depth11/sweep7 waveform holdouts plus all earlier controls pass218/218
in `xm-auto-vibrato-holdout-v1`. Commands:
`python tools/tracker-debug/xm_effect_probe.py STAGE` (fresh stages; rebuilds).
All commands retain the six-variable Temp environment and installed verified
reference; original reports/WAVs are preserved. Gates were not relaxed.

Expanded existing rollback scenarios27–30 cover all instrument waveforms with
active effect vibrato, both period modes and signed tuning; repeated snapshot
and cold reconstruction compare16000 subsequent PCM pairs and channel/state.
`cargo test --offline -p nethercore-zx -p nether-tracker -p nether-xm -p nether-it --lib -- --test-threads=1`
passes (log `C:/Users/rdave/AppData/Local/Temp/xm-auto-integrated-tests.log`).

NOT accepted as complete auto-vibrato parity: release/sweep behavior, instrument
selection/retrigger, pattern-delay first ticks and wider simultaneous effects
need fresh reference controls; historical playback/rollback must be rerun after
this change. Final independent review and comprehensive XM then IT remain open.

Follow-up original interaction controls exposed4 failures in230 cases:
release during sweep and standalone selection (`xm-auto-interactions-v1`).
The original implementation now suppresses unfinished sweep on release and
restarts selection phase/sweep without erasing active depth. The previous
unit assertion expecting depth0 was contradicted by fresh audible playback;
it now asserts retained depth5 plus reset sweep and phase. v2 passes230/230.
A stronger square/rate32/depth15 pattern-delay holdout exposed omitted delayed-row
tick0 advancement (`xm-auto-delay-holdout-v1`,2 failures); the render preparation
path now advances instrument modulation on those repeated first ticks.
`xm-auto-delay-holdout-v2` passes230/230 with unchanged gates.

Rollback scenarios27–33 now exercise instrument waveforms, selection, pattern
delay and release. The initial extended test incorrectly expected half volume
after selection reloads full sample default; that scenario-specific expectation
was corrected without changing playback. Full offline serialized rerun passes
IT66/converter54/XM67/runtime483,1 ignored; log `xm-auto-interaction-tests-v2.log`.
Fresh `xm-auto-historical-v2` passes1344 rows/2954 note events, zero mismatches,
zero clipping and finite end. `xm-auto-rollback-v2` passes three handoff+cold
checkpoints with64 subsequent frames each. Both commands use the established
Temp scripts/6-variable environment (capture with `--require-pass`).
Independent bounded review `deleg_c8995a85` is pending. Wider combinations,
source-mode/corpus and equivalent IT gates remain open; no overall acceptance.

Latest combined-delay follow-up: `xm-auto-combined-delay-v1` passes232/234;
active effect vibrato plus instrument vibrato diverges at repeated-row tick0.
Owner repaired that shared repeated-tick path to evaluate and advance effect
vibrato before instrument modulation, including the no-auto-vibrato sibling.
`xm-auto-combined-delay-v2` passes236/236, adding that zero-depth sibling and
release-after-full-sweep. No comparison tolerances changed.

Read-only reviewer `deleg_c8995a85` called suppressed unfinished sweep on release
a deadlock and proposed retaining modulation. That suggestion is NOT applied:
it conflicts with the measured `auto-release` reference ticks12–47, which stay
at neutral pitch after early release. `auto-release-full` separately verifies
continued modulation after completed sweep. Static intuition does not override
authored external playback. This review predates the combined-delay repair;
its quality/source limitations must not be treated as acceptance of that repair.

Owner corrected rollback release encoding from raw XM97 to unified
`TrackerNote::NOTE_OFF` (255); the former accidentally exercised a pitched note.
Rollback scenarios27–34 now include actual release and combined delayed-row
modulation. Full serialized offline libraries pass IT66/converter54/XM67/
runtime483 (1 ignored), log `xm-auto-combined-tests.log`.
Latest historical source/cart playback `xm-auto-historical-v3` and handoff/cold
subsequent-PCM replay `xm-auto-rollback-v3` pass, with zero clipping and all
musical assertions preserved. Same commands/Temp environment as above, fresh
stage names; no commits, cleanup, licensing changes or SpecCade edits.
Overall comprehensive XM/IT acceptance remains OPEN. Source-mode classification,
broader flow/effect combinations, corpus and equivalent IT coverage remain due.

## Independently measured E43/E73 random waveform

`xm-vibrato-random-v1` failed2 random cases (176 existing controls pass).
Native previously used an unrelated position hash. Fresh self-authored reference
measurements `xm-random-independent-measure-v1` use eight odd rates and depth15,
covering every64 waveform phase. No reference waveform table was read/copied.
Authorized independently measured runtime values are reconstructed from observed
pitch ratios: delta=-768*log2(hz/neutral), rounded to an integer; the authored
depth15 constrains each integer waveform magnitude. The midpoint of the feasible
interval has at most1 waveform-unit uncertainty. All observations and the exact
formula are retained in `inferred-waveform.json` in that stage; maximum repeated
phase spread in measured period delta was below0.03. This is inferred runtime
measurement data, not borrowed reference source/tables/tests/assets.

`xm-random-measured-replay-v1` passes; new even rates2/8, depths7/5 and notes53/61
holdouts (`xm-random-measured-holdout-v1`) pass198/198 total, with worst random
pitch error0.004560 (gate0.006). `xm-tremolo-random-v1` then fails2 E73 controls;
reusing the independently measured XM waveform in tremolo passes200/200 in
`xm-tremolo-random-v2`. These are not claims of bit-exact random-wave equivalence.
IT continues to use its existing waveform path; no reference dependencies added.

Original repeated snapshot/cold PCM replay now includes random vibrato/tremolo
(scenarios25/26), both period modes and signed tuning. Full offline serialized
library tests pass IT66/converter54/XM67/runtime483, runtime1 ignored.
Logs: `C:/Users/rdave/AppData/Local/Temp/xm-random-integrated-tests.log`.
Fresh `xm-random-historical-v1` playback and `xm-random-rollback-v1` handoff/cold
replay pass. Commands use the established six-variable environment:
`cargo test --offline -p nethercore-zx -p nether-tracker -p nether-xm -p nether-it --lib -- --test-threads=1`,
`python C:/Users/rdave/AppData/Local/Temp/xm-connected-capture.py xm-random-historical-v1`,
`python C:/Users/rdave/AppData/Local/Temp/xm-historical-rollback.py xm-random-rollback-v1 xm-random-historical-v1`.

Fresh read-only review of the pre-random176-control waveform source found no
scoped defect (deleg_5f878dcd); it does not review the subsequently measured random
addition. Owner inspected measurement consistency, both callers, IT isolation,
holdouts and subsequent-PCM replay directly. Broader simultaneous effects,
auto-vibrato, flow, implicit/legacy modes and equivalent IT acceptance remain open.

## Waveform follow-up: measured ramp, phase and silence repairs

Fresh `xm-tremolo-waveforms-v1` demonstrated ramp direction error. Original
ramp measurements establish a signed rising half-cycle; retained negative-half
vibrato reverses its slope. `xm-tremolo-phase-sign-v1` isolated that interaction
(164/166); original implementation in `xm_tremolo_delta` plus both tick callers
passes `xm-tremolo-phase-sign-v2` (166/166). Square, sine speed/depth holdouts,
new-note reset/no-reset and both period modes pass for named controls.
`xm-tremolo-silence-v1` demonstrated tremolo resurrecting zero base volume
(166/168). Mixer now preserves silence and gates XM modulation by the supplied
module's IT format flag; `xm-tremolo-silence-v2` passes168/168.

`xm-vibrato-waveforms-v1` failed both E41 ramp cases (174/176); v2 isolated the
negative-half slope. An original measured ramp branch in `xm_vibrato_period`
now passes `xm-vibrato-waveforms-v3`:176/176, including E41/E42, E40/E44 reset
and no-reset controls in both period modes, plus all previous effect/volume/delay
controls. Commands: `python tools/tracker-debug/xm_effect_probe.py STAGE`, stages
above under `target/tracker-compatibility/`; logs in Temp/STAGE.log.

The amplitude estimator was improved after square transitions exposed residual
reference tick ramps: for stationary smooth carriers it fits sine/cosine plus
linear amplitude slope over12–19.8ms, extrapolates to the20ms tick endpoint,
and rejects residual >1.5% of neutral RMS. Native volume uses whole-cycle RMS.
This supersedes the shorter19–19.9ms fit described below, without changing the
1% neutral-gain,2.5% normalized-volume,0.6% pitch gates. Ramp-shape parity is
still NOT accepted; no thresholds were relaxed. Retained failed stages preserve
both actual engine defects and diagnosed measurement failures separately.

Rollback test scenarios18–24 cover sine/square/ramp, retained vibrato phase,
zero-volume silence, and E41/E42. Both period modes, signed tuning, snapshots,
cold replay and repeated16000 subsequent PCM-pair/state comparisons pass.
`cargo test --offline -p nethercore-zx -p nether-tracker -p nether-xm -p nether-it
--lib -- --test-threads=1` passed in `xm-waveform-integrated-tests.log`.
Fresh historical playback `xm-waveform-historical-v1` and subsequent-frame
handoff+cold rollback `xm-waveform-rollback-v1` also passed after the latest
runtime change. All commands use the six required Temp variables, offline
serialized builds and the existing installed/hash-verified reference pipeline.
Source-bound independent review is pending. Random waveforms, auto-vibrato,
simultaneous live modulation, broad flow/extension/corpus and equivalent IT
acceptance remain OPEN; these scoped controls do not close the overall goal.

## Tremolo and delayed-slide review adjudication

Original sine `7xy` playback exposed cumulative corruption of channel volume.
XM now keeps modulation separate from authored volume and applies it at mixing;
row-zero re-evaluates modulation without advancing phase, nonzero ticks advance
phase, and empty rows clear modulation. IT retains its existing path. The sine
uses Nethercore's independently constructed mathematical waveform, not copied
reference code/tables. Calibration used original734 observations; new753/727
controls are holdouts. Ramp/square/no-reset interactions remain open.

`python tools/tracker-debug/xm_effect_probe.py xm-tremolo-holdout-v1` passed124/124.
Fresh `xm-tremolo-historical-v1/report.json` and `xm-tremolo-rollback-v1.log`
preserve full historical playback and three handoff+cold checkpoints,64 subsequent
frames each. `cargo test --offline -p nethercore-zx -p nether-tracker -p nether-xm
-p nether-it --lib -- --test-threads=1` passed483 runtime (1 ignored),54 converter,
67 XM,66 IT; log `C:/Users/rdave/AppData/Local/Temp/xm-tremolo-integrated-tests.log`.
Expanded existing replay test includes active tremolo plus repeated snapshots/cold
replay and16000 subsequent PCM pairs/state, preserving authored channel volume.

Review raised a possible ED3 + volume-column slide starting before the delayed
note. This was a lead, NOT an accepted defect: first tick means tick0, not tick1.
Fresh32 controls cover ED1/3/5/6, volume slides up/down, with/without instrument,
both period modes. `... xm-delay-volume-adjudication-v1` failed32 amplitude gates
because mid-tick RMS includes reference gain ramps, not because slides start late.
`... xm-delay-volume-adjudication-v2` passes156/156 total, including all32 delayed
controls WITHOUT runtime timing changes. The reference slides at ticks1/2 before
ED3, resets volume on the instrument trigger, then resumes sliding; this agrees
with the existing explicit native timing regression. The suggested deferral was
rejected based on actual playback. No source implementation was borrowed.

Measurement scope: native whole-cycle RMS; for the stationary smooth16-bit
sine tremolo/delay carriers, reference amplitude is fit at19–19.9ms of each20ms
tick to distinguish settled volume from reference intra-tick ramps. Ordinary
controls retain whole-cycle RMS. Baseline absolute-gain tolerance1%, normalized
volume2.5%, pitch0.6%, gate state and finite-end checks are unchanged. This does
NOT accept ramp-shape/transition PCM parity. Earlier diagnostic stages and all
raw WAVs/carts remain retained. Reference executable SHA256:
`9d809056e40e3d004b1ab9275081ee8ddb9d1874999369189d0ba172e7c3131c`;
source pin remains `f83cedb0cd5446e4dfaa83ac97e3087107e26767` (separate identity).
Every command uses all six TEMP/TMP/TMPDIR/temp/tmp/tmpdir set to
`C:/Users/rdave/AppData/Local/Temp`, fresh stages, rebuilt CLI/runtime, no downloads.
Overall comprehensive XM then IT acceptance remains OPEN.

## Remaining standard-behavior inventory (not acceptance)

The existing25-group XM driver was enumerated from actual
`xm_global_volume_probe` calls in `tools/tracker-debug/compatibility.py`:
its flags cover pan law; invalid/standalone selection and K00; Gxx;
cut/keyoff/noteoff/fade; delay/porta/empty-delay/slide combinations; sample
volume/pan/porta; zero-slide/zero-pan; high pattern indices; envelope
escape/pan-freeze/set-position; sample loop widths; multi-sample/empty slots.
Those25 groups are not an exhaustive effect-command inventory.

The current separate effect probe passes82/82 named pitch/tremor/interaction
controls (`xm-offset-porta-preserve-v2`); the independently measured repair
supersedes the historical43/44 glissando failure. The offset probe additionally
passes50/50 controls (`xm-offset-porta-memory-v2`). These are bounded test sets,
not denominators for a percentage of XM compatibility.

After the glissando repair, fresh reference coverage is still required for:

- E5x tuning override is now a demonstrated standard defect, not just a lead.
  `python tools/tracker-debug/xm_finetune_probe.py xm-e5-baseline-v1` rebuilt
  tools and ran24 fresh original source/cart/native/official-reference controls:
  8 pass/16 fail, including all6 no-override controls passing. E50/E58/E5F
  on a new note should override the source sample finetune; current native
  frequency remains at the baked sample tuning. Both period modes and source
  finetunes -64/0/+64 were measured. Existing0.006 threshold was not weakened.
  Retained report/WAVs/carts: `target/tracker-compatibility/xm-e5-baseline-v1/`.
  Follow-up connected repair now passes the bounded controls. Separate original
  sample finetune is preserved in TrackerSample/TrackerInstrument (IT defaults0);
  the existing combined transpose+tuning cannot recover the two independently.
  NCXM format is unchanged: already-preserved source fields supply this runtime
  metadata. E5x decodes to signed finetune, applies only the difference from baked
  tuning during XM note initialization, and no-note E5x does not retune playback.
  `python tools/tracker-debug/xm_finetune_probe.py xm-e5-repair-v1` passes24/24.
  `... xm-e5-sequence-v1` passes24/24 with four measured rows per case: override,
  no-note E58, fresh note without instrument, fresh note with instrument.
  `... xm-e5-transposition-v1` passes24/24 with additional -12/0/+12 sample
  transposition controls, preserving the intended octave while overriding fine.
  Original8/24 baseline remains retained; assertions were not weakened.
  Expanded existing PCM/state snapshot+cold-replay test with E5x and both source
  fine signs passes in full serialized offline suites: converter54/XM67/IT66/
  runtime480 passed, runtime1 ignored. Logs retained under xm-e5-sequence-v1.
  E5x plus glissando, delayed notes/retrigger and broader interactions are not
  accepted by these results; new historical reruns/final owner review still follow
  integration of the independent glissando work.
- 9xx offset scaling after resampling, offset recall, sample width and end/loop
  boundaries; E9x/Rxy retrigger phase, volume transforms and parameter memory.
- 5xy/6xy combined pitch+volume commands, 7xy/E7x tremolo, E4x waveforms and
  reset/no-reset, instrument auto-vibrato plus effect vibrato.
- Bxx/Dxx/E6x/EEx cross-channel priority and combined flow, F00/tempo boundaries,
  Hxy/Pxy simultaneous columns and zero-parameter memory.
- Source tuning extremes, representative sample rates/stereo data, supported
  explicit extension modes and additional authorized historical corpus.

These are unaccepted coverage obligations, not assertions that every listed
path is broken. Existing native unit/rollback tests remain supporting evidence;
source/cart/reference playback and subsequent-PCM rollback must close each
newly tested scope before equivalent broad IT expansion is accepted.

## Combined XM volume columns: connected repair; tremolo next

Fresh canonical FT2 `xm-combined-volume-interactions-v1` passes102/106:
all four 5xy/6xy plus volume-column up-slide controls fail, because the main
column overwrites the volume-column slide. Independent per-row XM signed slide
state now executes before the main-column slide without sharing Axy memory.
Both normal and delayed dispatch use the same helper; IT retains its old route.
`xm-combined-volume-interactions-v2` passes106/106; expanded opposite-direction
and zero-column holdout `xm-combined-volume-holdout-v1` passes114/114. Includes
both period modes, up/down, both nibbles, Axy memory, 500/600 and simultaneous
columns. Exact driver: `python tools/tracker-debug/xm_effect_probe.py STAGE`.

Retained initial `xm-combined-volume-v1/v2` amplitude preflights failed for
16-sample-period input (native/reference neutral gain~0.986); this remains an
interpolation/amplitude observation, not a mixer correction or accepted high-band
gain result. Whole-cycle RMS removes short-window phase bias. Lowband64 lacked
enough crossings and failed; midband32 supplies measurable pitch and neutral
absolute gain within1%, retaining volume-error0.025 and pitch-error0.006 gates.
No runtime gain changed and no threshold was loosened. These are fresh original
fixtures, not edits to authorized historical sources.

Rollback matrix now18 scenarios: the added 5xy/6xy cases snapshot at sample11297
inside active simultaneous-column playback, assert separate slide state and
volume0.5, then compare16000 subsequent PCM pairs and full state on repeated
snapshot/cold reconstruction. Offline serialized library suite passes IT66,
converter54,XM67,runtime483 (1 ignored). Command:
`cargo test --offline -p nethercore-zx -p nether-tracker -p nether-xm -p nether-it --lib -- --test-threads=1`;
log `C:/Users/rdave/AppData/Local/Temp/xm-column-integrated-tests.log`.
Fresh `xm-column-historical-v1 --require-pass`, `xm-column-rollback-v1 xm-column-historical-v1`
and `xm-column-inventory-v1` via existing Temp helpers exit0. Every command uses
all six Temp variables explicitly set to C:/Users/rdave/AppData/Local/Temp.
Final evidence inspection and independent column-repair review still pending.

Next fresh `xm-tremolo-baseline-v1` retains114/120 passes: all six standard
7xy tremolo/memory/gap cases fail amplitude (maximum normalized error~1.077).
Native mutates base volume cumulatively; reference oscillates around the base.
This is a demonstrated repair obligation, NOT ACCEPTED. Legacy mode inference,
remaining flow/other effects/corpus and equivalent IT gates remain open.

## Canonical FT2 source preflight: supersedes zero-name generalizations

IMPORTANT: earlier zero-filled-sample-name probe passes do NOT establish standard
FT2 effect semantics. Retain their artifacts and historical results, but retract
the generalized claims that standard FT2 E90 repeats each nonzero tick or that
standard Rxy tests increment-before-check. No runtime creator heuristic was added.

The existing `compatibility.py` pan-law fixture already authors FT2 space-padded
empty sample names. Reusing that convention in the offset generator (all other
source fields retained, no .MMP overrides) independently changed the reference's
pan/gain AND effect behavior. `xm-ft2-padding-gain-v1` retained165/660 passes.
This proves that explicit .MMP matching alone was insufficient to establish effect
mode equality. Unknown/legacy/default-inference parity remains OPEN; the current
standard repair does not claim to reproduce the earlier ambiguous oracle mode.

Original standard-source measurements repaired three root paths:
- E90 restarts PCM/envelopes at row start only; no nonzero-tick repeats.
- Rxy checks the remembered counter before incrementing; an authored note seeds
  its first cycle separately. Zero-parameter duration/volume memory persists.
- Note-less9xx (like volume-column portamento) does not update offset memory.
These are XM-only changes; IT direct offset and retrigger arithmetic are retained.
R6 arithmetic, note-only envelope age, and E9/Rxy envelope distinction continue
passing under the canonical source mode. No pan-law or gain compensation changed.

`python tools/tracker-debug/xm_offset_probe.py xm-standard-ft2-retrigger-v1`
retained600/660 passes, exposing new-note Rxy phase and E90 envelope reset details;
`... xm-standard-ft2-retrigger-v2` passes660/660 after their repair. All sources
now use implicit FT2 headers plus the existing space-padding convention, not an
explicit .MMP override. The new neutral tick-zero absolute-gain check is within1%
for every case (observed native/reference0.9954288..0.9966267). Normalized envelope
and phase gates remain separate; this is still not whole-waveform parity.

Focused unit checks now require E90 not to restart on tick one, the measured
Rxy tick3 first retrigger on a note-less row, note-less offset memory isolation,
IT seeded-state isolation, and remembered state. Existing16-scenario rollback
still compares16000 subsequent PCM pairs and full channel/state across repeated
snapshot/cold replay, both period modes and signed tuning. Offline serialized
IT66/converter54/XM67/runtime483 pass; runtime1 ignored. Exact suite command:
`cargo test --offline -p nethercore-zx -p nether-tracker -p nether-xm -p nether-it --lib -- --test-threads=1`
(log `C:/Users/rdave/AppData/Local/Temp/xm-standard-ft2-integrated-tests.log`).

Fresh existing helpers pass `xm-standard-ft2-historical-v1 --require-pass`,
`xm-standard-ft2-rollback-v1 xm-standard-ft2-historical-v1`, and
`xm-standard-ft2-inventory-v1`:1344 ordered rows/2954 note events, zero musical
mismatches/clipping, finite end, three64-frame handoff/cold checkpoints including
actual loop, and25/25 inventory groups. `xm-standard-ft2-pitch-preserve-v1` passes82/82.
All commands set the six authorized Temp variables; no downloads, source-asset
edits, licensing changes, commits, deletion, or SpecCade changes. Governing skill
now requires oracle-mode and neutral absolute-gain preflight before effect repairs.
Overall remains IN PROGRESS / NOT ACCEPTED: legacy/ambiguous mode selection,
broader combined effects/flow/corpus and equivalent IT gates remain obligations.

## XM envelope restart semantics and mix-mode comparison gap

`python tools/tracker-debug/xm_offset_probe.py xm-retrigger-envelope-v1`
retains480/525 passes, with all45 new volume-envelope controls failing.
Measured source/cart/reference behavior: a note without an instrument restarts
PCM but preserves envelope age; E9x restarts envelopes while Rxy preserves them.
The live row-processing path now gates envelope reset on IT or an authored
instrument, and the XM E9 tick path resets volume/pan positions and marks that
voice to skip same-tick envelope advancement. IT dispatch and Rxy are unchanged.
`xm-retrigger-envelope-v2` passes525/525 without changing the amplitude threshold.

Further volume-only note controls and panning envelopes exposed a DIFFERENT
comparison problem: in `xm-retrigger-pan-envelope-v1` / `xm-pan-envelope-directions-v1`,
the installed reference uses linear pan for these implicit-header fixtures,
while native classifies them as FT2 equal-power. Reference no-envelope left
amplitudes at pan64/128/192 were proportional to1.5/1/0.5. Do not repair that
with an envelope-only gain adjustment, and do not count those implicit cases
as pan/gain passes. Creator/default-mode parity for this original fixture shape
is an OPEN obligation; no creator heuristic was copied or blanket exclusion added.

The105 pan-related controls now explicitly author .MMP=5 in both players to
isolate envelope behavior under matching declared FT2 mode; reports label
`explicit_mix`. `xm-envelope-explicit-ft2-v1` passes660/660 total controls,
including the preserved480,75 volume-envelope and105 explicit-mode pan controls.
No mixer pan-law change was made. Volume/keyoff/sustain/fade and reverse-loop
interactions beyond these authored cases remain unaccepted. Existing normalized
probe results never established absolute gain or implicit-header mode parity.

Rollback scenarios14/15 add volume+pan envelopes, note-only cells, E9/Rxy and
instrument-note resets. An explicit snapshot assertion requires envelope age8
rather than a note-only reset to3. Both period modes, signed tuning, audible
mapped handles, three snapshot/cold paths and16000 subsequent PCM pairs plus full
channel/state comparison pass. Full serialized offline runtime483 passed/1 ignored
(log `C:/Users/rdave/AppData/Local/Temp/xm-retrigger-envelope-rollback.log`), then
named `xm_pitch_modes_glissando_and_tremor_replay_subsequent_pcm` passes again
with the age assertions. Command: `cargo test --offline -p nethercore-zx --lib -- --test-threads=1`.

Fresh rebuilt `xm-envelope-historical-v1 --require-pass` and fresh-cart
`xm-envelope-rollback-v1 xm-envelope-historical-v1` via the existing capture and
rollback helpers pass1344 ordered rows/2954 notes, zero musical-state mismatches,
zero clipping, finite termination and all three64-frame handoff/cold checkpoints,
including the loop. `xm-envelope-inventory-v1` passes25/25 existing groups.
All commands explicitly set all six upper/lowercase temporary-directory variables
to the authorized Temp; reference executable hash is unchanged. Owner inspected
the live row reset block, E9/Rxy branches and envelope-advance exclusion path.
Overall remains IN PROGRESS / NOT ACCEPTED; retain the newly identified implicit
mode comparison gap in addition to the remaining effect/flow/corpus/IT inventory.

## XM Rxy volume transforms: measured R6 repair

Extended existing offset/retrigger probe with all remaining Rxy volume nibbles.
`python tools/tracker-debug/xm_offset_probe.py xm-retrigger-transforms-v1`
retains441/480 passes:15 R6 cases have wrong amplitude decay;24 R4/R5 cases
hit an undefined phase comparison between exact native silence and a reference
ramp tail under0.002 of the original amplitude. Phase comparison now excludes
only those near-silent windows; the amplitude gate remains active. New transform
amplitude tolerance is0.01 (existing controls retain0.12); audible phase still
requires cosine>=0.99. This metric refinement does not excuse the R6 mismatch.

Fresh self-authored measurements establish XM R6 scaling5/8, not IT2/3, with
quarter-volume precision: normalized initial volume0.5 produces80,50,31,19
in units1/256. Original channel helper now receives an explicit XM boolean from
its two callers; XM multiplicative transforms quantize to1/256, IT retains its
old arithmetic. No reference source, tables, tests or assets were copied.
`xm-retrigger-transforms-v2` passes480/480. Independent odd-start-volume holdout
`python tools/tracker-debug/xm_offset_probe.py xm-retrigger-volume31-holdout-v1 31`
also passes480/480, exercising fractional intermediate values and silence clamps.
Same installed reference executable SHA256 as prior section; no downloads.

New direct measured-sequence/IT-isolation assertion and R6→R00 rollback scenario13
pass. The latter retains a nonzero partial counter, both period modes, signed
tuning, mapped audible handles, repeated snapshot/cold16000 subsequent PCM pairs
and full channel/state comparisons. Full affected serialized offline library
suites pass IT66/converter54/XM67/runtime483, runtime1 ignored; log
`C:/Users/rdave/AppData/Local/Temp/xm-r6-integrated-tests.log`.
Command: `cargo test --offline -p nethercore-zx -p nether-tracker -p nether-xm -p nether-it --lib -- --test-threads=1`.
All six TEMP/TMP/TMPDIR upper/lowercase variables were explicitly set to the
authorized Temp in every build/test/capture command.

Fresh rebuilt source/cart historical `xm-r6-historical-v1 --require-pass` and
`xm-r6-rollback-v1 xm-r6-historical-v1` using the existing capture/rollback helpers
pass musical state/termination/clipping and three handoff+cold checkpoints with
64 subsequent audio frames each, including the song loop. Fresh pitch/tremor
`xm-r6-pitch-preserve-v1` remains82/82. Owner inspected both helper callers,
explicit format dispatch, shared transform body and cloned rollback state.
Runtime scope is these measured transforms at initial volumes32 and31 plus the
retained controls—not envelope/keyoff/reverse-loop interactions or all XM/IT.
Overall goal remains IN PROGRESS / NOT ACCEPTED.

## XM note-plus-retrigger continuation

`python tools/tracker-debug/xm_offset_probe.py xm-retrigger-newnote-v1` rebuilt
and passed270/270 connected controls, including45 new note+instrument cells with
E93/R03/R93 after an offset, across the existing three maps and five width/tuning
configurations. Per-tick phase, normalized amplitude timing and finite stop all
pass unchanged thresholds against the same installed hash-verified reference.
No runtime/converter/packed-format repair was necessary; historical runtime
acceptance from the preceding stage remains unchanged, not newly rerun.

Existing rollback matrix adds scenarios11/12: new note+instrument+R03/E93 after
an active partial retrigger cycle, speed5, both period modes, signed tuning and
mapped audible PCM. Snapshot/cold repeated resimulation compares16000 subsequent
PCM pairs and complete channel/state equality. Full serialized offline runtime
suite passes483,1 ignored (`C:/Users/rdave/AppData/Local/Temp/xm-retrigger-newnote-runtime.log`).
Command: `cargo test --offline -p nethercore-zx --lib -- --test-threads=1`, all six
TEMP/TMP/TMPDIR upper/lowercase variables explicitly set to the authorized Temp.
Only probe/test/checkpoint edits. Broader volume transforms, envelopes/keyoff,
loop interactions, combined effects/flow and comprehensive IT remain open.

## XM E90 / Rxy measured repair (bounded, not overall acceptance)

`python tools/tracker-debug/xm_offset_probe.py xm-offset-retrigger-v1` retained
150/195 passes and 45 failures: E93→E90, R03→R00 and R93→R00 after a note offset,
across the existing width/tuning/three-map configurations. Fresh self-authored
modules and installed official executable only; no reference source/code/tables
were consumed for this repair. Executable SHA256:
`9d809056e40e3d004b1ab9275081ee8ddb9d1874999369189d0ba172e7c3131c`;
this is separate from the authorized source pin documented below.

Original native repair: XM E90 uses an every-nonzero-tick interval; Rxy retains
its parameter nibbles and a counter advanced on the command row including tick
zero. E9x clears the Rxy volume mode. New channel fields clone with snapshots;
row reset deactivates effects without clearing R memory. IT retains its old
interval arithmetic; the shared retrigger-volume body is reused, not replaced.
No packed format or source-asset change.

`... xm-offset-retrigger-v2` passes195/195. The stronger fresh
`... xm-offset-retrigger-phase-v1` also passes195/195: the45 new controls additionally
require per-tick waveform cosine similarity >=0.99 during both retrigger rows,
besides the unchanged normalized amplitude-timing and finite-stop gates.
This is not all Rxy volume transforms, arbitrary speed, envelope restart,
keyoff/stopped-voice, loop-direction, or note+Rxy acceptance.

Verification commands (all six TEMP/TMP/TMPDIR upper/lowercase variables explicitly
set to `C:/Users/rdave/AppData/Local/Temp`, offline):
- `cargo test --offline -p nethercore-zx -p nether-tracker -p nether-xm -p nether-it --lib -- --test-threads=1`
  passes IT66/converter54/XM67/runtime483, runtime1 ignored; log
  `C:/Users/rdave/AppData/Local/Temp/xm-retrigger-integrated-tests-v2.log`.
  First test-edit run failed compilation on a missing brace; repaired, not waived.
- New focused zero/memory/tick-phase/IT-isolation assertion passes; existing rollback
  matrix adds E90 and R00 scenarios, two period modes, signed tuning, mapped handles,
  repeated snapshot/cold replay comparing16000 subsequent PCM pairs and full state.
- `cargo build --offline -p nether-cli -p nethercore-zx`, then
  `python C:/Users/rdave/AppData/Local/Temp/xm-connected-capture.py xm-retrigger-historical-v1 --require-pass`
  passes1344 ordered rows/2954 note events, zero mismatches/clipping, correct finite stop;
  native peak0.96160888671875. Report and source/cart/audio retained in that named stage.
- `python C:/Users/rdave/AppData/Local/Temp/xm-historical-rollback.py xm-retrigger-rollback-v1 xm-retrigger-historical-v1`
  passes three checkpoints,64 subsequent audio frames each, handoff+cold state/PCM,
  including the real loop. Helper now accepts an optional named fresh-cart stage.

Fresh `xm-retrigger-pitch-preserve-v1` and `xm-retrigger-inventory-v1` also exit0;
their named pitch/tremor and existing XM-driver reports remain retained.
Independent read-only review `deleg_87209fc5` raised a counter-reset claim and
phase/partial-cycle/IT-isolation proof gaps. Owner resolved rather than blindly
applying the proposed reset: fresh `xm-retrigger-review-speed5-v1` passes225/225,
including30 speed5/interval3 controls with consecutive R03/R00 and an intervening
empty row. The reference agrees with retaining the counter, so the proposed
row-reset change was rejected. Per-tick phase assertions cover these rows too.
The original195 controls remain passing, with no relaxed threshold.

Rollback now uses speed5 for R00/E90 and explicitly asserts a nonzero saved R
counter (1), then compares repeated snapshot/cold subsequent PCM and full state.
`cargo test --offline -p nethercore-zx --lib -- --test-threads=1` again passes483,
1 ignored (`C:/Users/rdave/AppData/Local/Temp/xm-retrigger-reviewed-runtime.log`).
The focused `xm_retrigger_zero_memory_and_tick_phase_preserve_it` also passes after
adding an IT Q-style retrigger with deliberately seeded XM memory/count/activity;
those XM fields stay unchanged in the captured snapshot. This is an IT-isolation
regression, not equivalent broad IT acceptance. Final owner inspection checked
row flags, note reset, both tick callers, shared volume body and cloned state.
Overall XM/IT inventory remains open; broader R transforms/note/envelope/loop
interactions, combined effects, flow and equivalent IT coverage still follow.

## Split-map sample switching controls

`python tools/tracker-debug/xm_offset_probe.py xm-offset-slot-switch-v1` passes
150/150 controls. The additional50 use a split note map: initial note37 selects
slot one; new note49 selects slot two with independent tuning and envelope timing.
This exercises switching via ordinary notes,9xx and900 memory; porta controls
retain the old voice until a following ordinary note. Existing100 controls remain.
This report measures sample-envelope timing, not universal pitch/gain parity.
Source/cart/reference artifacts and provenance are retained in the named stage.

Expanded existing rollback scenario8 uses two audible distinct PCM handles `[1,2]`,
the split map, a mid-tick snapshot before the switch, later ordinary-note switching,
both period modes and signed destination tuning. Snapshot and cold reconstruction
match16000 subsequent PCM pairs and state/channels, repeatedly. The focused command
`cargo test --offline -p nethercore-zx --lib xm_pitch_modes_glissando_and_tremor_replay_subsequent_pcm -- --test-threads=1`
passed1/1 after correcting a test-only nested row-index compile error. Both commands
used all six explicit temp variables; the connected probe rebuilt tools offline.
No runtime implementation change was required. Different-instrument changes,
retrigger/flow interactions and comprehensive XM/IT coverage remain outstanding.

## Mapped-slot offset coverage

Expanded the existing original offset probe, not the renderer, with an explicit
second-slot sample map. The unselected first slot has distinct envelope timing
and default tuning; all mapped notes select slot two, including its signed fine
and transposition metadata. No runtime change was needed.
`python tools/tracker-debug/xm_offset_probe.py xm-offset-mapped-v1` rebuilt offline,
packed fresh sources and passed100/100 controls (50 mapped,50 existing single-slot).
Artifact: `target/tracker-compatibility/xm-offset-mapped-v1/report.json`; the report
retains source/reference/renderer provenance and per-tick normalized amplitude.
This covers selection of a non-first mapped slot, NOT switching slots mid-song.

Expanded the existing arbitrary-mid-tick rollback regression with a mapped
second-slot module, supplied handles `[0,1]`, signed tuning, both period modes,
portamento/offset memory and an explicit non-silent-PCM assertion. Snapshot and
cold reconstruction match16000 subsequent sample pairs and final state/channels,
with repeated replay. Command:
`cargo test --offline -p nethercore-zx --lib xm_pitch_modes_glissando_and_tremor_replay_subsequent_pcm -- --test-threads=1`
passed1/1. Both commands used all six explicit temp variables. Only probe/test
files changed, preserving the preceding historical/runtime evidence.
Still outstanding: changing mapped slots/instruments, retrigger/flow combinations,
additional tracker variants and comprehensive IT expansion. Overall NOT ACCEPTED.

## Post-offset full historical preservation

Fresh offline rebuild and named capture after the final portamento/9xx guard:
`python C:/Users/rdave/AppData/Local/Temp/xm-connected-capture.py xm-offset-porta-historical-v1 --require-pass` exited0.
The inspected rollback helper was executed with its single base-stage literal
`xm-connected-after-review-v1` replaced in memory by `xm-offset-porta-historical-v1`,
and argv output stage `xm-offset-porta-historical-rollback-v1`; it exited0.
Both used all six explicit temp variables and preserved old artifacts.

Capture:1344 exact ordered rows,2954 note events,zero note-state mismatches,
zero clipping,finite termination21/0 with no wraps. PCM SHA256 unchanged:
`eec083fad9b1fbe51853f6917e111dfb6adb9e00592ed70c41d6af091b147cdc`.
Runtime library SHA256:
`de02a17c49d28795e8055822a484c4719e98128eec28eac6ad608c917310a310`.
Rollback: all three checkpoints match both handoff and cold reconstruction,
64 subsequent PCM/state frames each, including actual loop traversal (wraps1).
This closes the historical-rerun gap noted below, not the comprehensive inventory.
Source-bound owner inspection confirmed the guarded arm precedes offset memory
updates and does not return before common post-effect processing. IT dispatch
is unchanged. Broader mapped-sample/flow/retrigger coverage remains outstanding.

## XM offset / volume-column portamento interaction

Original offset probe expanded to include past-end offsets and simultaneous
volume-column Fxx + main-column9xx. `xm-offset-interactions-before-v1` failed
all five new portamento combinations (35/40 pass); past-end controls passed.
Owner traced `process_note_internal`: volume column executes before main column,
so the shared effect handler can use the current row's tone-portamento flag.
A seek-only guard passed40/40 in `xm-offset-interactions-after-v1`, but the added
next-note900 holdout exposed incorrect offset-memory updates (45/50 in
`xm-offset-porta-memory-v1`). That incomplete guard was replaced, not accepted.

Final original guarded match arm ignores XM9xx during tone portamento, including
its memory, while still executing common post-effect processing. IT remains
unchanged. The later900 control confirms that suppressed9xx does not leak memory.
`xm-offset-porta-memory-v2` passes50/50: past-end, zero-speed porta, nonzero porta,
following-row memory, previous controls,8/16-bit and signed tuning/transposition.
`xm-offset-porta-preserve-v2` passes all82 prior pitch controls.

Existing snapshot/cold-reconstruction PCM regression now includes the paired
volume-column-porta/offset case with a subsequent900, explicit zero-memory
assertion, both period modes and signed tuning. Full runtime suite after repair:
482 passed,0 failed,1 ignored. Logs in `xm-offset-porta-memory-v2/runtime.log`.
Exact commands: `python tools/tracker-debug/xm_offset_probe.py STAGE`;
`cargo test --offline -p nethercore-zx --lib -- --test-threads=1`;
`python tools/tracker-debug/xm_effect_probe.py xm-offset-porta-preserve-v2`.
All ran with the six explicit temp variables and offline rebuilt tools before
packing. Fresh source/cart/reference artifacts and hashes remain in each stage.
Historical capture from the preceding offset repair is retained, but was not
rerun after this additional guarded-arm change. Mapped instrument switches,
additional retrigger/delay combinations and tracker variants remain uncovered.
Overall XM/IT parity remains IN PROGRESS / NOT ACCEPTED.

## XM sample-offset timing repair

Fresh pre-offset-change standard inventory `xm-post-measure-inventory-v1`
passed all25 existing groups after an explicit offline CLI/runtime rebuild.
Original new `tools/tracker-debug/xm_offset_probe.py` then demonstrated two
previously uncovered defects in fresh packed playback: 9xx on a row with no
note incorrectly sought the active sample; new-note 9xx used a source position
as a packed/resampled position. `xm-offset-timing-before-v1` exited1, 2/4 pass.

Minimal shared tick0 repair in `engine/effects.rs`: keep 9xx memory on empty
rows but seek only on XM notes1..96; reconstruct the already-preserved original
combined tuning and reuse `ExtractedSample::calculate_sample_rate` plus
`nether_tracker::convert_loop_points` to translate the offset. No new metadata,
packed format or dependencies. IT's existing branch is explicitly unchanged.
Owner traced source/mapped tuning and the shared effect caller before editing.

Fresh source/cart/native/installed-reference stages:
- `xm-offset-timing-after-v1`: 4/4 pass.
- `xm-offset-width-tuning-v1`: 20/20 pass (8/16-bit, +/-64 finetune, +12 transpose).
- `xm-offset-memory-v1`: 30/30 pass, adding empty-row memory followed by900 and
  note-triggered offset memory followed by900. The probe compares normalized
  sample-envelope timing, not gain parity; raw WAVs and reference hash retained.
- `xm-offset-preserve-pitch-v1`: all82 previous pitch/effect controls still pass.
- `xm-offset-historical-v1`: full historical capture passes1344 rows/2954 events,
  zero note mismatches/clipping and finite termination.

Expanded the existing mid-tick rollback regression with offset-memory execution,
a4096-sample original waveform, both period modes and positive/negative tuning:
asserts memory at snapshot, compares16000 subsequent PCM samples and channel/state
through handoff, cold reconstruction and repeated replay. Full serialized offline
runtime suite after expansion:482 passed,0 failed,1 ignored.
Log:`xm-offset-memory-v1/runtime-final.log`. Commands use the six explicit temp
variables set to C:/Users/rdave/AppData/Local/Temp; each probe rebuilds before
packing. Invocation:`python tools/tracker-debug/xm_offset_probe.py NEW_STAGE`;
`cargo test --offline -p nethercore-zx --lib -- --test-threads=1`.

This closes only the tested offset timing/memory slice. Remaining offset inventory:
out-of-range offsets, instrument/sample-map switches, E5/delay/retrigger and
volume-column interactions, version/extension differences. Comprehensive XM/IT
parity remains IN PROGRESS / NOT ACCEPTED.

## Independently measured Amiga-period repair — fresh evidence

The measurement authorization below has now been exercised. This supersedes the
older four-glissando-failures checkpoint for the explicitly tested scope, not the
unfinished comprehensive XM/IT goal. Independent source review `deleg_a46107f8`
completed and found one concrete runtime-domain lower-bound defect. Owner
reproduced and repaired it as recorded below; overall compatibility is not accepted.

Original `tools/tracker-debug/xm_period_measure.py` authors 192 sine-tone cases
(12 notes, 16 signed finetunes), renders only through the installed hash-checked
player, and measures two disjoint steady-state windows. It reads no reference
source/tables. `xm-independent-period-measure-v1` and the independently rerun
`xm-independent-period-repeat-v1` passed the 0.0001 relative window-spread gate
(maximum observed 0.0000166455066968). The 96 nonnegative-finetune observations
infer integer C-3 period coordinates using the existing engine reference rate:
`round(8363 * 1712 / (measured_hz * 64))`, with residual below 0.15 period units.
The repeat's `inferred-periods.json` exactly matches all 96 runtime constants.
Raw frequencies, source hash, reference executable hash and settings remain in
the measurement reports. This is measurement provenance, not a legal opinion.

Original runtime changes in `utils.rs` use piecewise source-period interpolation
and its inverse; integer slide increments no longer accumulate f32 coordinate
round-trip drift. The observed positive-finetune octave transition is bounded by
the final measured subdivision of the lower octave, rather than assuming ideal
periodic equal-temperament boundaries. Source finetune is carried by the existing
channel initialization/mapped-sample/E5 paths and snapshot channel state.
Linear XM and IT paths retain their previous quantizer/slide selection.
No packed-format, dependencies, license, reference implementation, or asset changes.

Fresh retained stages (all under `target/tracker-compatibility`):
- `xm-measured-coordinate-v1`: 57/62; retained unsuccessful candidate evidence.
- `xm-measured-coordinate-v2`: 61/62 after integer source-counter recovery.
- `xm-measured-octave-boundary-v1`: 64/66; slow descending control isolated the
  remaining octave transition (reference stays on the upper note at source
  periods 856 and 860, changes by 864). No tolerance adjustment.
- `xm-measured-coordinate-v3`: 66/66 after the observed boundary repair.
- `xm-measured-holdout-v1`: 82/82, including new D/A/C/E starts, finetunes
  32/96/80/-32, different speeds/directions, matched continuous slides, and both
  period modes. These behavioral holdouts were not used to infer the constants.
- `xm-measured-e5-v1`: E5x connected probe exited 0, including retained sequence
  and transposition assertions; its console headline reports 24 primary controls.
- `xm-measured-historical-v1`: 1344 rows, 2954 events, zero note mismatches,
  exact row order, zero clipping, correct finite termination. Native PCM SHA256
  remains `eec083fad9b1fbe51853f6917e111dfb6adb9e00592ed70c41d6af091b147cdc`.
- `xm-measured-historical-rollback-v1`: all three checkpoints matched handoff and
  cold reconstruction for 64 subsequent PCM/state frames each, actual loop included.

Commands used the explicit six TEMP/TMP/TMPDIR/temp/tmp/tmpdir variables set to
`C:/Users/rdave/AppData/Local/Temp`, offline builds and serialized tests:
`python tools/tracker-debug/xm_period_measure.py NEW_STAGE`;
`python tools/tracker-debug/xm_effect_probe.py NEW_STAGE` (rebuilds before packing);
`python tools/tracker-debug/xm_finetune_probe.py xm-measured-e5-v1`;
`cargo test --offline -p nethercore-zx --lib -- --test-threads=1`;
`cargo test --offline -p nether-tracker -p nether-xm -p nether-it --lib -- --test-threads=1`.
Runtime: 481 passed, 0 failed, 1 ignored, including expanded subsequent-PCM rollback
and new integer-coordinate roundtrip/boundary regression. Libraries: IT 66,
converter 54, XM 67 passed. Logs: `xm-independent-period-measure-v1/*-tests.log`.
Historical commands reused the inspected connected-capture helper with
`xm-measured-historical-v1 --require-pass`; rollback reused the previously recorded
in-memory base-stage substitution, replacing `xm-connected-after-review-v1` with
`xm-measured-historical-v1`, output `xm-measured-historical-rollback-v1`.

Remaining: wider octave/finetune/version and interaction
coverage; comprehensive XM inventory and then equivalent IT expansion. The
calibration sweep is not a claim that all tracker-specific period behavior or
all possible transitions are covered.

## Source review lower-bound repair

Read-only review `deleg_a46107f8` found no snapshot metadata omission, but noted
that the inverse coordinate can legitimately become negative outside the runtime
playable domain. `slide_xm_period` previously returned that negative result,
causing `period_to_frequency` to return zero. Owner added the original
`xm_slide_lower_bound_stays_playable` regression and reproduced the failure:
`cargo test --offline -p nethercore-zx --lib xm_slide_lower_bound_stays_playable -- --test-threads=1`
exited101, one executed test failed at tuning -16512 with runtime period -11897.6.
Log: `xm-independent-period-measure-v1/bounds-before.log`.

Minimal repair: clamp the returned Amiga slide result to the existing runtime
floor1, matching the linear path's invariant, leaving the mathematical inverse
unclamped. Roundtrip assertions now distinguish representable coordinates from
intentional saturation. The boundary regression covers every signed-byte relative
note with five finetunes, repeated lower-bound slides, positive frequency and
vibrato result invariants. This is runtime-bound safety, NOT reference-player
parity proof for extreme out-of-range pitch or its effect interactions.

Fresh owner verification with all six explicit temp variables, offline/serialized:
- Runtime suite: 482 passed, 0 failed, 1 ignored; includes rollback checks.
  `xm-independent-period-measure-v1/bounds-after-tests.log`.
- Rebuilt `python tools/tracker-debug/xm_effect_probe.py xm-reviewed-bounds-v1`:
  82/82 controls pass. `bounds-playback.log` retained beside the test log.
- Inspected historical helper with `xm-reviewed-bounds-historical-v1 --require-pass`:
  1344 rows, 2954 events, zero mismatches/clipping, finite termination; PCM hash
  unchanged (`eec083fad9b1fbe51853f6917e111dfb6adb9e00592ed70c41d6af091b147cdc`).
- Historical rollback using the same documented in-memory base-stage substitution
  to `xm-reviewed-bounds-historical-v1`, output `xm-reviewed-bounds-rollback-v1`:
  three checkpoints, handoff+cold, 64 subsequent PCM/state frames, actual loop.
All post-repair commands exited0. Review provenance comment now points to the
repeat measurement report; stale authorization-status wording is corrected.
No review worker remains active. Wider coverage gaps remain open.

## Independent measurement authorization

User explicitly approved: "Yes independently measured runtime values are allowed".
This permits runtime calibration values derived from fresh self-authored modules
rendered by the installed behavioral oracle. It does not permit copying or
slightly modifying reference implementation code, tables, tests or assets.
Keep the original measurement generator, executable hash/settings, measured
precision and independent holdout playback checks with any resulting values.
The prior authorization blocker is resolved. Later measured-repair evidence above
supersedes the then-unresolved glissando controls, not the overall parity goal.

## Paired continuous-slide discrimination

Re-read contracts and inspected the dirty tree (`git diff --check` exited0).
Fresh rebuilt `xm-current-gliss-baseline-v1` reproduced the existing four
Amiga glissando failures. Added five matched no-glissando controls in both
period modes to the existing `tools/tracker-debug/xm_effect_probe.py`, leaving
source sample bytes, slide speeds, direction and tuning identical to each
paired glissando fixture. No runtime or tolerance change was made.

`python tools/tracker-debug/xm_effect_probe.py xm-paired-slide-baseline-v1`
ran with all six temp variables explicitly set to C:/Users/rdave/AppData/Local/Temp,
offline rebuilt tools, fresh packing and the installed hash-checked reference.
It exited1: 58/62 pass; all ten new continuous-slide controls pass the existing
0.006 pitch gate. The same four Amiga glissando cases fail. This separates the
large discrete snap errors from continuous-slide pitch within the current
measurement tolerance; it does NOT prove exact accumulator/boundary arithmetic.
Artifacts and executable provenance: target/tracker-compatibility/xm-paired-slide-baseline-v1/report.json.
Overall remains IN PROGRESS / NOT ACCEPTED. No runtime repair is claimed here.

## E5x integrated historical verification

After removing both rejected glissando candidates, rebuilt offline CLI/runtime
with all six explicit temp variables. Commands from repository root:

```sh
python C:/Users/rdave/AppData/Local/Temp/xm-connected-capture.py xm-e5-integrated-historical-v1 --require-pass
python -c 'from pathlib import Path; import sys; p=Path("C:/Users/rdave/AppData/Local/Temp/xm-historical-rollback.py"); s=p.read_text(); assert s.count("xm-connected-after-review-v1")==1; s=s.replace("xm-connected-after-review-v1","xm-e5-integrated-historical-v1"); sys.argv=[str(p),"xm-e5-integrated-rollback-v1"]; exec(compile(s,str(p),"exec"))'
```

Both exited0. Fresh source/cart/native/reference capture retains1344 rows,
2954 events, zero note-state mismatches, zero clipping and correct finite21/0
termination. Native PCM SHA256 equals the prior accepted historical artifact:
`eec083fad9b1fbe51853f6917e111dfb6adb9e00592ed70c41d6af091b147cdc`.
Runtime SHA256:`ac4a3c28a826451b94e1b32682d2ec8d7f2dbbd74c29353e8338f08e5674d470`.
Rollback log confirms all three checkpoints, handoff+cold,64 subsequent PCM/state
frames each, actual loop traversal (wraps1). Reports/logs are in the named stages
under target/tracker-compatibility. These passes preserve the historical slice;
the four expanded Amiga glissando failures and broader inventory remain open.

## Glissando worker recovery and owner experiment

Worker `deleg_f84c9b11` finished without acceptance. Owner removed its failed
quantizer and process-global debug print, retaining the expanded52-case probe.
`python tools/tracker-debug/xm_effect_probe.py xm-gliss-owner-recovery-v1`
rebuilt tools and produced48/52 passing. Four Amiga failures remain:
`gliss-fine`, `gliss-fine-pos-fast`, `gliss-fine-neg-up`, `gliss-fine-neg-down`.

Owner tried an original analytic integer-boundary quantizer in
`xm-gliss-native-boundary-v1`. It repaired the original tick14 transition but
traded other failures: still48/52. This candidate and its temporary channel
metadata were removed rather than accepted. Both reports/artifacts are retained.
No glissando worker remains active. The previous quantizer and E5x repair remain.
Fresh serialized offline runtime suite after removal passed480/failed0/ignored1,
including expanded E5x/glissando subsequent-PCM and state rollback checks.
Comprehensive parity remains unaccepted; no tolerance was relaxed.

## Latest XM interaction recovery checkpoint

**IN PROGRESS / NOT ACCEPTED.** This supersedes older scoped-completion wording,
not the retained earlier passing evidence. The current outstanding XM defect is
finetuned Amiga glissando transition timing; broader XM/IT inventory is still open.

- Re-read AGENTS/CLAUDE, acceptance and prior checkpoint; inspected the extensive
  dirty tree and passed `git diff --check` before edits. No unrelated changes removed.
- `python tools/tracker-debug/xm_effect_probe.py xm-parity-review-interactions-v5`
  rebuilt CLI/runtime and the existing compatibility renderer, generated fresh
  source/cart/native/official-reference artifacts, and correctly exited1:
  **43/44 cases pass**. Added tone-portamento followed by vibrato and same-row
  fine-slide plus volume-column vibrato in both period modes; all four pass.
  Prior `xm-parity-interactions-v4` had39/40 passing with the same remaining case.
- `0-gliss-fine`, absolute tick14 (row2/tick2): native603.28Hz vs reference569.97Hz,
  maximum relative pitch error0.0584435976479587. No gate mismatch. Do not relax
  the existing0.006 pitch threshold or mistake finite termination for parity.
  The current logarithmic gliss quantizer lacks the reference's discrete Amiga
  period boundaries. Cached pinned caller/period flow was read as behavioral
  evidence only; no table/code copied and no speculative rounding patch applied.
- Delayed reviewer `deleg_5160917a` inspected an earlier accumulator revision.
  Current tick processing records unmodulated slide output before vibrato;
  the four new actual playback controls pass. This adjudicates those bounded
  combinations, not every simultaneous arpeggio/vibrato or auto-vibrato path.
- Extended the existing `xm_pitch_modes_glissando_and_tremor_replay_subsequent_pcm`
  regression with porta→vibrato and fine-slide+volume-vibrato scenarios in both
  period modes. Snapshot at sample7297; each snapshot/cold/snapshot replay compares
 16000 subsequent stereo samples, complete TrackerState bytes and channel state
  against uninterrupted playback, including traversal of the short song loop.
- Extended the same rollback regression again with `(source tuning,target note,
  porta speed)` pairs `(-64,37,3)` and `(64,61,5)`: both tuning signs and slide
  directions, both period modes and all four interaction scenarios. Fresh
  `cargo test --offline -p nethercore-zx --lib xm_pitch_modes_glissando_and_tremor -- --test-threads=1`
  under the six explicit temp variables passed1/failed0 (480 filtered out).
  This verifies replay determinism of the exercised revision, not reference
  correctness of the still-unaccepted glissando repair. Implementation worker
  `deleg_f84c9b11` is active; its future edits require owner verification.
- Serialized offline library suites with all six TEMP/TMP/TMPDIR case variants
  explicitly `C:/Users/rdave/AppData/Local/Temp`: converter54, XM67, IT66,
  runtime480 passed/runtime1 ignored. Exact Cargo shape:
  `cargo test --offline -p PACKAGE --lib -- --test-threads=1`.
  Logs in `target/tracker-compatibility/xm-parity-review-interactions-v5/`:
  `{nether-tracker,nether-xm,nether-it}-tests.log` and
  `nethercore-zx-tests-repaired.log`. Initial runtime compile failed on a new test's
  mistaken enum spelling; corrected to existing `FinePortaUp`, then full suite passed.
- Reference SHA256 is retained in report.json and verified by filter_reference:
  `9d809056e40e3d004b1ab9275081ee8ddb9d1874999369189d0ba172e7c3131c`.
  Source pin remains separately `f83cedb0cd5446e4dfaa83ac97e3087107e26767`.
  Existing-player presence is required before filter_reference; no download used.
- Fresh historical capture now passes for this pitch revision:
  `python C:/Users/rdave/AppData/Local/Temp/xm-connected-capture.py xm-current-pitch-historical-v1 --require-pass`
  after offline CLI/runtime rebuild with the six explicit temp variables.
  `xm-current-pitch-historical-v1/report.json`:1344 rows,2954 note events,
  zero held-note mismatches, exact row order, finite end21/0/no wraps,
  zero clipping; native peak0.96160888671875/reference0.960296630859375.
  Runtime SHA256:80e00da36431a502bb7e6485d10f108d3ebab9f226604d99a10c2368ebca93a1.
- Historical rollback execution initially hit an approval timeout; it was not
  bypassed. After explicit user approval, the following exact command exited0:
  ```sh
  python -c 'from pathlib import Path; import sys; p=Path("C:/Users/rdave/AppData/Local/Temp/xm-historical-rollback.py"); s=p.read_text(); assert s.count("xm-connected-after-review-v1")==1; s=s.replace("xm-connected-after-review-v1","xm-current-pitch-historical-v1"); sys.argv=[str(p),"xm-current-pitch-historical-rollback-v1"]; exec(compile(s,str(p),"exec"))'
  ```
  The inspected script explicitly sets all six temp variables and offline mode.
  It reuses the newly packed historical cart, leaving the original script intact.
  `xm-current-pitch-historical-rollback-v1/native.log`: three checkpoints at
  frames7/4000/14925, snapshot handoff and cold reconstruction each match64
  subsequent frames of uninterrupted PCM, TrackerState and channel state.
  Actual song loop crossed: final wraps1, order0/row11 after15060 frames.
- These historical passes do not establish reference parity for the unresolved
  finetuned Amiga glissando case. No commits, pushes, cleanup, assets/SpecCade,
  licensing or dependency changes.

## Connected XM pan/gain and rollback checkpoint (2026-09-07)

**WORKING, not full XM/IT acceptance.** Historical pan/gain now has a fresh passing connected capture; extension controls, final review and broader inventory remain gates.

- Fresh `xm-connected-modes-v2` runs: FT2, MilkyTracker1.03.00 and unversioned MilkyTracker each passed five source→cart→native/official-reference pan positions, with all measured steady PCM pairs identical between players. Earlier `xm-connected-modes-v1` failed1536 vs768 hard-left; compatible control `xm-connected-compatible-before-v1` failed2048 vs768. Both reference modes use the same default preamp, disproving the worker's FT2-only0.75 factor. Imported sample preamp48 now uses the existing shared `/128` path; no blanket XM output knob. Explicit preamp uses tagged NCXM header byte14 (flag0x10); absent flag defaults48 and the16-byte header stays readable. FT2 pan uses flag0x08. The misleading generic Gxx success print in pan-only runs was corrected; those runs did not exercise Gxx saturation.
- `python C:/Users/rdave/AppData/Local/Temp/xm-connected-capture.py xm-connected-after-v1 --require-pass` exited0 with real playback success. Source SHA256 `5d0ff2a44bc0ff7c0eaad9a91da605f37f1d323e7af241b1521cc279dd3d1f9a`, tracker `MilkyTracker 1.03.00`, independently rechecked. Native peak0.96160888671875 vs reference0.960296630859375; both zero clipped PCM samples (baseline native pre-clamp1.898502,112691 clipped samples). Native stopped at order21,row0 without wraps. All1344 rows appeared in source order;2954 source note events yielded zero held-note-state mismatches; max row-time error0.01666666666668s. Native249.183333s includes capture padding, reference248.968571s. RMS/transient/window differences remain diagnostics, not proof of waveform or universal compatibility. Report, exact hashes, cart, both WAVs and native state trace: `target/tracker-compatibility/xm-connected-after-v1/`.
- Original rollback checks passed for arbitrary mid-tick snapshot→fresh-engine audio handoff, repeated resimulation after cache eviction/song loops, and seek/restart/cold replay at44.1/48kHz; compare subsequent PCM, rollback bytes and channels. Restoring the old row-seek fallback failed the new cold-reconstruction check. Snapshot handle identity and sample-clock/seek-origin metadata now survive handoff; cold recovery uses real sample/tick transitions.
- `xm-connected-checks-v4/0.log`: serialized offline nether-tracker53, nether-xm64, runtime469 passed; runtime1 ignored. This precedes latest preamp/extension edits, not their final verification. Two integration regressions were repaired: empty initial IT modules initialize header defaults; cache-only regression initializes a real module before seeding cache state. `cargo build --offline -p nether-cli -p nethercore-zx` succeeded. Optional tracker-debug binary build hit uncached `castaway v0.2.4`; no download attempted, and compatibility.py/native captures do not require that binary.
- Source oracle external-only at `f83cedb0cd5446e4dfaa83ac97e3087107e26767`: inspected default initialization, complete preamp bypass/attenuation and pan branches, configuration, extension framing. Executable provenance is separate: official libopenmpt0.8.9+r25651, SHA256 `9d809056e40e3d004b1ab9275081ee8ddb9d1874999369189d0ba172e7c3131c`; not claimed built from source pin. No reference implementation/tables/tests/assets incorporated; no licensing/dependency changes, SpecCade/source-fixture edits, cleanup or commits.
- Latest integration verified in `xm-connected-checks-v8`: converter53, parser66, runtime470 passed (1 ignored), followed by successful offline CLI/runtime rebuild. Original parse→pack→strip metadata, legacy header defaults, unsupported explicit-property errors, supplied-handle pan/preamp and snapshot tests pass. `xm-connected-modes-v3` additionally passed five positions each for default FT2/current Milky/compatible and explicit compatible-preamp96/FT2-preamp24 (25 source/cart/native/reference controls; the half-preamp FT2 case differs by at most1 PCM16 unit). Latest integration: removed OpenMPT-version threshold heuristic; length-framed explicit mix modes4/5 and preamp0..255 override header defaults and survive stripping/packing. Unknown creators/opaque trailing data are not blanket rejected. Explicit mix levels0..3, preamps>255 and other extension-controlled playback are outside accepted scope; unsupported explicit mix/preamp values error instead of being silently misplayed. Ambiguous creator-only defaults remain qualified. IT acceptance has not begun.

## Scoped XM acceptance (2026-09-07, review-repaired)

**XM VALIDATED/WORKING for the explicit scope below; IT remains unaccepted.** Owner inspected the integrated writer, mode/preamp conversion/mixer and rollback/control callers, ran the checks independently, and reproduced each reviewed regression before accepting its repair. This is not universal XM compatibility.

- Real source → freshly packed NCXM/cart → native and hash-verified official reference: `xm-connected-inventory-v4/inventory.json` has **25/25 groups**, **72 native captures**; `cases.json` binds groups to their exact arguments. Scope includes Gxx saturation, cut/keyoff/fade, simultaneous note-delay/porta/volume effects, zero-slide memory, sample volume/pan and precision, selection/keymaps/empty slots, 8/16-bit forward/ping-pong loops, high pattern indices, envelope escape/sustain freeze/set-position. Original FT2 envelope fixtures used ambiguous creator metadata; `xm-envelope-clone-v2` proves both assertions unchanged with an explicit FT2-clone header. That header correction is now in the authored generator, not in production creator inference.
- `python C:/Users/rdave/AppData/Local/Temp/xm-connected-timing.py xm-connected-timing-v1`: fresh authored speed 3→6, tempo125→150→125, alternating C/octave notes and silent rows pass both players; frequencies and scheduled sound/silence windows recorded in `timing.json`. This is bounded pitch/timing coverage, not every fine-slide or tuning combination.
- `xm-connected-after-review-v1/report.json`: full historical source hash unchanged; native peak0.96160888671875/reference0.960296630859375, zero clipping both; all1344 source rows and2954 note events, zero held-note-state mismatches, exact row order, max sampled row-time error one60Hz frame. Finite end order21/row0/no wraps. Aggregate waveform diagnostics remain supplementary, not the acceptance criterion.
- `python C:/Users/rdave/AppData/Local/Temp/xm-historical-rollback.py xm-historical-rollback-v3`: same rebuilt cart, actual native history; fresh snapshot handoff AND cold reload/reconstruction at frames7,4000,14925 each match **64 subsequent frames** of uninterrupted PCM, TrackerState and channel state. The final checkpoint crosses the real song loop at frame14932 (order20,row63→order0,row0); run ends with one wrap. PCM is compared in memory, output sink is NUL; retained Rust probe and `native.log` are the evidence.
- Reviewed sample-stripping repair preserves original local sample headers/keymap, 16-bit byte loop units and original trailing metadata. `xm-review-repairs-v1/strip-before.log` fails the original zero-keymap behavior; `strip-after.log`:67 parser tests pass. Converter53; runtime473 pass/1 ignored; direct `test_compatibility.py` harness checks pass (`harness-direct.log`; the earlier unittest-discovery zero-test run is NOT evidence). Same-handle real audio-thread merge and active-rate cold seek also have fail-before/pass-after logs in this stage. Snapshot/control epoch stays inside the existing64-byte POD layout.
- Reference executable provenance remains separate from authorized source pin `f83cedb0cd5446e4dfaa83ac97e3087107e26767`: official openmpt123 SHA256 `9d809056e40e3d004b1ab9275081ee8ddb9d1874999369189d0ba172e7c3131c`. No reference implementation/heuristic was copied; no dependency/licensing changes or asset downloads.
- **Outside accepted scope:** ambiguous/legacy ModPlug creator-specific envelope quirks; unsupported explicit mix modes; exhaustive XM effects/tuning combinations; arbitrary external tempo/volume-command histories during uncached replay; output-rate changes mid-playback without a clock-reset seek/restart; untested historical modules. These remain open, not passes. IT must now undergo equivalent scoped gates without regressing this XM result.

## Review-repair continuation (2026-09-07)

- Owner reproduced and repaired the stale sample-rate cold-seek defect: both audio generation callers now pass the active rate; reset clears the old rate. `xm_seek_after_rate_change_uses_active_generation_rate` compares five complete 48kHz frames and POD state against uninterrupted playback at the target boundary. Restoring the old 44100 fallback fails the PCM assertion.
- Same-handle restart/jump now increments an explicit control epoch in existing reserved POD storage via both game FFI callers. The real `AudioGenThread::handle_snapshot` replaces the tracker snapshot when this epoch changes, not on ordinary predictive position differences. Original real-thread regression covers same-handle restart/jump/restart, 4000 subsequent PCM pairs per command and exact state, plus unchanged ordinary prediction. Restoring handle-only detection fails. Intentional crossfade decoration is separate from underlying PCM equality.
- Exact before logs and latest full runtime result: `target/tracker-compatibility/xm-review-repairs-v1/{handoff-before.log,rate-before.log,runtime-after.log}`. Offline serialized runtime: 473 passed,1 ignored. All six temporary-directory variables explicitly set to `C:/Users/rdave/AppData/Local/Temp`. Earlier `xm-connected-checks-v9` also passed converter53, parser66, runtime471 (1 ignored), including the tempo-shortened cold seek regression.
- Broader original official-reference inventory `xm-connected-inventory-v3/inventory.json`:23/25 groups pass; envelope-escape and pan-freeze fail the reference-side assertions. Investigation is active; no assertion weakening or native acceptance inferred. Clone-header control `xm-envelope-clone-v1` was blocked by an intermediate worker compile error, not a playback result.
- Reviewer-confirmed sample-stripped XM metadata loss is being repaired in parser/write.rs plus original parser regression. That work is not yet owner-tested. IT acceptance has not started; overall XM/IT remains NOT ACCEPTED. Historical connected pan/gain evidence above remains bounded, not an overall verdict.

## Active continuation: XM playback plus rollback, then IT

- Owner goal: validate the complete XM import→NCXM→native playback and rollback path first; only then validate IT to the same standard. Do not restart the inventory or substitute isolated selection fixes for this outcome.
- Recovery: unfinished mix-mode/parser edits are unaccepted. Audit broad creator/trailing-data rejections; preserving dirty work does not mean preserving incorrect exclusions. Complete immutable mix metadata, mode-aware panning/gain and all consumers together.
- Acceptance includes original regressions, freshly rebuilt tools/carts, full historical playback, musical events/state/end and clipping checks, plus native continuous-versus-restored state/PCM at mid-tick, repeated resimulation, seek/restart, order/loop and audio-thread handoff boundaries. First-frame restart equality alone is insufficient. Independent owner review precedes a bounded VALIDATED/WORKING label; untested behavior is never a pass.
- Retained before evidence: `target/tracker-compatibility/xm-connected-before-20260907/report.json`. `napping on a cloud.xm`, SHA-256 `5d0ff2a44bc0ff7c0eaad9a91da605f37f1d323e7af241b1521cc279dd3d1f9a`, header `MilkyTracker 1.03.00`: native pre-clamp peak 1.898502, 112691 clipped samples; official reference PCM16 peak 0.960297, zero clipped samples. Report process exit 0 is not a playback pass.
- Preserve all existing artifacts/dirty IT and selection work; no commits, cleanup/deletion, dependencies, licensing or source-asset changes. OpenMPT remains an external behavioral oracle only, never implementation material. Reuse capture helpers in fresh named stages; generic `--case` fixed-output/deletion path is forbidden. Offline serialized checks use all six TEMP/TMP/TMPDIR and lowercase aliases set to `C:/Users/rdave/AppData/Local/Temp`.
- Status remains **IN PROGRESS / NOT ACCEPTED** until the above evidence and review are complete. Worker exhaustion is recoverable, not a completion gate.

## XM standalone-selection envelope retrigger checkpoint (2026-09-07)

- Pinned OpenMPT behavior at `Snd_fx.cpp:2906-2910`, `3027-3060` and
  `3095-3177` qualifies an XM instrument-only cell as an envelope retrigger:
  it resets envelope positions, fade/release flags and auto-vibrato state before
  the deferred selection is consumed, without restarting the active sample.
  The implementation keeps the existing active voice/sample and pending
  selection, then resets only that XM standalone-selection state.
- Existing original regression `xm_valid_instrument_selection_defers_voice_change_until_note`
  covers both legacy and mapped samples, a progressed envelope and released
  voice, and asserts sample position/identity plus envelope, fade, release and
  auto-vibrato reset. With the reset branch disabled it fails at the key-off
  assertion; restored it passes.
- Strengthened the existing fresh XM valid-selection source/cart/native/reference
  probe with a short first-instrument volume envelope. The probe now requires
  the selection-only window to remain audibly retriggered while retaining the
  existing relative equal-pan gain checks. `--xm-valid-selection-probe
  --xm-invalid-selection-probe` passed after fresh rebuild/packing and official
  reference capture. The invalid-selection probe remains unchanged so its
  activity/pan behavior is not conflated with this valid-selection envelope
  control.
- Focused runtime test passed: `cargo test -p nethercore-zx --lib
  tracker::engine::control_tests::xm_valid_instrument_selection_defers_voice_change_until_note
  -- --exact --test-threads=1`. Overall compatibility remains **IN PROGRESS /
  NOT ACCEPTED**; known XM pan-law/absolute-gain and other inventory gaps remain
  open. No dependency, asset, licensing, commit, cleanup or unrelated change.

## XM valid-selection verification checkpoint (2026-09-07)

- Repaired standalone valid XM instrument selection separately from invalid selection: remember the selection, reload current voice defaults without premature sample replacement, then consume the pending instrument at the subsequent instrument-less pitched note without reloading its volume/pan. Explicit note/instrument still reloads defaults. Original `xm_valid_instrument_selection_defers_voice_change_until_note` checks direct/mapped sample identity, tuning, position, volume and pan.
- Owner reran the final strengthened `python tools/tracker-debug/compatibility.py --xm-valid-selection-probe --xm-invalid-selection-probe`: exit 0 after fresh rebuild/packing/native and official reference capture. Selection-only and following-note windows have matching crossing counts **6 then 52** in both layouts, retaining left pan until explicit recovery. Volume checks compare equal-pan row 2/3 against each player's row 0 (10% relative tolerance); absolute XM gain is still a known failure, not normalized into acceptance. The worker's initial absolute-gain assertion failed and was corrected to isolate selection behavior; `--xm-pan-law-probe` remains the separate unresolved absolute/pan gate.
- Owner `cargo test -p nethercore-zx --lib --offline --no-fail-fast -- --test-threads=1`: **465 passed, 0 failed, 1 ignored**. Initial parallel run had **464 passed, 1 failed, 1 ignored**, with `replay::tests::wasm_timeout_is_an_error_report` interrupted during WASM init; serialized execution passes without editing unrelated replay code. `cargo test -p nether-tracker --lib --offline`: **53 passed**. Independent bounded review is pending; this slice is not yet accepted.
- Reference provenance follow-up: local `openmpt123.exe` SHA-256 matches its recorded manifest and reports release 0.8.9. In-memory comparison against `https://source.openmpt.org/svn/openmpt/tags/libopenmpt-0.8.9/soundlib/` found cached Git-pin `Snd_fx.cpp` lines 1459–2110 and 2906–3206 exactly match release lines starting 1463 and 2926; cached `Sndmix.cpp` 2509–2583 matches release starting 2521. Full files differ elsewhere; this is bounded source-region correspondence, not whole binary/source equivalence. No downloaded files, reference code reuse, dependencies or licensing changes.
- **IN PROGRESS / NOT ACCEPTED:** pending review, XM mix-mode/pan/headroom, and remaining inventory obligations are unchanged. Existing IT panning and unrelated dirty work preserved.

## XM K00 selection-only envelope checkpoint (2026-09-07)

- Source behavioral reading of the cached local `Snd_fx.cpp` control flow (including the reviewed K00 path) confirms K00 clears the working note/instrument before envelope-retrigger qualification. The original repair therefore keeps the shared XM standalone-selection path but excludes `TrackerEffect::KeyOff`; K00 still reaches the existing release/effect handler and `last_xm_instrument` memory.
- Added the minimal guard in `nethercore-zx/src/tracker/engine/row_processing.rs` and the original focused regression `xm_k00_selection_keeps_voice_release_and_selection_memory`. It runs both legacy and mapped layouts and checks volume-envelope position `7` is retained, K00 release flags are set, active sample handle/position remain continuous, co-located volume remains `0.5`, selection memory remains instrument `2`, and the following instrument-less note selects sample handle `22`. Baseline without the guard failed at envelope position (`0` vs expected `7`); restored code passed.
- Extended the existing `compatibility.py` fresh selection-envelope fixture with a K00 + volume control. `python tools/tracker-debug/compatibility.py --xm-valid-selection-probe --xm-invalid-selection-probe` exited **0** after fresh source/cart/native capture against the official local reference: ordinary selection still passes its existing timing/identity checks, K00 selection rows are inactive in both players without envelope retrigger, and invalid-selection activity/pan checks still pass for legacy/mapped layouts.
- Serialized verification with all six `TEMP/TMP/TMPDIR` case variants set to `C:/Users/rdave/AppData/Local/Temp`: `cargo test -p nethercore-zx --lib --offline -- --test-threads=1` returned **466 passed, 0 failed, 1 ignored**. No dependency, asset, licensing, commit, push, cleanup, or global XM gain/pan compensation was added.
- Follow-up source-bound review `deleg_6c5711c9`: **bounded pass** for the K00 exclusion and retained voice/release/volume/selection memory. Reviewer fresh valid-selection CLI exited 0; reviewer Rust execution was not established (malformed temporary linker path, then zero matching tests). Owner independently reran the full serialized library suite: **466 passed, 0 failed, 1 ignored**, and both fresh selection CLI probes exited 0. Silence in the K00 windows establishes suppression/recovery, not audible sample identity; the original Rust regression checks identity separately. This closes the concrete review findings from `deleg_5168668a` and `deleg_deb6290c` for this bounded selection slice.
- **IN PROGRESS / NOT ACCEPTED:** absolute XM gain/pan, broader inventory, and other open compatibility work remain open; this slice does not accept them or alter the existing IT pan work.

## XM sample-pan precision checkpoint (2026-09-07)

- Found a distinct silent conversion loss: single-sample and mapped XM pans were rounded from 256 byte positions to 65 IT-scale positions. Preserve original XM sample-pan bytes in `TrackerInstrument`/`TrackerSample.default_pan`; decode XM at dispatch as source/128 - 1, while IT retains its existing 0..64 units and /32 law. Raw XM/NCXM already preserves these bytes; no binary layout or dependencies changed. Field documentation now explicitly describes the format-specific units.
- Converter regression failed before repair. The existing runtime reload regression also failed before the dispatch repair. Expanded coverage checks all 256 positions, both single-sample and mapped NCXM pack/parse/conversion, and both runtime dispatch paths plus explicit pan override. Existing mapped high-edge fixture now uses the actual source byte 255, not the obsolete normalized 64.
- Owner verification: `cargo test -p nether-tracker --lib --offline --no-fail-fast`: **53 passed**. `cargo test -p nethercore-zx --lib --offline --no-fail-fast`: **464 passed, 0 failed, 1 ignored**. Fresh rebuilt source/cart/native/reference `xm_global_volume_probe(helper_root, sample_pan=True)` and its `sample_porta=True` variant both passed. These endpoint/override/portamento playback checks do not accept the known mixer pan-law/gain mismatch or every intermediate audible pan; the exhaustive state/metadata tests establish byte precision separately.
- Bounded independent source review `deleg_84b44d9a` passed: no missed XM default-pan consumer or unintended IT scaling change found. Tests were not rerun by the reviewer; execution evidence is the owner verification above. IT pan law unchanged. Overall status remains IN PROGRESS / NOT ACCEPTED.
- Original `python tools/tracker-debug/compatibility.py --xm-invalid-selection-probe` adjudicates the previously ambiguous invalid instrument-only case against the official local reference, with unchanged samples and both legacy/mapped layouts. Five rows play, center-pan, select nonexistent instrument 99 without a note, play an instrumentless note, then recover via a valid note/instrument. Before repair native became silent immediately on row 2; reference kept the voice active/restored its old pan and only became silent on row 3. After the bounded pending-selection repair the CLI exits 0 and both players follow `[left, center, left, silent, right]`. Recovery pan tolerance remains affected by the explicitly unresolved XM pan law; this is activity/transition evidence, not absolute pan calibration.
- Source control flow at pinned `Snd_fx.cpp:2906–3062` reloads active sample defaults and retriggers envelopes before suppressing the actual instrument change for this row; `3095–3177` applies pending selection on the later musical note. New `last_xm_instrument` state preserves the invalid selection independently of the active instrument; snapshots retain it. Mapped instrument-only dispatch retains current loop/voice metadata instead of reading missing legacy-instrument sample metadata. This is an original implementation using existing channel/dispatch paths, not a port of reference implementation.
- Fresh playback disproved the old legacy unit assertion that invalid selections should leave explicit pan untouched. It was replaced, not treated as an oracle: both layouts now assert old sample pan reload, active instrument retention, pending 99, reset/snapshot restoration. Existing ordinary note-only volume, K00 context, mapped sample selection, sample-pan reload, explicit override and portamento checks remain passing. **464 runtime tests passed, 1 ignored** after repair. Follow-up independent source review `deleg_857d6354` passed: no introduced defect found in immediate reload, pending selection consumption or cloned snapshot state; no blanket invalid/valid selection, delayed/special-note or gain acceptance. Generic valid instrument-only selection and XM command-pan precision remain open obligations.

## Handle-bound IT pan follow-up (2026-09-07)

- `process_channels` now derives its pan law from the explicitly supplied loaded module, alongside channel count/mix volume/stereo separation, rather than mutable row-dispatch format state. This does not change the verified IT linear formula or the unresolved XM formula.
- Strengthened `it_pan_law_preserves_sum_and_doubles_center_gain_at_edge` to restore a snapshot, exercise foreground and a real `copy_to_background` voice, and deliberately mismatch the row-format flag. Before the runtime repair the constant-sum assertion failed; after repair `cargo test -p nethercore-zx --lib --offline --no-fail-fast` returned **464 passed, 0 failed, 1 ignored**. This is explicit-handle/NNA pan evidence, not arbitrary seek or audio-thread acceptance.
- Fresh rebuilt source/cart/native/reference checks: `python tools/tracker-debug/compatibility.py --old-effects-off-probe --terminal-envelope-probe` exited 0, including special-pan controls and terminal release/restart assertions. The unchanged `--xm-pan-law-probe` was independently rerun and still exits 1 with the reference/native levels below.
- Follow-up independent source review `deleg_d878192c` found no concrete explicit-handle binding defect and verified production callers pass the corresponding raw handle. Reviewer Cargo reached linking but hit environmental `LNK1104`; executable evidence is the successful owner run above, not a claimed reviewer test pass. No changes to dependencies, licenses, assets, or pre-existing IT pan behavior.
- Source-first XM investigation: pinned `Load_xm.cpp` initializes compatible mixing, distinguishes FT2/other creators, can select original ModPlug levels, and subsequently loads extended song properties. `SoundFilePlayConfig.cpp` selects both pan law and preamp/attenuation by mix mode. Nethercore `nether-xm/src/parser/read.rs:50–51` currently discards tracker identity; `XmModule`/NCXM carries no mix mode; `from_xm_module` assigns every XM the same mix volume. Therefore a blanket square-root pan or global gain reduction is not a faithful repair. Preserve source-derived mode through import/packing before mode-specific mixer calibration; ambiguous creator/extension cases remain unverified. These are behavioral findings only: no third-party detection heuristics, tables, code, or tests were ported.

## Next verified failure: original FT2 XM pan law and steady gain

- New repeatable `python tools/tracker-debug/compatibility.py --xm-pan-law-probe` uses self-authored constant PCM, five sample pans, a standard 263-byte instrument header and FT2-identifying space padding. Fresh official reference stereo levels: `[[768,0],[665,384],[543,543],[384,665],[48,766]]`; fresh native: `[[2048,0],[1830,730],[1379,1379],[730,1830],[0,2048]]`. The native square-root-law assertion fails at pan64. This isolates both pan shape and steady headroom, rather than mistaking sinc edge overshoot for gain error.
- An exploratory 243-byte header caused the reference to select a different linear mode. Source-first check of pinned `Load_xm.cpp` confirms creator/header heuristics determine mix mode; `SoundFilePlayConfig.cpp` selects FT2 square-root versus compatible linear modes. This is a provenance/control issue, not permission to change historical assets. The original probe now uses unambiguous FT2 structure. No new XM runtime change has been applied from these measurements; format-variant routing and normalization need source-backed repair.

## Current verification checkpoint: IT pan law and review adjudication

- Corrected the pan-law test harness: raw mixer handle plus engine reset are both required. Earlier claim that only the encoded handle explained its silence was incomplete: uninitialized instrument volume/filter state also contributed. Test now exercises initialized playback, not a silent channel. Temporarily bypassing the IT linear branch fails the constant-sum assertion; restored runtime passes **464 tests, 1 ignored**.
- Existing shared foreground/NNA mixer now uses independently expressed linear amplitude distribution for IT only. XM is unchanged and its pan-law/headroom work remains open. No global gain compensation was applied. Behavioral source: pinned `Sndmix.cpp:2509-2583`; public `soundlib/SoundFilePlayConfig.cpp` at commit `f83cedb0cd5446e4dfaa83ac97e3087107e26767` was read (source only, no license/package downloaded), confirming Compatible selects linear rather than FT2/soft panning. No source code/tables were reused.
- `deleg_b0c64c02` NNA omission was reproduced by a focused assertion and repaired; selected-sample pan omission reproduced as native stereo [32,32] versus reference right-only and repaired with source-backed changed-sample gating. Old Effects selected-pan probe then exposed incorrect native center/edge gain; linear IT routing fixes that fresh probe. These changes still require final bounded review.
- Its same-sample reset recommendation is rejected for the bounded non-portamento Old Effects path: earlier `Snd_fx.cpp:3015-3024` resets before InstrumentChange's sampleChanged branch. Original rising-envelope fresh playback confirms reset with same sample (reference gain 0.529, native 0.563 in the inspected window), far below the near-endpoint retained-clock alternative. This is reset-vs-retention evidence, not exact envelope/ramping timing parity. `--old-effects-off-probe` now includes flat, selected-pan/default-volume and same-sample rising controls.
- Fixed arbitrary silence thresholds for deliberately quarter-volume special-note fixtures; gain ratios, pitch and pan assertions remain. Fresh special-note/terminal/sample-mode probes pass after IT pan routing; native absolute center gain differs from reference and is NOT accepted as calibrated.
- An original constant-PCM source control now checks absolute steady amplitude in `--terminal-envelope-probe`: native and official reference both return **375 PCM16** before fade. The square-wave peak difference (375 versus448) is not evidence by itself of gain error; the reference sinc resampler can overshoot edges. Broader headroom and gain calibration remain unaccepted.
- Review `deleg_8ccaa0df` requested for current IT pan routing. Status remains **IN PROGRESS / NOT ACCEPTED**: full inventory, absolute headroom/gain, XM pan law, remaining special-note/portamento semantics, seeking/rollback and final review obligations remain open. Dirty unrelated work and existing IT command-panning repair preserved; no dependencies, licensing changes, commits or destructive cleanup.

## Previous checkpoint: special-note interactions

- `deleg_7c10a7f1` bounded review verified terminal dispatch and identified Old Effects Note Off plus sample-mode special-note swaps. Both leads were independently reproduced using original fresh source/cart/reference probes, not accepted on review assertion alone.
- Old Effects instrument-mode Note Off: selected envelope gain was 0.25 in official reference, 1.0 native before repair. The bounded repair loads the selected envelope without swapping the running sample and separates sample-sustain release from envelope key-off. Original `it_old_effects_noteoff_changes_envelope_not_sample` fails before/passes after; `--old-effects-off-probe` passes. New source-bound review `deleg_b0c64c02` is pending; portamento/filter/invalid-selection interactions are not accepted yet.
- Sample-mode Off/Fade (Old Effects off): original different-waveform samples showed crossings [7,7] reference versus [7,16] native, confirming an unintended sample swap. Special cells now recall selected default volume without replacing the active sample. Original `it_sample_special_recalls_volume_without_replacing_voice` fails before/passes after (includes Cut state); `--sample-special-probe` passes fresh Off/Fade source/cart/reference and finite termination/restart. Later note-selection memory and Old Effects on remain unaccepted.
- Owner `cargo test -p nethercore-zx --lib --no-fail-fast`: **463 passed, 1 ignored**; `cargo test -p nether-tracker --lib`: **53 passed**. Run with all six TEMP/TMP/TMPDIR variants pointing to the real Windows user temp directory.
- No licensing changes, new dependencies, reference downloads, commits, pushes, fixture redistribution or unrelated edits. **IN PROGRESS / NOT ACCEPTED**; required new review and remaining compatibility inventory obligations remain open.

## Previous checkpoint: terminal-envelope dispatch

- Fresh original `--terminal-envelope-probe` exposed a missed production setter: channel helper setup retained the endpoint, but normal `process_note_internal` did not. Held notes with a nonzero terminal node stayed at gain 1.0 in native playback while official libopenmpt faded after tick 12.
- Added endpoint assignment alongside existing instrument envelope metadata in row dispatch. Original regression exercises actual note dispatch, NNA copy, snapshot/reset/restore and endpoint fade; fails before and passes after. No reference implementation code or assets reused.
- `python tools/tracker-debug/compatibility.py --terminal-envelope-probe --special-instrument-selection-probe` passes fresh source/cart/native/official-reference playback for held/Off/Fade cases. Window checks establish bounded onset/decay behavior, not exact tick identity or human listening acceptance.
- `cargo test -p nethercore-zx --lib --no-fail-fast`: **461 passed, 1 ignored**. Fresh envelope-enable, XM global-volume/effect/multisample, direct fade, NNA and DCT probe batch passes. No dependencies, downloads, license changes, commits or unrelated edits.
- Independent review `deleg_7c10a7f1` confirms the production setter and passing tests but reports remaining Old Effects/special-note interactions; adjudication follows. Overall remains IN PROGRESS / NOT ACCEPTED.

## Previous verification checkpoint: special-note release

- Resumed under the user's replacement licensing contract; no downloads, dependencies, license files, copied code, or redistribution were introduced in this slice. Worktree remains dirty and unrelated work is preserved.
- Default-volume regression now checks selected sample 2's volume without changing active sample 1/instrument 1; invalid selection retains the existing volume. Disabling the recall fails the focused regression; restored runtime passes. Fresh packed special-selection, empty-map and invalid/valid-sample probes pass.
- Expanded original special-selection fixture with Note Off alongside NoteFade. Official reference holds gain 1.0 during the flat non-looping envelope for Off; old native prematurely faded to 0.93663. Explicit Fade still uses approximately 0.936 gain. Same/different default-volume cases are now permanently asserted for both forms.
- Independently implemented IT release timing: explicit fade, disabled envelope, or looping release fades immediately; an enabled non-looping volume envelope runs before automatic fade. Terminal nonzero volume nodes now retain their tick in channel state and initiate fade after completion. Existing channel snapshots/NNA copies preserve the added metadata.
- Added foreground/background release and snapshot regression. The existing row-clock test now explicitly requests NoteFade, retaining its original every-tick fade assertions rather than assuming IT KeyOff always fades. Full runtime: **460 passed, 1 ignored**. Fresh special-selection, direct-fade and envelope/filter probes pass. Pinned behavioral references: `Snd_fx.cpp:6174-6222`, `Sndmix.cpp:1258-1388`; no implementation code copied.
- Independent review `deleg_843059c8` was dispatched for preceding special-note identity/default-volume changes. New release timing still needs independent review and broader terminal-envelope/flag interaction checks. Overall compatibility remains NOT ACCEPTED; prior checkpoints below are historical.

## User licensing boundary and interrupted checkpoint

- User now explicitly prohibits downloading/adding license files or incorporating third-party licensing obligations into Nethercore. OpenMPT is a behavioral reference only: no copied, translated, ported, or adapted implementation code, tables, or assets; no vendoring, linking, or new third-party dependency. Existing project license/notices stay unchanged. If a proposed reuse is legally uncertain, do not use it; ask before crossing this boundary. Existing local reference tools/fixtures are testing-only and are not release assets. No destructive cleanup of prior reference artifacts is authorized.
- Work interrupted at the user's request for a replacement goal. Latest independent special-note default-volume implementation and original probe produce native gain 0.23366 versus reference 0.23214 (previous native 0.93663). Permanent special-selection probe passes both same-volume and changed-volume cases. Latest volume-recall change has NOT yet received a focused unit fail-before/pass-after, full runtime test, or independent review. The preceding 459-pass total predates this change; do not treat it as post-change evidence.

## Special-note active instrument identity regression

- Permanent `--special-instrument-selection-probe` now asserts NoteFade with an explicit second instrument retains the first instrument's enabled volume envelope. Fresh source/cart/native/reference probes pass, including subsequent selection memory and valid recovery. Existing empty-selection and invalid/valid sample probes also pass.
- Added unit coverage for Off/Cut/Fade with valid instrument 2 and invalid selection 99: active instrument stays 1 while selection memory updates. Removing the instrument-identity guard fails this test; restoring it passes. Full owner runtime **459 passed, 1 ignored**.
- This supersedes the diagnostic-only status below for active instrument identity. Default-volume recall from a different sample, old-effects envelope-reset exceptions, special-note sample-mode behavior and source-bound review of final changes remain open. Not overall acceptance.

## Source-first direction and current checkpoint

- User explicitly prioritizes OpenMPT implementation source for complex compatibility behavior, with license care. Existing architecture and original acceptance/boundaries remain unchanged. Trace complete relevant OpenMPT paths (row preparation, compatibility-flag selection, effect dispatch, voice lifecycle, mixer), derive expected semantics, then implement in Nethercore and verify with discriminating fresh playback. Do not infer rules from aggregate audio metrics or isolated source snippets.
- Pinned OpenMPT root LICENSE was fetched and inspected: BSD 3-Clause terms, copyright OpenMPT contributors 2004–2026 and Olivier Lapicque 1997–2003. Source/binary redistribution requires retained/reproduced notices, conditions and disclaimer; no endorsement. Component-specific code and third-party fixture licenses must still be checked. Reference-only source inspection does not authorize untracked code copying.
- In-flight bounded special-note repair: original `--special-selection` fixture uses two different volume envelopes. Before guard, NoteFade + instrument 2 changed native gain to 0.23366 versus reference 0.93527; retaining active instrument identity gives native 0.93663. The latest row_processing guard covers Off/Cut/Fade instrument identity. Only this diagnostic has run post-change; permanent probe assertions, full tests, default-volume exceptions and independent review are still pending. Do not report the slice as accepted.

## Empty-map whole-cell effect suppression

- Independently confirmed review finding 1 with an original explicit instrument-2 empty-map note plus volume 16. Official 0.8.9 retains gain 1.0 on the empty mapped musical cell; native incorrectly reduced it to approximately 0.25. The instrument-only sibling correctly uses 0.25 in both players. This discriminates the effect behavior that the previous pitch-only probe missed.
- Suppression now occurs in row normalization before Sxx memory resolution and timing collection, preserving pending instrument selection. Cached normalized notes also drive delayed-row fine slides via the existing `row_notes` helper; no new playback architecture. The old implicit-map path remains separate.
- Extended the original probe with `--selection-volume` and the repeatable `--empty-instrument-selection-probe` assertions. Both Gxx flags, pitched/command-only cells and with/without portamento now match the expected gain and following mapped note. Existing invalid sample and delayed selection controls pass.
- Extended the snapshot regression with volume and S63: asserts no S-memory change, no fine row delay, and a cleared normalized cell while selection survives. Full owner runtime **458 passed, 1 ignored**. Special-note instrument/default-volume semantics and broader compatibility remain open; no full acceptance claim.

## Empty-map snapshot and cross-format checkpoint

- Added `it_empty_map_selection_survives_snapshot_without_replacing_voice`: verifies the empty-map cell leaves the active instrument/period intact, selection 2 survives an actual engine snapshot, and the following instrument-less note uses instrument 2's transposition. Removing the repair block makes the regression fail; restoring it passes. Full owner runtime: **458 passed, 1 ignored**.
- Fresh `--xm-global-volume-probe --note-fade-probe --nna-switch-probe --dct-parent-probe` completed successfully, including XM multi-sample mapping and the existing bounded envelope/effect controls. These do not certify all behavior.
- Independent review `deleg_f21400a4` supports pending selection for ordinary musical notes, but raises two separate interaction leads: empty-map co-located effects and explicit instruments beside Note Off/Cut/Fade. Owner inspected pinned `Snd_fx.cpp:597-606`: whole-cell suppression occurs during row preparation under `kITEmptyNoteMapSlotIgnoreCell`, while `nNewIns` retains the selection. This is distinct from invalid-instrument suppression later in effect processing. Verify that compatibility flag and official-reference behavior with volume/effect-sensitive controls before changing dispatch; existing pitch-only probes do not adjudicate it. Special-note instrument handling likewise needs dedicated controls, including default-volume exceptions. No blanket review pass claimed.

## Empty-map instrument selection repair

- Confirmed the remaining review lead using an original two-instrument module: instrument 2 maps note 64 to no sample, but maps subsequent note 72 down to note 60. Before repair the ignored empty-map cell discarded selection 2; the following instrument-less note played an octave too high (native 15 crossings versus reference 8 in the same 30 ms window). The active voice itself must remain untouched on the empty-map row.
- `row_processing.rs` now remembers explicit selection before empty-map suppression and applies a pending different selection on the next musical note. No sample-mode or XM behavior was intentionally changed. The memory remains in the cloned channel state.
- Repeatable check: `python tools/tracker-debug/compatibility.py --empty-instrument-selection-probe`. Fresh source/cart/native/official-reference checks pass for pitched/command-only cells, with/without portamento, both Gxx flags, subsequent mapped pitch and explicit recovery. Native/reference following pitch now both 8 crossings in the observed window.
- Existing invalid-selection, SD0/1/5/6/F and envelope/filter probes pass; owner runtime **457 passed, 1 ignored**. Independent bounded review `deleg_f21400a4` requested; not yet accepted. Broader compatibility remains IN PROGRESS / NOT ACCEPTED.

## Invalid selection review adjudication and delayed boundary probes

- Reviewed `deleg_e4bab317` against pinned OpenMPT `f83cedb0cd5446e4dfaa83ac97e3087107e26767`, rather than accepting its requested changes as proof. Its special-note claim is incorrect: `soundlib/modcommand.h` explicitly defines `IsNoteOrEmpty` as empty OR musical note, not note-off/cut/fade. Do not expand the invalid-selection suppression guard to special notes on that basis.
- Its out-of-range delay claim is also disproved for the tested boundary: `Snd_fx.cpp:2761-2769` continues before the `nOldIns` update at 3871-3874. Fresh official 0.8.9 playback allows the following instrument-less octave after SD6/SDF at speed 6; in-range SD0/SD1/SD5 suppress it. Native matches both behaviors without a runtime change. `nNewIns` is a separate reference state; wider pending-instrument interactions remain unaccepted.
- Added original generator `--selection-delay=N` and repeatable `python tools/tracker-debug/compatibility.py --invalid-instrument-delay-probe --invalid-sample-probe`. Each delay checks pitched/command-only selection and both Gxx flags through fresh source/cart/native/reference playback, including original pitch, delayed selection, following octave, and explicit recovery. All passed. Owner runtime: **457 passed, 1 ignored**.
- The completed review also confused a source-file SHA-256 with a Git commit identifier. Fresh retrieval from `https://raw.githubusercontent.com/OpenMPT/openmpt/f83cedb0cd5446e4dfaa83ac97e3087107e26767/soundlib/Snd_fx.cpp` is byte-identical to both cached copies (`Snd_fx.cpp`, `openmpt-Snd_fx.cpp`): SHA-256 `5a7e85cddcc9a90fa06856b0c7f6ec61e691279b4389f4450b4cfe7a8e6a1a6e`. No source-pin correction is needed; executable provenance remains a separate issue.
- This adjudicates those specific review claims, not a blanket clean review or overall acceptance. Empty-map selection memory and other stated gaps remain open.
- A separate fresh historical `FT2PanLaw.xm` capture remains UNREVIEWED. Pinned `Sndmix.cpp:2530-2545` uses square-root FT2 panning (right endpoint capped at 255/256) or linear panning, whereas native uses a sine LUT for both formats. Do not apply a blind master-gain correction for historical clipping; pan-law/format-routing and byte precision must be verified with discriminating controls.

## XM historical headroom localization

- Fresh official libopenmpt0.8.9 float output, gain0/stereo100/filter8/ramping-1,
  repeat0 at249.008s has maximum0.960304 (left0.931157), not clipped. Native
  source/cart renderer previously measured1.8981972 and retains its failure.
- This rules out treating the observed native overflow as unavoidable authored
  clipping under these reference settings. It does not justify blindly halving
  mixer gain: Sndmix.cpp:2124-2158 includes player preamp/attenuation settings,
  so source-aligned gain and per-voice behavior still require localization.
- No gain compensation, asset edits, or softened clipping assertions applied.
  Invalid-instrument selection review deleg_e4bab317 is running independently.

## Invalid instrument selection memory follow-up

- Strengthening the following instrumentless note to an octave exposed a real
  memory defect: reference kept the old tone while native crossings rose7/8->15.
  A separate last_it_instrument selection now remembers invalid instrument IDs
  without replacing active voice ownership. Subsequent instrumentless notes
  remain suppressed until a valid explicit selection restores playback.
- The instrument-mode fresh source/cart/native/official-reference probe now uses
  the octave follow-up and passes. Existing sample-mode and envelope/recall
  probes pass. Snapshot/reset regression preserves invalid selection separately
  from active instrument and clears it on reset. Runtime457 passed/1 ignored.
- Independent source-bound review of this new memory/guard slice remains pending;
  complete arbitrary rollback and broader format acceptance remain open.

## Instrument-mode invalid pitched cells

- Fresh original instrument-mode fixture confirms the pitched invalid instrument
  defect: reference preserves its old tone; native ordinary note went silent and
  native portamento changed pitch. Pinned Snd_fx.cpp:2887-2896 applies the
  invalid-instrument guard to both notes and empty cells, unlike sample mode.
- Expanded the existing guard to discard the invalid instrument AND pitched note
  while leaving volume/effects available. Runtime456 passed/1 ignored.
- Extended --invalid-sample-probe with instrument-mode variants across both flags,
  pitched/command-only, subsequent instrumentless note and valid recovery.
  Full invalid/valid sample-mode and instrument-mode probe passes fresh carts and
  official reference. General invalid-instrument memory remains outside this
  bounded same-pitch follow-up; full compatibility is still not accepted.

## Compatible Gxx memory review adjudication

- Pinned OpenMPT Snd_fx.cpp:4538-4540 explicitly shares G/E/F memory when
  SONG_ITCOMPATGXX is NOT set. The runtime negation is therefore intentional;
  reviewer concern about reversed semantics was caused by legacy flag naming.
- Corrected misleading nether-tracker flag/accessor documentation without
  changing bit values, conversion or runtime behavior. Invalid pitched
  instrument-mode cells remain a separate unadjudicated review lead.

## Historical full-song recheck and loop classification

- Fresh source/cart captures of both licensed historical modules still fail the
  current helper's acceptance checks. `Brute VII 2.it` runs173.8s then reports
  order1/row2/wraps1/playing=true at174s; `napping on a cloud.xm` reports native
  peak1.8981972 and fails the shared renderer's unclipped-output assertion.
- Independent source inspection corrects the Brute interpretation: its order
  list is0..32,255 and pattern32 row63 explicitly contains B01. The observed
  return to order1 is authored looping, not evidence of accidental overrun.
  Reference default one-playthrough duration cannot establish that native must
  stop here. This requires an explicit deliberate-loop check; do not erase the
  existing capture failure or call the whole historical song accepted.
- Added explicit `--historical-loop-probe`: fresh source/cart/native playback at
 174s asserts order1/row2/wraps1/playing=true and audible continuation. Official
 libopenmpt0.8.9 with repeat1 is independently audible after first traversal.
 This bounded B01 flow check passes; finite-song helper assertions are unchanged.
 It does not accept every loop interaction or whole-song musical fidelity.
- XM clipping remains a real capture failure requiring reference-aligned gain/
  headroom investigation; no asset attenuation or assertion weakening applied.
- Source-bound review deleg_3c0a330f found the latest sample-mode Compatible Gxx
  command-only retention correctly scoped. Reviewer reports four sample-mode
  tests, compatibility self-tests and invalid/valid sample probes passed.
  Separate leads require independent adjudication: LINK_G_MEMORY naming versus
  negated runtime sharing semantics, and instrument-mode invalid pitched cells
  escaping the note-empty-only guard. Neither is accepted as proof from the
  review alone. Overall compatibility remains unaccepted.

## Invalid sample-mode numbers: reference adjudication

- Fresh original fixtures against official libopenmpt0.8.9 reject the proposed
  blanket no-op: ordinary invalid sample numbers silence playback, with or
  without a pitched note. Non-compatible Gxx also silences. Compatible Gxx
  retains the playing sample for both pitched and command-only cells.
- Native matched those checks except command-only Compatible Gxx, where it
  became silent (native peaks505/0/0 vs reference448/448/448 at .06/.2/.5s).
  This independently confirms a narrower defect than deleg_1e615ff6 proposed.
- Runtime now retains active sample metadata/gain for Compatible Gxx commands
  even without a pitched note. Follow-up reference probing also requires retaining
  the active sample selection for the next instrument-less note.
  Repeatable `--invalid-sample-probe` covers pitched/command-only and both Gxx
  modes using fresh source/cart/native and pinned reference. It passes after
  repair; stopped-sample restart probe also passes. Runtime456 passed/1 ignored.
- Added next-row instrument-less note and subsequent explicit valid-sample recovery
  to the original fixture. Reference retains audio after Compatible Gxx in both
  pitched and command-only cases; native command-only was silent. Moved the
  retention predicate before sample-memory assignment and gated that assignment.
  Fresh probe now matches reference activity at all three transitions, with
  valid-sample recovery also asserted. Runtime456 passed/1 ignored and fresh
  sample-restart probe passes. Broader valid-sample porta interactions remain open.
- Follow-up valid second-sample control uses a doubled-frequency waveform.
  Fresh reference/native crossings across onset/porta/next-note/recovery match:
  command-only incompatible8/16/16/8, compatible8/8/8/8; pitched compatible
  8/9/8/8; pitched incompatible reference8/19/16/8 vs native8/20/16/8.
  The repeatable invalid-sample probe now also runs these valid-sample siblings,
  asserting activity and frequency transitions with one-crossing tolerance.
  No additional runtime repair was required for these bounded sibling cases.
- This supersedes the review's blanket invalid-sample-no-op recommendation,
  not all invalid-instrument/sample memory behavior or general acceptance.

## Cross-format regression checkpoint after sample-mode repairs

- Fresh `--note-fade-probe --nna-switch-probe --dct-parent-probe` passes all
  existing bounded controls: direct fade, empty-map preservation, SC0 timing,
  NNA defaults/overrides, DCT parent/instrument/source-sample identity. Captures
  rebuild/repack and exercise termination/restart; no blanket lifecycle claim.
- Fresh `--xm-global-volume-probe` passes the existing gain, cut/release, delayed
  cells/volume effects, sample defaults/panning, high pattern indices, envelope
  release/Lxx, loop-width/type and two-sample/empty-slot mapping controls.
  These are concrete covered cases, not proof of all XM combinations.
- `cargo test -p nether-tracker -p nether-it -p nether-xm` exits0: unit suites
  report62/53/64 passed; doctest suites report1 passed and2/2/4 ignored.
- Historical song-control adjudication, broader filter and voice lifecycle,
  complete seeking/rollback, reference/provenance and corpus coverage remain
  open. Overall status remains IN PROGRESS / NOT ACCEPTED.

## IT sample-mode instrument-only restart

- Pinned OpenMPT Snd_fx.cpp:3027-3047 restarts the remembered note after sample
  end in both instrument and sample modes. The runtime restricted this behavior
  to instrument mode and validated only instrument headers. Validation now uses
  the selected mode's table and the same restart branch serves both modes.
- Extended existing sample-mode regression: ended voice + same sample number
  failed to set note_on before, and now restarts at position0 with remembered
  pitch rather than the previous slide. Runtime456 passed/1 ignored.
- Repeatable `python tools/tracker-debug/compatibility.py --sample-restart-probe`
  generates original one-shot implicit/explicit controls via note_fade_probe.rs.
  Fresh source/cart/native restart onset fails with the old mode guard and passes
  with the fix. Within each player, implicit and explicit first0.3s PCM match;
  official libopenmpt0.8.9 is independently pinned by the existing helper.
  Fresh envelope/recall checks also pass. Original historical assets unchanged.
- Review deleg_1e615ff6 requests changes for invalid sample numbers, but owner
  source inspection disputes its cited basis: Snd_fx.cpp:2887 explicitly gates
  the :2889-2896 invalid-instrument no-op on GetNumInstruments()!=0. Sample-mode
  InstrumentChange instead selects Samples[0] for an out-of-range sample (:1463).
  Therefore do not apply an instrument-mode no-op to sample mode on this review
  alone. Invalid-sample behavior needs a targeted official-reference comparison;
  the review is unresolved, not a bounded PASS or confirmed no-op defect.
- This does not accept the full historical PortaResetAfterRetrigger.it fixture,
  all instrument-only effect interactions, or general tracker fidelity.

## IT sample-mode portamento position follow-up

- Pinned OpenMPT Snd_fx.cpp:3095-3137 applies pitched sample-swap position reset
  in sample mode as well as instrument mode. Native sample-mode dispatch omitted
  this reset. New real-dispatch regression failed (position5.5, expected0).
- Reset now occurs only for a changed sample under incompatible Gxx. Same-sample
  cells and Compatible Gxx retain position; pitch is not retriggered. Corrected
  an older control assertion that had incorrectly required position12 for both
  flags; source semantics require0 on actual swaps,12 on retained samples.
- Full runtime456 passed/0 failed/1 ignored. Fresh envelope/filter/recall source
  -> cart -> native/reference probe passes. Fresh PortaSample.it and
  PortaSampleCompat.it captures remain UNREVIEWED; aggregate differences persist.
- Instrument-less instrument-mode portamento is not itself a missing remap:
  Snd_fx.cpp:3095 only calls InstrumentChange for an explicit instrument and
  :2080 retains an active sample during portamento. Added retained-handle/index/
  position/release assertions in both compatibility modes; they pass unchanged.
- Independent source-bound review deleg_a654bab9 found no concrete defect in
  this bounded position repair. It confirmed changed-sample reset, same-sample
  and Compatible Gxx retention, and that the older assertion correction follows
  source semantics rather than weakening coverage. Reviewer reports six targeted
  subprocess tests passed. The later instrument-only restart change is separate;
  historical fixture and overall compatibility acceptance remain open.

## IT same-instrument explicit multisample portamento follow-up

- Confirmed reviewer defect with an added row in the existing real-dispatch
  regression: instrument1/sample4 to instrument1/sample1 retained handle10
  instead of7. The branch now accepts explicit same-instrument cells; only a
  genuinely changed instrument clears key-off/fade. Compatible Gxx still retains
  the playing sample/position. Same-instrument release remains set in the test.
- Regression failed before and passes after; runtime455 passed/1 ignored.
  Fresh source/cart envelope/recall probes pass. Instrument-less remaps and
  complete portamento semantics remain unaccepted; this is not blanket repair.

## IT sample global volume (bounded repair)

- Row2 source sample2 has global volume20; the other audible samples use64.
  The converter retains this field but mixer omitted it. Shared audible mixer
  now multiplies by the active sample global-volume scalar, including NNA voices.
- Focused foreground/background20/64 and zero-volume regression failed before
  the multiplier and passes after. Runtime455 passed/1 ignored. Fresh historical
  capture pan MAE0.16157007670263202; still UNREVIEWED, not a fixture pass.
- Original/decoded sample-equivalence passed both players before and after,
  ruling out compressed sample decoding as the observed volume discrepancy.
- Fresh official0.8.9 comparison after the multiplier resolves the localized
  row2 imbalance: settled-window reference/native RMS0.086822/0.086360,
  pan0.010357/0.018499, crossings6/6 (previous native pan-0.822).
  These are bounded musical-feature checks, not whole-fixture acceptance.
  Independent source review deleg_e6233b48 covers this gain fix and the latest
  explicit same-instrument portamento change returned bounded PASS/no defect.
  Reviewer confirmed single application and foreground/NNA index ownership,
  defaults, release-state gating, and compatible sample/position retention.
  Reviewer full-suite attempt hit Windows linker/temp failure; owner reruns
  above supply fresh successful execution. This does not cover the later
  sample-mode position follow-up.

## IT instrument-portamento Compatible Gxx follow-up

- Independently found mapped-sample switching ignored Compatible Gxx in the new
  instrument-mode branch. Pinned Snd_fx.cpp1517-1524 retains the playing sample.
- Extended the existing row-dispatch sustain fixture with the compatibility flag:
  it failed before the switch guard and passes afterward, preserving the old
  sample handle/index. Full runtime454 passed/1 ignored; fresh envelope probe
  passes. Independent review deleg_b1a0cdd3 confirms Compatible Gxx gating,
  swap-position reset and released-sustain lifecycle in the inspected scope.
  It identifies a remaining same-instrument multisample defect: the swap path
  requires an instrument-number change, so a mapped-sample change within the
  same instrument is skipped. Not a complete portamento PASS. Reviewer focused
  sample-mode tests:3 passed; broader execution hit Windows temporary-path linking
  failure. The later sample-global-volume mixer fix was outside that review.
- Follow-up: pinned Snd_fx.cpp3130-3137 resets sample position on IT pitched
  sample swaps. Extended regression starts at5.5 and failed before the fix;
  now actual swaps start0 while Compatible Gxx retained samples stay5.5.
  Runtime454 passed/1 ignored. Fresh historical sustain fixture still differs;
  position repair alone does not resolve row2. Review brief updated.
- Fresh PortaSampleCompat.it source/cart capture remains UNREVIEWED with five
  activity-disagreement windows and pan MAE0.4424818395535155. Not a fixture pass
  and not full instrument/sample portamento acceptance.

## IT S00 runtime command recall (repair in progress)

- New source/cart probe independently confirms S00 fails to recall prior SD3:
  official libopenmpt0.8.9 matches explicit-SD3 control PCM exactly; native does
  not. Source Snd_fx.cpp2744-2747 uses channel-local last nonzero Sxx memory.
- Added SD3 recall, S62 timing recall and SE1 row-repeat recall fixtures to the
  existing generator/probe. Official-reference S00/S62 and S00/SE1 match their
  explicit-command control PCM exactly. A fourth pair confirms S81 replaces
  earlier SD3 in the same S00 memory (reference PCM exact), not a separate
  per-subcommand delay memory.
  Runtime resolution must precede timing collection and delayed-cell dispatch;
  pre-expanding in source pattern order would be incorrect across song jumps.
- Runtime repair now retains raw ItExtended commands and resolves channel-local
  nonzero S memory before row dispatch/timing. Resolved row cells feed repeats
  and flow; memory and resolved-row state travel with engine snapshots.
- Fresh owner `python tools/tracker-debug/compatibility.py --envelope-enable-probe`
  passes all four recall/control pairs in native and official libopenmpt0.8.9,
  plus the existing envelope/filter/delay probes. Converter53 passed; runtime450
  passed/1ignored. Three old control tests now exercise source conversion through
  real row resolution, retaining SC0 timing and S7x voice-ownership assertions.
- Post-integration fresh `--nna-switch-probe`, `--dct-parent-probe` and
  `--note-fade-probe` pass, including S73/S74 ownership, direct fade and SC0 timing.
  These exercise the raw-S transport through fresh carts rather than only unit dispatch.
- Fresh historical `--case SampleSustainAfterPortaInstrMode.it` still reports
  5 activity-disagreement windows, pan MAE0.2779447248685186 and zero-crossing
  MAE4.841059602649007. This remains UNREVIEWED/not accepted; the corpus path uses
  its recorded FFmpeg reference and is not the official0.8.9 envelope oracle.
- Review deleg_25493a41 found correct bounded S00 transport but a seek defect:
  sync treated IT order254 as end-of-song and restarted. Owner reproduced a
 20-second timeout after the targeted test began. Seeking now reuses playback
  normalize_order before reading/caching rows; the same regression passes and
  checks retained SD3 memory plus subsequent S00 dispatch. This is bounded marker
  traversal, not full tick-accurate seek reconstruction. Independent bounded
  review deleg_da06ff97 returned bounded source PASS for IT254/255 normalization,
  XM ordinary254/255 indices, target validation and normalized-target stopping.
  Reviewer converter tests:53 passed; runtime linking failed on malformed temp
  path, so runtime evidence remains the owner's two passing seek regressions.
  Full tick-accurate rollback is not established. End-marker255 and skip-marker254 regressions
  now both pass after concurrent sustain-state integration restored compilation.
  Latest owner full runtime:453 passed/0 failed/1 ignored; new instrument-portamento
  regression now passes. Fresh envelope/recall packed probe passes.
  Fresh official0.8.9/native localization: all five activity disagreements are
  terminal fade windows6.86..6.94s (reference audible/native zero), not sustain
  onset. A separate residual is row2 (~80ms after its start): reference pan0.010
  versus native-0.822 when note51/instrument2/volume202 switches to sample2.
  Other active rows at the same offset are near centered. These diagnostics do
  not accept the fixture; changed-sample portamento and terminal tails remain open.
  Fresh historical capture still has5 activity disagreements; pan MAE now
  0.2288094961313077 and crossing MAE4.357615894039735 are diagnostics, not acceptance.
- Reference cross-check for unmodified SampleSustainAfterPortaInstrMode.it:
  official0.8.9 rendered at44100Hz/stereo100/filter8/ramping-1/gain0/dither0,
  repeat0/subsong-1, versus existing FFmpeg libopenmpt defaults at44100 stereo:
  348 paired20ms windows, zero activity disagreements, pan MAE0.12368148879543583.
  Settings are not identical; this is not proof of oracle equivalence and pan
  measurements must not be interchanged. Temporary WAVs removed after analysis.
- Full S-command behavior,
  arbitrary seek/rollback and overall compatibility remain unaccepted.

## IT delayed-note row repetition (bounded)

- Pinned Snd_fx.cpp2816-2821 retriggers delayed notes on repeated rows. Existing
  Nethercore prepare_row repeated only fine pitch effects, leaving SDx consumed.
- Original two-channel fixtures compare SD1 plus SE1 with expanded plain rows
  containing SD1 twice. A short volume envelope isolates both audible events.
  Reference first220ms matched exactly; native missed the second note (peak0).
- Repeated IT rows now re-arm only authored NoteDelay cells through the shared
  deferred path; other note cells are not retriggered. No additional cached state.
  Fresh source/cart/reference matches expanded-row PCM within each player;
  second-note native650/reference483. This does not claim cross-player amplitude
  equality. Complete envelope probe and runtime448 passed/1ignored.
- Broader S-command memory, XM row-delay behavior, NNA/carry interactions and
  complete rollback remain unaccepted. Review deleg_cc1f8965 returned bounded
  static PASS for repeated-row gating and SDx-only rearming; no test rerun.

## IT SDx plus fine tick delay boundary (bounded)

- Rechecked dirty sole worktree; existing panning and unrelated changes preserved.
  Pinned Snd_fx.cpp2750-2769 uses speed plus frame delay for SDx validity.
  Nethercore's row clock already included fine_pattern_delay but its deferred
  note dispatch compared only against base speed.
- Fresh two-channel source fixture SD6 with S62 at speed6 failed: reference
  early0/late483, native0/0. The IT-only dispatch boundary now includes fine delay;
  XM retains its base-speed rule. Fresh source/cart/reference now native0/650,
  reference0/483. All existing envelope probes remain passing.
- Unit tests cover the exclusive boundary: delays6 and7 trigger with speed6+2;
  delay8 does not. Runtime448 passed/1ignored. Review deleg_55db91ca returned
  bounded PASS for the extended boundary and later-channel timing collection.
  Reviewer Cargo execution hit LNK1104 and timeout; owner execution remains
  the test evidence. Neither review establishes S00 recall compatibility.
- This does not accept broad fine-delay interactions, repeated rows, S-command
  memory, out-of-range instrument memory or complete rollback.

## IT delayed volume-only and cut cells (bounded)

- Source Snd_fx.cpp2749-2821 and2858 gates note/instrument/volume processing by
  delayed trigger time. Rechecked dirty sole worktree and guidance; preserved
  unrelated changes and existing panning repair.
- Fresh SD3 volume-only probe failed before repair: reference early483/late0,
  native0/0 (premature row-start mute). The shared deferral gate now includes all
  IT cells, while the XM condition is unchanged. No extra state or new engine.
- Fresh packed volume-only and note-cut probes pass: native650/0; reference483/0.
  These are timing assertions, not amplitude-parity assertions. Unit regression
  proves no sample-position restart for volume-only/cut and correct note lifetime;
  reverting only the gate makes it fail, restoring the gate passes.
- Runtime447 passed/1ignored. Review deleg_05d8777b returned bounded PASS;
  reviewer independently ran the focused regression (1 passed, 447 filtered).
  This review does not cover the subsequent SDx/S6x boundary change. Full SDx memory,
  row-delay repeats, fine delay, out-of-range instrument memory, general special
  note release and complete rollback remain unaccepted.

## IT delayed filtered onset (bounded)

- Re-read guidance and sole dirty main worktree; preserved unrelated changes and
  the existing IT panning repair. No commits or sibling asset modifications.
- Fresh source/cart/native SD1 fixture failed before repair: peak650 before tick1
  versus official libopenmpt0.8.9 peak0; post-onset windows were native452 and
  reference483. This is an onset/timing defect, not a gain acceptance comparison.
- Pinned Snd_fx.cpp2749-2821 defines SD0=SD1, out-of-range handling and delayed
  trigger timing. Independently generated official-reference SD0 and SD1 produce
  exactly equal PCM; SD6 at speed6 stays silent through subsequent empty rows.
- Runtime repair deleg_f65eb57b reuses the delayed-cell path: SD0 maps to tick1;
  actual trigger applies instrument, volume and filter initialization. Owner fresh
  source/cart/reference probes pass for SD0, SD1, sample mode and SD6 silence.
  SD0/SD1 PCM matches within each player. SD6 has a later audible row3 control,
  retaining the renderer's silence/clipping guards without weakening assertions.
- A subsequent shifted-control probe exposed delayed envelope advancement skipping
  authored tick0: reference onset PCM matched its control (peak113/113), native
  failed (650/53). Actual note processing now reports envelope reset; per-tick
  advancement excludes only newly initialized voices, retaining NNA/porta and
  normal row-boundary advancement. Fresh shifted native control now matches
  exactly (53/53); reference remains exact. This does not assert amplitude parity.
- Owner runtime446 passed/1ignored; XM64, IT62, converter52, CLI80 passed.
  Fresh IT default-panning and XM packed probes still pass.
- Onset review deleg_ac410463 returned bounded PASS for deferred initialization,
  SD0 mapping, XM gating and snapshot transport. Its targeted test linking failed
  with LNK1104; owner execution above remains distinct.
- Changed-envelope review deleg_5ae7cb15 returned bounded PASS for actual-reset
  reporting, newly initialized voice masks, continuing NNA/non-retrigger porta
  advancement and row-boundary callers. Static review only: no independent rerun.
  Neither review accepts the excluded broader SDx or rollback areas below.
- Still unaccepted: full SDx (special-note/volume-only cells, S-command memory,
  out-of-range instrument memory, fine tick delay, row-delay repeats), filter
  lifecycle and complete rollback. Onset checks do not establish native/reference
  amplitude equality or human audible fidelity.
- Separate source discrepancy to adjudicate: Snd_fx.cpp2704-2719 documents whole
  cell suppression for kITEmptyNoteMapSlotIgnoreCell, unlike the current local
  empty-map path retaining effects. Do not count earlier bounded map tests as
  complete compatibility-flag acceptance.

## IT full-cutoff note transition (bounded)

- Re-read guidance and dirty sole main checkout/worktree; unrelated changes and
  existing IT panning repair preserved. Verified pinned Snd_flt.cpp full-cutoff
  branch: full cutoff with zero resonance disables only on a note trigger; without
  a note it retains prior coefficients and active filter state.
- Added explicit active/pending-note state to the existing channel. Live and legacy
  note paths mark the transition through their shared filter-envelope reset.
  Activation clears stale filter history; continuing active filtering retains it.
  Channel Clone/snapshots carry both flags alongside coefficient/output history.
- Original generated source/cartridge probe failed before repair: native no-note
  Z7F jumped to16.889x first-row level while reference remained1.001x. After repair
  native remained0.998x; a subsequent note+Z7F opens the filter. The no-note Z7F
  module and blank-command control produce EXACT matching PCM within each player
  independently, using pinned official libopenmpt0.8.9 and freshly rebuilt native.
- Runtime445 passed/1ignored including retained coefficients, note-only bypass,
  and reactivation with deliberately contaminated prior history. Fresh packed
  envelope/filter/resonance probes pass. Review deleg_4fb2eab7 returned bounded
  PASS for note/no-note transitions, activation history and cloned state transport.
  Reviewer targeted regression passed (1 passed, 445 filtered); full test target
  encountered LNK1104, so owner-side full-run evidence above remains distinct.
  Cloned state transport does not establish complete rollback fidelity.
  Prior DSP review deleg_323278f7 returned bounded PASS (details below).
- This is not complete lifecycle acceptance. Filter-envelope retrigger ordering,
  delayed/porta interactions, output-rate changes during full-cutoff hold,
  extended-range flags, filter clipping, historical discrepancies, song control
  and full rollback remain IN PROGRESS / NOT ACCEPTED. No commits/pushes or
  sibling music-asset edits.

## Original-IT normal-range filter response (bounded)

- Re-read repository guidance/worktrees and preserved the dirty main checkout.
  Confirmed the prior implementation used cookbook low-pass biquad coefficients
  and Q=1+10*resonance, rather than the pinned OpenMPT Snd_flt.cpp original-IT
  damping/r/d/e equations. Also confirmed full cutoff incorrectly bypassed
  filtering even with nonzero resonance.
- Replaced only filter coefficient calculation/recurrence within the existing
  channel/mixer path. Output-history samples now survive live coefficient changes
  without carrying old coefficient-weighted transposed state. No engine change.
- Fresh generated source/cart/reference resonance sweep failed before repair:
  cutoff64 high-resonance ratio native1.428783 vs reference1.505013; after repair
  native1.499204. Full-cutoff resonance previously stayed flat; repaired ratio
  1.212827 vs reference1.223090. The original .03 normalized response bound is
  unchanged. Both sweeps use the hash-pinned official libopenmpt0.8.9 reference.
- Numeric impulse vectors independently calculated from pinned source cover
  22050/44100/48000 Hz, resonance at full cutoff, and a live coefficient change
  retaining previous outputs. Runtime444 passed/1ignored. Existing packed filter
  defaults, envelope sweep/pause/start-disabled and restart/termination probes pass.
- Review deleg_323278f7 returned bounded PASS for normal-range coefficients,
  output-history recurrence and full-cutoff resonance. Independent impulse
  calculations matched at 22050/44100/48000 Hz; reviewer cargo check passed.
  Reviewer targeted test linking hit transient LNK1104; owner-side passing tests
  above remain the execution evidence. This review does not accept the later
  Z7F lifecycle slice. Full active/bypass lifecycle and reset, filter
  clipping, extended-range/compatibility flags, stereo, carry, historical corpus,
  song control and complete rollback remain IN PROGRESS / NOT ACCEPTED.

## IT filter-envelope evaluation and reference adjudication (bounded)

- Re-read guidance and sole dirty main worktree; preserved unrelated work and IT
  panning. The standing contract and all prohibitions above remain unchanged.
- Confirmed live notes excluded filter envelopes and mixer evaluation overwrote
  base cutoff with signed envelope values divided by64. Fresh filter_env source
  failed native attenuation before repair (all three windows near unity), while
  reference swept low/mid/high. Base cutoff is now retained, with an independent
  signed envelope contribution mapped to `(value + 32) / 64` by the coefficient
  path. Source: pinned OpenMPT Snd_flt.cpp CutOffToFrequency and Sndmix.cpp
  ProcessPitchFilterEnvelope (f83cedb0cd5446e4dfaa83ac97e3087107e26767).
- Shared filter-envelope reset initializes live/legacy positions, enablement,
  sustain/loop metadata, clears started bit8 and invalidates cached coefficients.
  Completed active ticks distinguish held tick0 from disabled-at-start. S7B
  pauses authored-on envelopes without losing their cutoff contribution; stopped
  at start uses the source's midpoint rule. NNA copies and silent advancement
  share state. Base Zxx/default cutoff changes remain independent.
- The S7B-start exact-control assertion FAILED with FFmpeg's embedded reference.
  Did not relax it: upstream fix c7474dfe87a9 (issue1914, 2025-08-31) documents
  this historical IT behavior. Official libopenmpt0.8.9+r25651 passes the original
  exact-control comparison, as does native. This newer reference is now used for
  filter modes only; other probes still use the previously recorded FFmpeg.
- Official reference: OpenMPT1.32.11.02-r25651, clean SVN tag libopenmpt-0.8.9,
  revision25651, source timestamp2026-08-19T07:15:55Z. Download:
  https://lib.openmpt.org/files/libopenmpt/bin/libopenmpt-0.8.9+release.bin.windows.zip
  Archive SHA256 d20c7589b323013ce55d2198fd55f1a0586e66b98c7cede59dea7ee97460eb79;
  amd64 executable SHA256 9d809056e40e3d004b1ab9275081ee8ddb9d1874999369189d0ba172e7c3131c.
  `filter_reference()` verifies hash/version and downloads the hash-pinned archive
  if missing, retaining only amd64 executable/DLL and licenses. No player swap in
  Nethercore: this is reference-only offline tooling. Current setup is Windows.
- Reference settings explicit:44100Hz, stereo2, signed16, dither0, gain0dB,
  stereo100%, interpolation8, ramping-1, repeat0, subsong-1, end0.7s. Source/cart
  regeneration plus restart and termination checks are retained. Probe command:
  `python tools/tracker-debug/compatibility.py --envelope-enable-probe`.
- Fresh filter sweep, held sustain-zero pause/resume and S7B-at-start midpoint
  control pass. Relative low/high attenuation native0.05984 vs official0.05776;
  mid/high native1.04596 vs official0.99945. These bounded spectral assertions
  do NOT establish the complete IT resonant filter's transfer function.
- Runtime443 passed/1ignored; CLI80 passed, helper checks and git diff --check
  passed. Cold reference download/hash/version setup was independently exercised
  in a disposable directory and removed afterward. New regression checks live initialization, base
  cutoff preservation, paused tick0, new cutoff commands, NNA/silent snapshot
  parity and removal of stale envelope state. Read-only review deleg_89524ea7
  found no concrete defect in scoped initialization, mapping, pause/start-disabled,
  cache invalidation and sustain/loop metadata. The reviewer did not rerun tests;
  owner-run results above remain the execution evidence.
  Full DSP/filter-active reset, carry, delayed/porta interactions,
  historical discrepancies, clipping, song control and complete rollback remain
  IN PROGRESS / NOT ACCEPTED. No commits, pushes or sibling asset modifications.

## Filter output-rate propagation (bounded)

- Re-read AGENTS/CLAUDE and inspected the dirty sole-main worktree; unrelated
  changes and existing IT panning repair preserved. Prior Lxx/pause review
  completions are recorded below, including the superseded pause review.
- Confirmed the shared audible/silent mixer passed no output rate to apply_filter;
  coefficient refresh hard-coded22050 despite real packed capture running44100.
  Focused regression failed before repair: b0 .013041992 vs .0033330235 at44100.
- apply_filter now receives the render rate and refreshes its coefficient cache
  when that rate changes. The cached rate belongs to cloned channel state, so
  snapshot/restore and background voices use the same coefficients. Regression
  exercises44100->22050->48000 and identical audible/silent filter history.
- Runtime441 passed/1ignored. Existing source->fresh cart->native/reference
  envelope checks pass. Added original filter_cutoff.it to the existing generator:
  Z00/Z40/Z7F on one held note. Normalized steady-window levels are native
  [.0598432,1.044577,1] vs reference [.0578189,.999350,1]; bounded attenuation
  assertions pass without modifying source audio or normalizing rendered files.
  These ratios compare cutoff behavior to each player's own open-filter control;
  they do not establish complete filter-response or historical-module fidelity.
- Fresh FilterEnvReset.it remains UNREVIEWED:5 activity-disagreement windows,
  pan MAE .7808595, crossing MAE8.39604. Independently confirmed next missing
  path: live row_processing explicitly excludes filter envelopes. The separate
  missing authored cutoff/resonance initialization is repaired below. Full filter
  DSP, envelope/reset/carry semantics and historical acceptance stay open.
- Read-only sample-rate/cached-state review deleg_3db238fe returned bounded PASS:
  render-rate propagation, cache invalidation, reset/clone and snapshot state.
  Overall IN PROGRESS / NOT ACCEPTED; no commits, pushes or sibling asset edits.

## IT live-note filter defaults (bounded)

- Pinned OpenMPT Snd_fx.cpp1766-1767 copies enabled instrument cutoff/resonance
  on the ordinary note path. Nethercore's legacy trigger_note did this, but its
  actual row-processing path did not. Existing shared filter module now owns the
  default application used by both paths; absent defaults preserve prior values.
- Original filter_defaults source fixture selects three instruments with cutoff
  0/64/127 on successive notes. Before repair, reference showed low/mid/open
  attenuation but freshly packed native stayed unfiltered across all three rows
  (window levels21093.678/21099.722/21087.632). The historical fixture was not edited.
- After repair fresh native normalized levels [.0598405,1.042605,1] match the
  bounded reference expectations [.0578133,1.001681,1]. Both command/default
  source fixtures pass existing packer->cart->mixer/reference and restart/end
  checks. No gain compensation was applied to either rendered audio file.
- New live-row regression covers absent/zero/half/full defaults, resonance and
  simultaneous Zxx override. Runtime442 passed/1ignored; CLI80 passed before
  this runtime-only addition. Reference-first probes explicitly recapture native
  afterward so reference WAVs cannot be mistaken for native output.
- Review deleg_81b942ad returned bounded PASS for shared default application,
  absent-value preservation and row-effect ordering. Reviewer Cargo execution
  hit MSVC LNK1104 on a malformed .hermes-tmp path; owner execution above remains
  the test evidence. Preceding rate review deleg_3db238fe also passed.
  Delayed notes, portamento, filter envelopes, full IT response,
  historical discrepancies and complete rollback remain unaccepted. Next source
  investigation is the live filter-envelope initialization/application path.

## XM Lxx panning and IT authored-on envelope pause (bounded)

- Re-read guidance and inspected dirty sole-main worktree; unrelated work preserved.
- Pinned OpenMPT Snd_fx.cpp3758-3771: XM Lxx sets volume position always, but
  sets pan/pitch only when volume sustain is enabled. Fresh flag-pair probe passed
  reference first, failed native right1830/left816 on expected-left case. Runtime
  condition repair passes both players; regression distinguishes volume from pan
  sustain flags and preserves frozen-pan state. Review deleg_9bcef21e returned bounded PASS.
- Pinned Sndmix.cpp1073-1095: IT S77/S79/S7B pause authored-on envelopes; they do
  not remove the envelope contribution. Original volume pause fixture reproduced
  incorrect gain252->505->252. Mixer now keeps authored-on volume/pan/pitch
  contribution while pausing positions. Authored-off enable/disable remains distinct.
- A pos>0 first attempt failed the tick-zero sustain boundary. Retained-started
  bits now mark completed active envelope ticks independently of position. All
  three note-reset paths clear them; snapshots/NNA clone them. A regression tests
  initially paused unity versus subsequently paused half-volume at held tick0,
  snapshot parity and new-note reset. Fresh volume/pan/pitch pause/resume at tick0
  all pass native/reference. Review deleg_a5535b46 was dispatched before the
  started-bit correction and is superseded; final current-state review
  deleg_3e3d1ed0 returned bounded PASS for bit/reset/snapshot semantics.
- Runtime440 pass/1ignored. Source->fresh cart->mixer/reference checks pass for
  these bounded semantics, and full fresh XM probes also pass. Fresh historical
  s77.it still has83 activity-disagreement windows/pan MAE0.659982; SetEnvPos.xm
  has3/pan MAE0.030576. Neither whole historical fixture is accepted by these
  aggregate diagnostics. Filters, carry, full timing/end behavior, historical
  discrepancies and complete rollback remain unaccepted; overall NOT ACCEPTED.

## XM reached panning-sustain freeze (bounded)

- Source: pinned Sndmix.cpp1280-1290 (kFT2PanSustainRelease). FT2 freezes an
  already reached panning sustain even after key-off; IT does not.
- Original fresh XM fixture showed reference late left/right peaks57/916 (held
  right) but native2048/0 (incorrectly traversed to left). Reference is not exactly
  hard-right because of envelope/ramp arithmetic; the directional probe requires
  >90% right, not bit-identical pan or fabricated silence on the left.
- A cloned channel flag freezes only panning advancement, preserving evaluated
  held pan. New-note live/trigger paths clear it. Focused regression covers early
  release, IT exclusion, later note reset and snapshot equivalence.
- Fresh source/cart/native/reference directional control passes after repair;
  runtime437 pass/1ignored and complete XM probe suite pass. Historical
  PanSustainRelease.xm renders, but still has3 activity-disagreement windows and
  pan MAE0.207878; the complete historical fixture is NOT accepted.
- Review deleg_e2b4d8fb reported two issues. Its unreachable-freeze claim is
  contradicted by the source condition and repeated passing control: XM sustain
  start=end, so previous==end and held position==start do hold without a loop.
  Its legacy delayed-note reset finding was confirmed: a frozen channel retained
  the flag after resetting pan position. Added the missing reset and regression.
  The regression failed before repair; after aligning its expected position with
  process_tick's existing completed-tick advancement, runtime438 pass/1ignored.
  Fresh packed/reference pan-freeze probe also passes after this one-line repair.
  Overall compatibility, full envelope interactions and rollback remain unaccepted.

## XM release-loop escape (bounded)

- Reconfirmed guidance/dirty sole-main worktree; unrelated changes preserved.
- Pinned OpenMPT Sndmix.cpp1263-1277 documents FT2 escape on key-off when normal
  loop end and sustain end are the same authored node. Prior runtime always looped.
- Existing XM probe generator now emits matched-node and different-node controls.
  Fresh reference passed; native failed before repair (late peak1379, expected<5).
  After node-identity flags and XM-only key-off suppression of normal looping,
  both fresh source/cart/native/reference controls pass.
- Volume/pan node identities retained in cloned channel state, not inferred from
  equal tick values. Regression includes distinct nodes with identical tick4,
  matched/mismatched node cases and snapshot restoration. IT path unchanged.
- Runtime436 pass/1ignored and full fresh XM probe suite pass. Historical
  EnvLoops.xm still fails capture at native peak1 with4 activity-disagreement
  windows; this historical fixture is NOT accepted. No clipping gate relaxed.
  Independent review deleg_cd476d61 returned bounded PASS and independently
  passed the node-identity regression and fresh packed/reference escape probe.
  Broader XM pan-sustain freeze, envelope carry/end handling, historical defects
  and complete rollback remain unaccepted. No overall acceptance claim.

## IT envelope sustain ranges and inclusive normal loops (bounded)

- Re-read guidance/worktree state; sole main checkout remains dirty. No unrelated
  work or pre-existing IT panning repair removed; no commits or asset edits.
- Live-note regression confirmed IT sustain range [1,3] froze at1 instead of
  visiting2/3. Channel snapshots now retain both sustain endpoints; source node
  indices resolve through TrackerEnvelope::sustain_ticks for volume/pan/pitch.
- Shared advancement gives held IT sustain precedence over normal loops and
  includes the end tick for both kinds. Key-off returns to the normal loop.
  Existing XM advancement is kept separate, not declared fully FT2-compatible.
- Source: pinned OpenMPT Sndmix.cpp IncrementEnvelopePosition lines1305-1335
  (EnvLoops.it/EnvOffLength.it semantics). Focused source/live-note regression
  failed before (held1 vs expected2), passed after. Boundary checks cover held
  single tick, end65535, normal-loop endpoint and existing XM exclusivity.
- Existing envelope fixture generator now includes an original volume_sustain
  module: held range and key-off into normal loop. Fresh CLI/runtime build,
  source->cart->mixer and reference assertions pass for recurring low/mid/high
  sustain cycles and inclusive normal-loop release. All earlier envelope probes
  pass. Initial probe construction accidentally emitted note0 for non-sustain
  controls; corrected to ItNote::default().note before verification, not a source
  asset change or assertion relaxation.
- Foreground/NNA state and snapshot-restored range/release are covered by focused
  runtime regression. Full song rollback, filter initialization, carry and FT2
  sustain quirks remain unaccepted. Runtime435 pass/1ignored; converter52 pass;
  complete fresh XM probe suite passes. Historical EnvLoops.it/EnvOffLength.it
  both render fresh; first still has3 activity-disagreement windows, second0.
  Aggregate metrics do not accept either historical fixture.
  Independent review deleg_1ac752b7 returned bounded PASS; no concrete regression
  found in sustain ranges, inclusive advancement, metadata or snapshot copying.
  Reviewer diff check passed; reviewer Cargo execution hit a malformed .hermes-tmp
  linker path, so passing owner runs above remain the execution evidence.

## IT pitch-envelope playback-rate repair (bounded)

- Reconfirmed two independent losses: the mixer evaluated pitch_envelope_value
  but sample_channel never used it; the real row note-start path initialized
  volume/pan envelopes but not pitch envelopes (trigger_note is not that path).
- Fresh original envelope probe failed before repair: native crossing rates
  [222.22,277.78,275.00] did not rise an octave. Direct foreground/NNA sample-rate
  regression also failed before repair. Neither historical fixture was edited.
- Live notes now initialize non-filter pitch envelope enable/position/sustain/loop;
  sampling applies a temporary linear-period offset without mutating base/slide
  period. Mixer clears stale envelope values when disabled/missing. Silent
  advancement shares the exact same sampling path.
- Source: pinned OpenMPT f83cedb0cd5446e4dfaa83ac97e3087107e26767,
  soundlib/Sndmix.cpp ProcessPitchFilterEnvelope (envelope*8, slide index cap255)
  and linear slide table semantics: one source envelope unit is half a semitone.
  URL: https://github.com/OpenMPT/openmpt/blob/f83cedb0cd5446e4dfaa83ac97e3087107e26767/soundlib/Sndmix.cpp
- `python tools/tracker-debug/compatibility.py --envelope-enable-probe` rebuilds
  CLI/runtime, writes fresh IT, packs and compares pitch rise/return with reference;
  passes together with S78/S7A and termination/restart checks. Runtime suite:
  433 passed / 1 ignored; converter52 passed after adjacent S7B/S7C repair.
  Independent source-bound review deleg_ed0087c6 returned bounded PASS for the
  pitch-envelope repair (live review transcript): half-semitone conversion/cap,
  live initialization, stale-value clearing, NNA and shared silent sampling path.
  No blanket filter/envelope compatibility acceptance is claimed.
- S7B/S7C no longer disappear during conversion. Foreground pitch/filter enable
  state toggles without resetting position or changing NNA voices. Mixer honors
  live enable state instead of rejecting authored-off envelopes. Source:
  pinned ModChannel.cpp InstrumentControl, cases0xB/0xC. Authored-off constant+24
  pitch, S7C enable, S7B disable fails before and passes after fresh packing in
  both players. Positive and negative octave rise/return also pass. Pitch probe
  measures distance between zero-crossing edges, avoiding window-count rounding.
  Filter command dispatch is covered; full audible filter fidelity is not.
- Additional current checks: IT62 unit tests plus1 doctest, CLI80, complete fresh
  XM generated probe suite, cargo check runtime library and git diff --check pass.
  Repacked SampleSustainAfterPortaInstrMode.it still has5 activity disagreement
  windows (pan MAE0.2779447248685186, crossing MAE4.841059602649007); no acceptance
  inferred from the diagnostic runner exit0.
- Still NOT ACCEPTED overall. Filter behavior, complete sustain
  ranges/carry, broad pitch-envelope interactions and complete rollback remain
  repair/verification obligations; this bounded probe does not cover them.

## XM multi-sample transport integrated (bounded verification)

- Source parser now retains 96-note local maps plus per-slot sample metadata;
  extractor retains original local indices, including gaps from empty PCM slots.
  Owner repaired the worker's incorrect delta-decoding expected value: deltas
  1,2,3 decode to 256,768,1536, not 256,768,768. No decoder assertion was removed.
- NCXM flag 0x04 carries map[96] followed by each sample's loop start/length
  (two LE u32), finetune, relative note, loop type, volume, pan (five bytes).
  This is additive: legacy NCXM without the flag remains supported. New mapped
  carts require the updated runtime; old runtimes reject unknown flag 0x04.
  Maps/counts, truncated extensions and sample volume are validated.
- Converter flattens all local slots; shared note sample numbers are u16 to avoid
  truncating XM module totals above 255. IT source u8 slots widen without change.
  CLI extraction identity uses instrument*256+local slot rather than names or
  compacted nonempty enumeration. Nonempty missing samples and extraction errors
  fail packing; empty slots remain silent; explicit named overrides remain valid.
- Runtime reuses mapped-sample resolution, preserving XM C-4 pitch convention,
  per-sample loops, volume/pan, and note-only retained volume. An unmapped XM note
  formerly aliased another sample; the new regression failed with handle9 rather
  than0, then passed after clearing the unmatched handle.
- Fresh paired source/cart/native/reference probes pass for two samples selected
  by distinct notes, distinct PCM/volume/pan, and empty first sample mapped to the
  second slot. Existing full XM probe suite also passes. NCXM roundtrip/truncation
  regression and >255-slot converter regression pass.
- Owner suites: 64 XM, 52 converter, 80 CLI, 431 runtime passed / 1 runtime ignored.
  Read-only integration review deleg_7facc6ad returned bounded PASS: no concrete
  regression found in slot identity, NCXM layout/bounds or runtime handoff. Its
  test statement relies on owner runs above, not an independent test execution.
  The delayed implementation report deleg_05c430e7 describes an earlier compile
  collision already reconciled by those owner runs; it is not a new failure.
  No blanket multisample or effect-interaction acceptance claimed.
- Fresh unmodified CC0 `napping on a cloud.xm` now packs beyond the former `perc`
  collision and reaches the native renderer, but fails clipping at peak1.8981972
  over the reference-derived249.008s capture. Historical song remains NOT ACCEPTED.
  Headroom, broader effects/portamento interactions, stereo, full song control
  and complete seeking/rollback remain open. No commits or asset edits.

## XM 16-bit loop units repaired

- Source inspection found both XM metadata readers exposing byte-based loop
  positions as decoded-sample positions. The converter then resampled incorrect
  loop boundaries. Pinned OpenMPT `XMTools.cpp:387-396`, commit
  `f83cedb0cd5446e4dfaa83ac97e3087107e26767`, divides loop start/end for 16-bit.
- Corrected parser and extractor metadata. Raw sample length remains bytes for
  payload consumption. Loop end is halved separately, preserving odd-offset
  truncation; u64 arithmetic avoids overflow while adding byte boundaries.
- Both focused regressions failed before: (3,7) versus expected (1,4); both pass
  after. XM suite: 61 passed, 4 ignored doctests; converter: 51 passed;
  runtime: 430 passed, 1 ignored.
- Extended existing `--xm-global-volume-probe` with paired 8/16-bit sources,
  nonzero loop start and forward/pingpong loops. Full fresh source/cart/native
  and reference suite passes. PCM equivalence is checked within each player,
  not across engines. Synthetic fixture is original; no historical asset edited.
- Read-only bounded review deleg_38e73cf0: PASS. Reviewer confirmed loop-unit
  semantics and focused tests, but its overall Cargo command failed creating a
  doctest temp directory. Owner reran with all six temp variables set to the
  valid Windows Temp directory: exit 0, 61 passed, 4 ignored doctests.
  This does not accept the
  still-missing multi-sample keymaps, stereo extensions, historical clipping,
  complete song control or full seeking/rollback reconstruction.

## XM high pattern indices and cache verification

- Re-read AGENTS.md/CLAUDE.md and inspected main/7dac9fb dirty checkout; only
  one worktree exists. Unrelated changes and IT pan repair remain intact.
- Confirmed completed row-cache payload restoration in sync.rs: channel mutes,
  delay state, global-volume memory, format modes and tempo slide now restore.
  Both cached-seek focused regressions pass. This is not full tick/sample replay.
- Reproduced XM order254 being treated as an IT skip marker: the new runtime
  regression failed before repair. Both TrackerModule::pattern_at_order and
  normalize_order now apply 254/255 marker semantics only to IT format.
- Runtime suite: 430 passed, 1 ignored; converter suite: 51 passed.
- Fresh synthetic XM controls differing only in pattern placement 0/254/255
  produce equal nonzero PCM independently in native and libopenmpt paths;
  freshly rebuilt packer/renderer, finite termination and restart checks pass.
  Probe is included in --xm-global-volume-probe via high_patterns=True.
- Full fresh XM probe suite rerun exits 0 with the new high-pattern case.
  PanSlideZero.xm rerun still fails at native peak 1.7576425; no gate weakened.
  Independent reference rerender peaks at PCM16 27365 with zero clipped samples.
  This localizes a real native headroom/output discrepancy, not shared clipping;
  it does not yet identify its root cause. Pinned OpenMPT Sndmix.cpp at
  f83cedb0cd5446e4dfaa83ac97e3087107e26767:2124-2158,2514-2516,2578-2580
  applies configured pre-amplification/attenuation absent from a raw sum; matching
  source gain semantics/settings needs investigation, not an arbitrary gain patch.
- Independent read-only review deleg_5648cc43: PASS for the high-pattern marker
  slice; no concrete implementation defect found. Its test execution hit LNK1104;
  owner subsequently reran high-pattern regression (1 passed) and cached-seek
  regressions (2 passed). Owner CLI suite: 80 passed; helper self-check passed.
  Review caveat: paired high-pattern audio checks establish marker preservation,
  not arbitrary pattern identity (target notes are identical across sources).
  The runtime regression distinguishes the target pattern from empty alternatives.
- Cache worker deleg_daed40fb completion reconciled against current state.rs,
  sync.rs and control_tests.rs: payload fields are stored/restored, and the
  later-loaded-module invariant is tested. This is bounded control-state
  restoration, not complete tick/sample-clock seeking or rollback acceptance.
  Historical clipping, multi-sample maps,
  broader song flow and complete uncached seeking/rollback remain unaccepted.

## Fresh packed D0 and XM suite passes

- Owner rebuilt and reran `python tools/tracker-debug/compatibility.py
  --xm-global-volume-probe`: exit 0. All named probes, including D0/E0 onset,
  settled panning, row reset and explicit-instrument portamento, pass through
  fresh source/cart/mixer/reference playback.
- Owner runtime suite: 429 passed, 1 ignored during cache integration. Cache
  changes still need final source review; not accepted merely from suite count.
- Historical PanSlideZero clipping and broad importer/playback/seek gaps remain.
  Overall IN PROGRESS / NOT ACCEPTED.

## D0 ramp-aware reference check

- Owner successfully rendered exact source SHA ac82db20489ec9ef1aab3255cbcd8e86a855d996e590d404397f2e92f95f13c6
  with the verified absolute FFmpeg executable. Source declares FT2 metadata;
  pinned Load_xm.cpp enables kFT2VolumeRamping for FT2 version >=1.04.
- Reference left shares: tick0 center 0.5; tick1 transition 0.916755559144309;
  settled at .18-.19 seconds 1.0; next-row explicit center 0.5. Earlier centered
  reports are superseded for these exact current source/binary bytes.
- Replaced incorrect instantaneous hard-left PCM requirement with center-before,
  onset >.75, settled >.98, E0 no-op and next-row reset assertions. Native D0
  tick-state regression remains unchanged. Standalone reference check passes.
- Full fresh source/cart rerun temporarily awaits cache worker completing its
  CachedControlState API integration; no fresh full-suite pass claimed yet.

## D0 contradictory observation resolved (not playback acceptance)

- Owner inspected diagnostic proc_2da5024ecee8: ffmpeg launch raised
  FileNotFoundError before reference rendering. Its retained audio.wav with
  left_share 0.916755559144309 was the native render, not reference evidence.
  Worker was steered to use the absolute executable path and correct XM offsets.
- Owner's actual subprocess resolves FFmpeg 8.1.2 with SHA-256
  ad8f211bc894755e0061c55ab280ae00e8d3d4f15a8cc4372b24cfa247b5942e.
  Its D0 reference assertion still fails. The mislabeled native WAV cannot be
  used to weaken or replace that result. No claim of D0 resolution.

## Row-cache control-state repair in progress

- Caller inspection confirms row-cache entries currently preserve only channels
  and global volume; engine-level timing/effect-memory fields are omitted.
- deleg_daed40fb owns a bounded fail-before/pass-after repair of that payload,
  preserving the pre-row boundary and invalid-target guard. A cache restore must
  not replace the current module registry or drop newly loaded modules.
- Full tick/sample-clock replay is still separate and remains unaccepted.
  No worker completion or new passing count is claimed at dispatch.

## Cached seek row-boundary repair

- Independently reproduced a cached seek replaying tick-zero fine volume twice:
  fresh target volume 0.015625, repeated cached target 0.03125. The cache stored
  post-row channels under the same row that replay executes again.
- Moved cache capture before processing the indexed row, matching replay's
  boundary convention. The fresh-versus-cached regression fails before and
  passes afterward. Runtime428 pass / 1 ignored; git diff --check passes.
- This does not repair the cache's incomplete engine/timing payload or the
  tick-zero-only uncached replay. Full seeking/rollback acceptance remains open.
  No full packed-playback acceptance is inferred from this state regression.

## Invalid seek termination repair

- Independently reproduced seek(order beyond song) hanging in the old wrapping
  replay loop. Added a bounded invalid-order/invalid-row regression; it timed
  out before the guard and passes afterward.
- Reject invalid target coordinates before replay while preserving the initial
  (0,0) reset path, including header-only IT modules. Initial guard broke that
  existing IT reset test; repaired it and reran the full runtime package:
  427 passed, 1 ignored.
- This is a termination guard, not faithful seek/rollback reconstruction:
  row-cache completeness, mid-row effects/sample clocks and song-control replay
  remain repair obligations. No full seek acceptance claimed. D0 investigation
  remains separately active. Overall NOT ACCEPTED.

## D0 exact-reproduction isolation

- Owner verified the authored XM bytes remain unchanged across native packing;
  packer source mutation is not the cause of the reference disagreement.
- Nonzero DF/EF diagnostic trials produce directional reference motion, so this
  is not a blanket stereo-output failure. No timing/amplitude acceptance follows.
- Narrow retry deleg_7deed673 owns only the exact D0 helper reproduction and a
  proven diagnostic repair, if any. It must preserve assertions and historical
  sources. Earlier ad-hoc reviewer reproduction does not supersede the failing
  exact callable. Overall NOT ACCEPTED.

## Independent review adjudication and surround repair

- deleg_509b1d40's extended-header finding is not present in the inspected
  source: parser/read.rs seeks to the instrument header end and skips extra
  sample-header bytes. New 263/48-byte sentinel-padding regression passes with
  no parser implementation change and preserves the following instrument.
- Its portamento-pan finding is also contradicted by pinned Snd_fx.cpp:
  kFT2PortaIgnoreInstr retains pIns/pSmp before ApplyInstrumentPanning. A fresh
  two-instrument source/cart/reference probe confirms retaining the old pan.
  Owner's trial selecting the explicit new pan failed reference and was reverted.
- Confirmed surround reset defect: ApplyInstrumentPanning clears surround when
  default pan applies under kPanOverride. Added the missing XM channel reset.
  Runtime regression fails without that line and passes with it, preserving old
  sample identity/pan during portamento. XM59 and runtime426/1ignored pass;
  fresh basic-pan/override and two-instrument portamento probes pass.
- This does not accept all surround-mode flags, multi-sample behavior or D0.
  D0 binary/source/reproduction disagreement remains unresolved; review's
  version-mismatch diagnosis alone is insufficient. Overall NOT ACCEPTED.

## Established PanSlideZero fixture follow-up

- Fresh unmodified PanSlideZero.xm source/cart/mixer run fails native clipping:
  renderer peak 1.7576425. Source SHA256
  9e83f426eb4a12fa94184b7eafdfe199fd738651f08223635550da43fc0e8305.
  Activity disagreements5, pan MAE0.2771026232665006; FAILED, not acceptance.
- Synthesized D0 probe identifies as FastTracker 2 or compatible in ffprobe.
  Disposable in-memory checks forcing stereo input, two channels, and a 263-byte
  instrument header do not resolve centered reference output. These variants
  were not retained or used to weaken the failing acceptance assertion.
- Source PanningSlide additionally shows potential regular-slide scaling mismatch
  (FT2 one unit in 0..256 versus runtime one unit in -1..1 divided by255);
  independently verify with paired command probes before repairing.

## XM panning integrated rerun

- Added runtime regression for sample pan 0/32/64, explicit row override and
  invalid-instrument preservation; passes. Full runtime: 425 passed, 1 ignored.
- Rebuilt and reran the combined fresh source/cart/native/reference probe:
  Gxx, ECx, Kxx, note97, fade4096, EDx, ED3+F1, empty ED3, delayed volume
  slide, sample volume, 60/70 and sample panning all pass their bounded checks.
  Combined command still exits nonzero at D0/reference tick1; not a suite pass.
- Fresh GlobalVolume.xm still has 5 activity-disagreement windows, pan MAE
  0.16215192172471157 and zero-crossing MAE 5.090909090909091; UNREVIEWED,
  not an audible acceptance. Diff whitespace check passed before this doc edit.
- No overall acceptance; independent D0 reference diagnosis remains pending.

## XM panning owner integration checkpoint

- No live pan implementation worker remained on recovery. Owner completed the
  missing XM explicit-instrument default-pan runtime application before row
  effects, preserving the IT-specific default-pan branch. Fresh source/cart/
  mixer/reference sample pans 0/128/255 and explicit override now pass.
- Fixed worker NCXM tests inspecting byte15 (reserved) instead of byte13 (flags).
  Owner package checks: XM58, converter51, runtime424 pass / 1 ignored.
- D0 focused runtime tick0/tick1/reset passes, but the fresh reference remains
  centered where native goes left. Pinned Snd_fx.cpp specifies hard left under
  kFT2VolColMemory. Authored probe now declares FT2 space-padded metadata per
  Load_xm.cpp; disagreement persists. No D0 packed acceptance is claimed.
- Independent source review/oracle diagnosis deleg_509b1d40 is active. Multi-
  sample mapping, tracker-identity compatibility flags and other inventory gaps
  remain open. Overall IN PROGRESS / NOT ACCEPTED.

## XM D0/E0 volume-column panning in progress

- Pinned kFT2VolColMemory gives D0 a special hard-left action on non-row ticks;
  E0 is ignored. Converter previously emitted zero panning slides for both.
  Added explicit PanningLeftOnTicks state, per-row reset and tick dispatch.
- Conversion regression failed before; runtime tick0/tick1/reset regression and
  fresh packed D0/E0 probe added. Verification is pending: pan worker's current
  XmInstrument extension temporarily lacks a minimal-parser constructor field.
  No runtime or packed pass claimed for this slice. Overall NOT ACCEPTED.

## XM default-pan follow-up in progress

- Sample-volume worker deleg_dadde447 reached iteration cap after implementation;
  owner verified source handoff, fresh packed probes, XM54/converter49/runtime423
  (1 ignored)/CLI80 tests. Its cap is not a remaining build blocker.
- Re-ran GlobalVolume.xm fresh: clipping gate no longer fails, but 5 activity
  disagreements remain, pan MAE .7640860670621493 and crossing MAE5.090909090909091.
  Reference duration .9375s; this remains UNREVIEWED, not playback acceptance.
- Confirmed next root cause: source XM parser skips sample pan. New authored
  sample-pan0/128/255 + explicit128 override probe passes reference then fails
  native at default pan0. Existing sample-volume/zero-slide groups still pass.
- Worker deleg_73f990f5 owns bounded single-sample pan handoff/legacy format tests;
  owner owns packed regression. Multi-sample mapping remains open. NOT ACCEPTED.

## XM sample-volume and zero-slide integration checkpoint

- Owner freshly rebuilt/packed/rendered all existing probe groups plus authored
  sample volumes0/32/64 and explicit override: native and reference now pass.
  Zero-column60/70 sources match no-command control independently in both players.
- Zero-slide focused test fails with prior conversion restored and passes with
  repair. Owner package checks: XM54, converter49, runtime423 pass/1 ignored.
  Initial XM parser-test syntax error was repaired by worker; rerun passes.
- Owner inspected parser bounds, reserved-byte write, minimal decode and runtime
  default-volume handoff. Multi-sample volume/keymaps remain unresolved.
  Independent bounded source review deleg_fb365541 reports a pass for zero
  preservation, legacy encoding, bounds, complete handoff, explicit overrides,
  IT isolation and constructor coverage; also reports 60/70 handling consistent
  with reference memory separation while main A00 remains unchanged. No new tests
  were run by that source-only review. Multi-sample behavior remains open.
  Overall NOT ACCEPTED.

## XM zero volume-column slide repair in progress

- Pinned kFT2VolColMemory discards volume-column 60/70, unlike main A00 recall.
  Converter previously emitted VolumeSlide(0,0), recalling shared memory.
  Converter now returns no effect for 60/70; main A00 remains unchanged.
- Added focused conversion regression and fresh paired 60/70/no-command probe
  after an active slide. Neither is accepted yet: compilation encountered the
  sample-volume worker's intermediate missing-field constructor updates.
  Worker integration remains active; no test-pass claim for this slice.

## XM sample-default volume repair in progress

- Fresh source/cart/mixer regression added to --xm-global-volume-probe for
  sample volumes 0/32/64 followed by explicit volume32 override. Reference is
  evaluated first and passes; native fails with identical initial PCM peaks
  [2757,2757,2757], independently confirming loss of authored sample volume.
- Source parser skips this byte; minimal metadata and shared instrument/runtime
  handoff must retain it. Worker deleg_dadde447 owns bounded implementation and
  package regressions; owner owns packed probe and integration. No pass claimed.
- Helper direct-run checks pass. Full compatibility remains NOT ACCEPTED.

## XM EDx regular volume-slide timing checkpoint

- Confirmed reviewer interaction concern for regular volume slides: pinned
  kFT2VolColDelay executes before the delayed note, skips tick0 and the delayed
  instrument-trigger tick. Deferral formerly suppressed pretrigger slides and
  applied one on the trigger. Activate slide on deferral; skip that trigger tick.
  One channel loop now handles deferred dispatch followed by tick effects.
- Focused regression failed before/passes after. Fresh packed paired sources
  check gain before/at/after ED3 in both players; late-tick windows allow reference
  ramps (absolute gain-ratio tolerance .06). Authored slide fixture uses sample
  volume64 to isolate timing. Other existing sources are unchanged.
- Investigation also exposed XM sample-default volume loss: native reloads full
  volume where authored sample volume32 stays half in reference. This is an OPEN
  repair obligation, not covered by the timing pass. Other volume effects remain
  open as well. Temporary byte-level diagnostic was replaced with PCM16 asserts.
- Runtime422 pass/1 ignored; fresh --xm-global-volume-probe all checks pass.
  Independent bounded review deleg_be32c364 reports source-bound agreement
  with pinned kFT2VolColDelay, including pending-state capture before dispatch;
  1 focused test passed, 0 failed, 422 filtered. Scope is regular nonzero
  volume-column slides only; other volume effects and sample-default volume
  remain outside this review. Overall NOT ACCEPTED.

## XM empty-cell EDx checkpoint

- Pinned FT2 delay branch substitutes the previous note for NOTE_NONE. Owner
  reproduced empty ED3 failing to retrigger. Reused deferred-cell dispatch for
  empty XM cells and substitutes current_note only when the delayed tick fires.
- Focused regression fails before/passes after. Fresh empty ED3 and explicit
  repeated-pitch ED3 sources yield equal PCM independently in native/libopenmpt;
  packed termination/restart and existing Gxx/ECx/Kxx/note97/fade/delay probes pass.
- Initial full run: 420 passed, 1 failed, 1 ignored; previously recorded unrelated
  replay::tests::wasm_timeout_is_an_error_report failed. Isolated replay rerun
  passed; full rerun passed 421 tests, 1 ignored. Intermittent failure retained
  here rather than erased. Independent bounded review deleg_b6842b5c reports
  source-bound agreement for original-note identity, no-previous-note boundary,
  and instrument handling; 1 focused test passed, 0 failed. Packed PCM comparison
  remains owner evidence, not independently rerun. Overall NOT ACCEPTED.

## XM EDx plus volume-portamento checkpoint

- Pinned FT2 delayed-note branch sets bPorta=false. Owner regression reproduced
  the delayed dispatch incorrectly preserving volume-column Fx portamento.
  Deferred pitched-note dispatch now removes that command before existing note
  processing, producing a fresh note. Other format/effect paths are unchanged.
- Regression fails before/passes after for note change and sample-position reset.
  Fresh ED3+F1 versus ED3 control sources yield identical PCM within Nethercore
  and independently within libopenmpt (not cross-player PCM identity), with
  packed termination/restart. Existing --xm-global-volume-probe cases also pass.
- Runtime420 pass/1 ignored; diff check passes. Independent bounded review
  deleg_c69800ed reports source-bound agreement with pinned bPorta=false and
  1 focused test passed (0 failed, 420 filtered); packed comparison remains owner
  evidence, not independently rerun. Broader delay interactions remain open.
  Overall IN PROGRESS / NOT ACCEPTED.

## XM pitched-note EDx checkpoint

- Owner confirmed pitched XM EDx triggered immediately, then restarted at the
  delayed tick. Pinned FT2 triggerNote defers the cell and ignores delays >= speed.
  Retain full note/handle in channel state before any note/instrument/volume
  mutation; dispatch once at requested tick through existing note processing.
  Channel snapshots clone pending state; row reset clears it. No engine replacement.
- Regression failed before/pass after, including ED1/ED3/ED6, deferred volume,
  and snapshot restore. Fresh packed source/native/libopenmpt ED1/ED3 stay silent
  until the scheduled trigger; ED6 ignores the note and next row plays normally.
  Termination/restart checked; existing Gxx/ECx/Kxx/note97/fade probes pass.
- Runtime419 pass/1 ignored, helper direct-run and diff checks pass. Independent
  bounded review deleg_be0a3cb3 found no local dispatch/reset/snapshot defect
  and ran 1 focused test (0 failed, 419 filtered). Its upstream fetch returned
  no body. Follow-up deleg_639c0d8b retrieved pinned source via urllib and confirms
  pitched trigger timing: tick == delay and delay < speed, with ED0 immediate.
  It did not rerun tests. Its local snippets predate the empty-cell extension.
  It flags whole-cell early return versus upstream continuation of generic effect
  processing as an unresolved interaction concern, not a demonstrated timing
  defect. Source-bound trigger-gate review is complete; full effect ordering is not.
  IT SDx and XM delayed note-off/empty-cell boundaries,
  extended-delay interactions and full rollback reconstruction remain open.
  Overall IN PROGRESS / NOT ACCEPTED.

## XM fade-rate normalization checkpoint

- Owner confirmed pinned XMTools.cpp preserves volFade while Sndmix.cpp
  ProcessInstrumentFade subtracts twice that value from a 65536 accumulator.
  Nethercore subtracted the raw XM value from 65535, doubling release duration.
  Converter now saturating-multiplies XM fadeout by two; IT conversion unchanged.
- Fresh packed authored fade4096 probe failed before (native release too long)
  and passes after independently in native/libopenmpt, with audible release then
  retirement and packed termination/restart. Existing Gxx/ECx/Kxx/note97 pass.
  Reuses --xm-global-volume-probe. Boundary conversion includes zero and saturation.
- Converter48 pass; runtime418 pass/1 ignored; diff check passes. Independent
  bounded review deleg_6ef30785 reports a pass against pinned XMTools.cpp and
  Sndmix.cpp, tracing the converted rate through runtime subtraction. Focused
  test: 1 passed, 0 failed, 47 filtered. Saturated rates still retire on the first
  runtime tick. This closes the rate-scaling review only. Full release/envelope/loop semantics
  remain open. Overall IN PROGRESS / NOT ACCEPTED.

## XM immediate note97 checkpoint

- Pinned Snd_fx.cpp FT2 NOTE_KEYOFF branch distinguishes volume-setting commands
  from other volume-column effects. Owner confirmed note97 without envelope was
  not muted. It now mutes and starts fade when neither instrument nor a volume
  setting is present; instrument/Cxx/volume-column volume retain release instead.
  Pan commands do not exempt note97, unlike K00. Existing IT path is unchanged.
- Five-context regression failed before and passes after. Fresh source/cart/native
  and libopenmpt probe agrees for blank, instrument, pan, volume-column volume,
  main Cxx contexts, including later volume restoration and termination/restart.
  Reuses --xm-global-volume-probe; Gxx/ECx/Kxx checks also pass.
- Runtime418 pass/1 ignored; helper direct-run passes and git diff --check passes.
  Independent bounded review deleg_669ce757 reports a pass against pinned FT2
  NOTE_KEYOFF handling and caller/effect ordering, with 1 focused test passed,
  0 failed. No concrete defect found in immediate no-envelope scope. Delayed note-off, nonzero
  fade-rate and full envelope release remain open, not inferred from these checks.
  Overall IN PROGRESS / NOT ACCEPTED.

## XM immediate K00 checkpoint

- Owner confirmed pinned CMD_KEYOFF: no-envelope K00 mutes unless an instrument
  or recognized volume-column command is present, in which case it starts fade.
  Row dispatch retains that context; ordinary KeyOff dispatch now mutes XM only.
  IT behavior remains unchanged. Existing fade/key-off state is reused.
- xm_k00_mutes_or_fades_with_row_context failed before and passes after across
  envelope on/off and empty/instrument/volume-command contexts. Runtime417 pass,
  1 ignored; git diff --check passes.
- Existing --xm-global-volume-probe now freshly packs K00, K01, K03 and K00 with
  instrument/pan-volume column; native and libopenmpt agree on immediate mute,
  delayed mute, zero-rate fade preservation and later volume restoration.
  All sources verify termination/restart; Gxx and ECx probes also pass.
- Independent bounded K00 review deleg_93001ebf reports a pass against pinned
  CMD_KEYOFF, including row-context dispatch and volume-column ordering; focused
  test 1 passed, 0 failed, 417 filtered at its checkpoint. No K00 defect found.
  This review excludes the separately repaired note97 path. Note97, full fade-rate,
  loop release and envelope interactions remain outside this slice, not accepted.
  Overall IN PROGRESS / NOT ACCEPTED.

## XM delayed key-off checkpoint

- XM Kxx conversion discarded all tick parameters. Added KeyOffAt for nonzero
  parameters, reusing existing per-row key_off_tick and reset. No-envelope XM
  delayed release also mutes volume, as pinned CMD_KEYOFF specifies.
- All 255 nonzero conversions tested (failed before). Runtime verifies delayed
  release with/without envelope and row scheduler reset. Runtime416 pass/1 ignored,
  converter48 pass. Immediate K00 semantics remain a separate open obligation.
- Existing --xm-global-volume-probe includes fresh K01/K03 sources, showing audio
  before release, silence after release and later volume restoration, independently
  in native/reference. Packed termination/restart and existing Gxx/ECx pass.
- Independent bounded review deleg_8544c04d reports nonzero timing and
  no-envelope behavior aligned with pinned CMD_KEYOFF; focused runtime check:
  1 passed, 0 failed, 416 filtered. Review explicitly excludes K00 and broader
  release semantics. It flags the already-open K00 path: immediate KeyOff only
  sets key_off, missing no-envelope muting and the instrument/volume-column
  fade special case. This finding requires owner confirmation and repair;
  nonzero timing review is complete, not full key-off acceptance.
  Overall NOT ACCEPTED.

## XM ECx mute-and-restore checkpoint

- Pinned ExtendedMODCommands routes ECx to NoteCut(cutSample=false): volume
  becomes zero, playback stays active. EC0 executes on tick0. Repaired ignored
  EC0 and delayed XM cuts incorrectly stopping the sample; IT stop behavior stays.
- Runtime regression failed before/passes after for EC0/EC1/EC3 and later volume
  restoration. Corrected IT test setup to actually initialize the IT format flag.
- Existing --xm-global-volume-probe now additionally verifies three fresh ECx
  sources: audible before cut, silent after, audible after C20 without a new note,
  independently in native/reference, with termination/restart. Gxx probes pass.
- Runtime415 pass/1 ignored; diff check pass. Bounded review deleg_3a77ea04
  reports alignment with pinned OpenMPT ECx/NoteCut(false), confirms format
  initialization before row effects, and reports 1 focused test passed (0 failed,
  415 filtered). This closes the pending ECx review only, not broader playback
  acceptance. Overall IN PROGRESS / NOT ACCEPTED.

## IT SC0 tick-one cut checkpoint

- Pinned ExtendedS3MCommands explicitly translates IT SC0 to SC1. Converter
  formerly emitted NoteCut(0), which runtime ignored. Clamp IT cut tick to >=1;
  XM conversion unchanged. Integrated 0..15 regression failed before/passes after.
- --note-fade-probe now includes SC0: fresh native/reference remain audible during
  row tick0 (.125-.135s) and silent after tick1 (.16-.18s), with termination/restart.
  Existing direct fade and empty-map probes still pass.
- Runtime414 pass/1 ignored, converter47 pass, diff check pass. Independent
  bounded review deleg_d2769765 passed IT conversion and scheduled tick cut
  against pinned OpenMPT, with 1 passing focused test; XM remains unchanged.
  Overall IN PROGRESS / NOT ACCEPTED.

## IT empty note-map slot checkpoint

- Confirmed actual empty slot (sample0) wrongly retriggered/changed the old voice.
  Ignore its note/instrument before mutations; co-located effects/volume remain.
  Actual populated regression failed with guard disabled; passes with guard.
- Extended --note-fade-probe with source sample0 mapping: native gain peaks
  [505,505,505,505], reference [448,448,448,448], stable pitch crossings in both.
  Direct fade companion still decays correctly; fresh cart termination/restart pass.
- Runtime413 pass/1 ignored; helper and diff checks pass. Independent bounded
  review deleg_dced7ef4 passed the ordinary empty-slot case (1 focused test),
  including no NNA displacement and preserved volume/main effects. It flagged
  empty-slot plus tone-portamento as unverified: clearing the note suppresses
  target selection. Owner resolved this bounded concern against NoteChange:
  it returns for empty slots before pitch target selection. Added G10 empty-slot
  fresh source/cart probes with LINK_G_MEMORY both off/on: gain/pitch remain
  unchanged independently in native/reference; termination/restart pass. Unit
  check preserves existing target and sample position while activating Gxx.
  No runtime change needed. Instrument-only and invalid slots
  also remain unverified; broad compatibility IN PROGRESS / NOT ACCEPTED.

## IT S70-S72 past-note control checkpoint

- Confirmed pinned ExtendedS3MCommands case0x70 selects background voices owned
  by the pattern channel; 0 cuts, 1 releases key, 2 starts fade without key-off.
  Added PastNoteAction conversion and dispatch reusing apply_dca; no new engine.
- Integrated conversion/dispatch regression failed before; all three actions now
  preserve foreground and other-parent voices. Runtime412 pass/1 ignored.
- Existing fresh DCT probe includes S70 after an instrument switch: old right
  background is cut while left foreground persists, in native and reference.
  Five variants pass fresh packed playback and finite termination/restart.
  S71/S72 full audible release semantics remain unaccepted, not inferred from S70.
- Converter47 pass. Independent bounded review deleg_71ce1a29 passed conversion,
  tick-zero dispatch and parent ownership against pinned OpenMPT; reviewer ran
  1 passing focused test. Broad key-off/release behavior remains outside this pass.
  Overall IN PROGRESS / NOT ACCEPTED.

## DCT source-sample identity checkpoint

- Confirmed CheckNNA compares source sample object identity, not PCM identity.
  Added incoming source sample_index to engine DCT selection, requiring a present
  matching index for Sample mode. Existing per-channel index and snapshots reused.
- Same handle/different source-index regression failed before and passes after;
  only the matching source voice is cut. Runtime411 pass/1 ignored.
- Extended existing DCT fixture with two byte-identical samples mapped within one
  instrument; old source-sample right background persists when incoming left
  notes use the other source. All four fresh packed/reference variants pass,
  including termination/restart. Repeat --dct-parent-probe.
- Helper and diff checks pass. Review deleg_27db851d confirmed mapped-index
  plumbing and ran 1 passing focused test, but proposed fallback to the current
  sample for missing maps. Owner source inspection rejects that as unproven:
  pinned CheckNNA bypasses duplicate checking without instruments and returns
  for empty mapped slots under kITEmptyNoteMapSlot. Falling back could falsely
  kill a prior voice. No fallback added. Broader empty-slot behavior remains open.
  DCT order and per-voice policies remain open; overall IN PROGRESS / NOT ACCEPTED.

## DCT sample instrument-match checkpoint

- Pinned CheckNNA Sample criterion with standard kITDCTBehaviour requires the
  incoming instrument to match. Added missing instrument equality to predicate;
  regression failed before and passes after. Corrected old contrary unit assertion.
- Existing DCT fresh probe now additionally exercises Sample mode across two
  instruments sharing PCM. Other-instrument right background survives matching
  incoming sample on left, independently in packed Nethercore and libopenmpt.
  Parent/Note/Sample variants pass with finite termination/restart checks.
- Runtime410 pass/1 ignored; helper and diff checks pass. Bounded independent
  review deleg_418a5361 passed the instrument guard against pinned OpenMPT,
  with 1 passing focused test. Corrected stale predicate documentation.
  This review did not cover source sample identity versus deduped audio
  handles, DCT order and per-voice policies remain open; NOT ACCEPTED overall.

## DCT note instrument-match checkpoint

- Pinned CheckNNA Note criterion requires matching instrument as well as note.
  Runtime predicate omitted instrument; new regression failed before the repair.
  Added instrument equality and corrected an old test that asserted the opposite.
- Extended existing dct_parent_probe with same-parent/different-instrument notes
  sharing identical PCM. Fresh packed/reference playback preserves the right-side
  old-instrument voice after matching-pitch left-side incoming instrument.
  Both parent/instrument variants pass termination/restart through existing capture.
- Repeat: python tools/tracker-debug/compatibility.py --dct-parent-probe.
- Independent bounded review deleg_b130877c passed note/instrument matching,
  caller input and preserved background instrument identity against pinned
  OpenMPT; reviewer ran 1 passing focused test. DCT sample identity,
  ordering and per-voice policy are not accepted. Overall IN PROGRESS.

## Direct IT pattern NoteFade checkpoint

- Confirmed missing row dispatch; regression failed before (note_fade false).
  Instrument-mode fade now sets foreground note_fade without releasing key;
  sample mode ignores it and co-located effects still execute.
- Fresh reference probe exposed a second root cause: source NOTE_FADE was 253,
  which pinned OpenMPT Load_it.cpp treats as absent in IT (MPTM differs).
  Corrected source constant to 246 / 0xF6, matching its IT writer. Unified tracker
  note remains 253. Arbitrary invalid source notes remain incompletely covered.
- Repeat: python tools/tracker-debug/compatibility.py --note-fade-probe.
  Fresh source/cart/native/reference gain peaks at .06/.5/1/1.6 seconds:
  native [505,355,158,0], reference [448,314,138,0]. Decay, finite termination
  and restart pass; this is not whole-engine audio fidelity acceptance.
- Runtime409 pass/1 ignored, IT62 and converter47 pass; diff check passed.
  Independent review deleg_86ef5235 passed direct 0xF6 encoding/conversion,
  instrument-mode dispatch, callers and tick/mixer consumption against pinned
  OpenMPT; reviewer ran 1 passing focused test. Generic invalid-note handling
  remains outside this pass. Overall IN PROGRESS / NOT ACCEPTED.

## DCT parent-isolation checkpoint

- Directly verified pinned OpenMPT CheckNNA restricts duplicate actions to the
  same pattern channel or its background voices (nMasterChn == nChn + 1).
  Nethercore previously scanned every background voice, cutting unrelated parts.
- Added parent identity to the existing process_duplicate_check helper; its sole
  production caller supplies ch_idx. Background identity/parent guard prevents
  another pattern channel's voices being cut, released or faded.
- New focused regression failed before (other parent's voice was cut), passed
  after. Full runtime: 408 passed / 1 ignored. Helper checks and diff check pass.
- Repeat: python tools/tracker-debug/compatibility.py --dct-parent-probe.
  Original generated two-channel fixture shares instrument/note across parents;
  after a same-note arrival left, unrelated right background remains audible.
  Fresh source -> packer/cart -> native and libopenmpt checks passed, including
  native termination/restart. No historical fixtures/assets were altered.
- Independent bounded review deleg_b20f2c5a passed parent isolation and the sole
  caller against pinned OpenMPT CheckNNA; reviewer ran 1 passing focused test.
  This only repairs parent isolation: DCT pre-NNA ordering, instrument/sample identity and per-voice
  criteria still need repair/verification. Overall IN PROGRESS / NOT ACCEPTED.

## IT NNA ownership and override checkpoint

- Independently confirmed pinned OpenMPT
  f83cedb0cd5446e4dfaa83ac97e3087107e26767 `Snd_fx.cpp:CheckNNA` switches on
  `srcChn.nNNA`; `NoteChange` installs the next instrument default afterward.
  Nethercore instead used the incoming instrument to dispose of the old voice.
  The row path now uses live old-voice NNA and then installs incoming defaults.
  Existing channel snapshots already carry nna. DCT/DCA remains separate/unaccepted.
- New actual-note regression failed before Continue→Cut (no background voice),
  passes both directions after. Corrected two pre-existing tests/comments that
  explicitly asserted the contrary behavior; NoteFade setup now specifies the
  displaced voice's action rather than relying on incoming defaults.
- Added IT S73-S76 conversion/runtime handling to set foreground NNA 0..3.
  Pinned `ModChannel.cpp:InstrumentControl` confirms mapping. Integrated all-four
  conversion/dispatch test failed with old converter, passes after; background
  voices remain unchanged. New notes reinstall instrument defaults before effects.
- Repeat: `python tools/tracker-debug/compatibility.py --nna-switch-probe`.
  Two original stereo fixtures, then two variants with opposing instrument
  defaults overridden by S73/S74, pass in fresh packed Nethercore AND libopenmpt.
  Old voice hard-left persists only for Continue; incoming voice hard-right plays
  in both cases. Finite termination/restart checks pass. Temporary output removed.
  S75/S76 conversion/state are tested; broader release/fade semantics remain open.
- Runtime405 pass/1 ignored; CLI80/helper2 pass in this slice; rustfmt/diff checks
  pass. Earlier authored-off envelope and default-pan fresh probes still pass.
  SwapNNA.it and CarryNNA.it were freshly captured; sample equivalence true within
  both players, activity disagreements1/5 respectively, not acceptance.
- Source review `deleg_e765486a` passed ownership/caller analysis. Its focused
  test attempts hit Windows link.exe errors; owner reran the exact regression
  successfully (1 passed), in addition to the full runtime suite. S73-S76 review
  `deleg_4cab0e92` checked ordering but missed the external source and ran zero
  tests; it is NOT a completed pass. Retry `deleg_fd56deab` ran 1 passing test,
  but incorrectly attributed adjacent PlayControl cases to InstrumentControl;
  its source comparison is rejected. Owner fetched the pinned ModChannel.cpp
  directly and verified InstrumentControl cases 3/4/5/6 are respectively
  NoteCut/Continue/NoteOff/NoteFade (ping-pong/offset cases belong to PlayControl).
  Local converter/handler match; owner reran the focused test: 1 passed.
  Independent source verification of S73-S76 remains incomplete.
  Overall IN PROGRESS / NOT ACCEPTED.
- NoteFade repair: directly rechecked pinned CheckNNA NNA/DCA branches: both
  set CHN_NOTEFADE without key release. Added cloned channel note_fade state;
  NNA/DCA fade preserves key_off and authored rate, including zero. Tick gain
  and rendered gain now honor explicit fade. New-note, delayed-note and reset
  paths clear it; NNA and channel snapshots retain it.
- New note_fade_keeps_key_and_authored_rate regression failed before repair
  (fade released sustain), passed after. Corrected older contrary expectations.
  Tick regression covers zero/nonzero rate, sustained envelope position, clone
  and new-note reset. Runtime: 407 passed / 1 ignored; diff check passed.
- Extended existing --nna-switch-probe with generated zero-rate NoteFade fixture;
  fresh source -> packed cart -> native/reference stereo checks passed, along
  with prior defaults and S73/S74 variants, termination and restart assertions.
  This does not establish nonzero-rate audible timing or complete DCT fidelity.
- Helper checks passed via python tools/tracker-debug/test_compatibility.py.
  pytest is absent in the selected interpreter; no dependency was needed.
- Read-only bounded review deleg_6713bdbd passed NNA/DCA NoteFade separation,
  authored-zero preservation, reset/trigger paths and clone/snapshot retention.
  Reviewer compared pinned CheckNNA branches and ran 5 passing focused tests.
  Direct pattern NOTE_FADE handling was outside acceptance and remains unverified.
  Full compatibility remains IN PROGRESS / NOT ACCEPTED; key-off semantics and DCT ownership/matching,
  XM multi-sample mapping, historical song control and rollback remain open.

## Initially-disabled IT envelope checkpoint

- Pinned OpenMPT f83cedb0cd5446e4dfaa83ac97e3087107e26767
  `ITTools.cpp:ITEnvelope::ConvertToMPT` retains nodes irrespective of ENV_ENABLED;
  `ModChannel.cpp:InstrumentControl` toggles channel volume/pan enable via S78/S7A.
  Nethercore discarded disabled source envelopes and additionally gated mixer
  evaluation by the immutable authored flag. Both failure points are repaired.
- Source parser preserves nonempty disabled volume/pan/pitch envelopes and flags;
  NCIT packing/conversion already preserves these. Volume/pan mixer evaluation now
  uses the live channel flag. Initial row setup still uses the authored flag.
  Pitch/filter S7B/S7C support remains an explicit outstanding obligation.
- Fail-before/pass-after: disabled envelope nodes were absent after parsing;
  authored-off pan envelopes with live channel enabled produced wrong stereo gain.
  Tests now cover source/NCIT retention and foreground/background pan modulation.
- Repeat: `python tools/tracker-debug/compatibility.py --envelope-enable-probe`.
  Two original generated IT sources pass fresh source→packer→cart→mixer and
  libopenmpt: centered→right→center for S7A/S79, full→half→full for S78/S77.
  Initial-off state, finite termination and restart parity pass. Temporary output
  is removed automatically; historical inputs unchanged.
- Runtime403 pass/1 ignored, IT62, converter47, CLI80, helper2 pass; rustfmt and
  diff checks pass. Existing EnvOffLength.it and EnvReset.it freshly captured;
  decoder equivalence holds within both players. Activity disagreements 0 and7,
  respectively, are diagnostics only; EnvReset remains unaccepted.
- Bounded independent review `deleg_c9eefbd2` confirmed source-node preservation,
  channel initialization and live enable flags against pinned reference; both
  focused regressions passed. Overall NOT ACCEPTED.

## IT default-panning checkpoint

- Source-backed repair: pinned OpenMPT f83cedb0cd5446e4dfaa83ac97e3087107e26767
  `Snd_fx.cpp:NoteChange/ApplyInstrumentPanning` restores defaults on note changes,
  including Gxx; sample pan overrides instrument pan and cancels surround. Applied
  after old-voice/NNA handling and before row effects. Instrument-only same-instrument
  cells retain pan. Missing defaults preserve pan/surround. Regression failed before
  at sample-mode first note (0.25 versus 1.0), passes after.
- Real packed probe exposed S91 preserving a hard pan. `ExtendedChannelEffect`
  centers on S91; SetSurround(true) now centers, false retains pan.
- Reference disagreement then independently exposed inverted instrument DfP bit7
  in BOTH parser and writer. Unlike sample DfP, instrument bit7 disables panning.
  Pinned `ITTools.cpp` confirms semantics and enabled malformed values >64 center.
  Raw-byte regression (all 256 parser values; writer None/left/center/right) failed
  before and passes after. Sample DfP encoding is unchanged.
- Repeat: `python tools/tracker-debug/compatibility.py --panning-defaults-probe`.
  Three original source fixtures exercise sample mode, instrument default, and
  competing sample/instrument defaults. Rebuilt importer/packer/runtime and fresh
  carts, checked surround → right → center → right(Gxx) → center in BOTH native
  mixer and libopenmpt; finite termination and exact restart-frame checks pass.
  Temporary fixtures/carts/audio/compiler output are automatically removed.
- Tests: runtime 403 pass/1 ignored; IT61, converter47, CLI80, helper2 pass.
  Helper tests used `uv run --with pytest python -m pytest -q
  tools/tracker-debug/test_compatibility.py` because system Python lacks pytest.
  `git diff --check` passes. No commits or source-asset compensation.
- Historical PanReset.it and SmpInsPanSurround.it were freshly repacked/rendered;
  decoded PCM equivalence holds within each player. Pan MAE respectively
  0.00012018993315174245 and 0.03606055488334585; activity disagreements 192 and 5
  remain diagnostic, NOT acceptance. Independent runtime/defaults review
  `deleg_e8b57951` found no concrete defect, including sample identity and NNA order;
  DfP boundary review `deleg_8ecfc6a2` matched the pinned source and passed the focused
  regression (1 passed). Both are bounded verdicts, not full tracker acceptance.
  Existing fresh G0..G9 and XM global-volume probes also pass after the writer change.
  Overall IN PROGRESS / NOT ACCEPTED.

## Coarse-panning and surround-override checkpoint

- Confirmed XM E8x and IT S8x incorrectly mapped 0..15 to inset 2..62 pan.
  Pinned OpenMPT f83cedb0cd5446e4dfaa83ac97e3087107e26767
  `Snd_fx.cpp:Panning(Pan4bit)` uses `(value*256+8)/15`; conversion now
  quantizes that to the existing 0..64 representation, reaching both hard edges.
  The shared explicit SetPanning handler now clears surround, matching IT override
  behavior. Existing X00/X80/XFF repair is preserved. Finer pan precision remains
  the existing representation limitation, not a bit-identical PCM claim.
- Both all-nibble converter tests failed before at 0 (2 instead of0), pass after.
  Surround-after-explicit-pan regression failed before, passes after. Corrected
  an older XM test that asserted the wrong inset endpoints. Converter47/runtime401
  and CLI80 pass (one runtime timing probe ignored); rustfmt/diff checks pass.
- Rebuilt source/cart/renderer path with two original fixtures: IT S91 then S80/S8F,
  XM E80/E8F. Row-interior L/R peaks were native IT(750,0)/(0,750), XM(2048,0)/
  (0,2048); libopenmpt IT(888,0)/(0,888), XM(918,0)/(0,918). Hard-edge output,
  surround cancellation, finite termination and restart assertions pass. Temporary
  source/cart/audio/compiler output removed; no historical assets edited.
- Existing `surround-pan.it` freshly repacked/captured, decoder equivalence true
  within both players; five activity disagreements, pan MAE0.49047474169036465,
  crossing MAE0.6598984771573604 remain unaccepted diagnostics.
- Independent bounded review `deleg_504a0189` found no issue: pinned Pan4bit,
  IT first-tick dispatch and enabled kPanOverride confirm behavior. Both converter
  tests and the runtime surround-cancellation test passed independently.
  Overall NOT ACCEPTED.

## XM simultaneous tone-portamento checkpoint

- Confirmed FT2 volume Fx plus main 3xx uses doubled volume-column speed and
  ignores the main parameter. Pinned OpenMPT
  f83cedb0cd5446e4dfaa83ac97e3087107e26767 `Snd_fx.cpp:GetVolCmdTonePorta`
  documents this with `TonePortamentoMemory.xm`. Conversion now applies this rule
  only to the two tone-portamento columns; main effects otherwise stay intact.
- Widened channel tone-portamento speed from u8 to u16, preserving doubled speeds
  up to 480; IT E/F/G byte memories remain bytes. The existing snapshot copies the
  widened state. Converter regression fails before and passes after across all
  16 volume digits and four main parameters; runtime regression checks speed480,
  per-tick period change, zero-parameter recall and snapshot restoration.
- Converter45/runtime400 pass (one timing probe ignored). Rebuilt CLI/runtime and
  existing renderer. Ten original fresh carts (digits0,1,7,8,15 with main00/FF)
  produced equal paired PCM within Nethercore and independently libopenmpt,
  verifying ignored main parameters, finite termination and restart. Temporary
  source/cart/audio/renderer removed, no historical module edited.
- Fresh `TonePortamentoMemory.xm` remains FAILED by the renderer clipping gate
  (peak1.3241502); activity disagreements5, pan MAE0.030385431970111166,
  crossing MAE0.5409836065573771. These are diagnostics, not full playback acceptance.
- Bounded review `deleg_f1c705b1` passed: pinned precedence, zero recall,
  maximum480 speed, snapshots and unchanged IT byte memories checked; both focused
  tests passed. CLI80, existing ten-pair IT fresh-portamento check and diff check
  also passed after widening.
- Uncached seeking still advances only row tick0 rather than replaying complete
  state; investigation reconfirmed this separate open repair obligation.
  Overall IN PROGRESS / NOT ACCEPTED.

## XM normalized global-volume runtime checkpoint

- Found a separate integration defect after the overflow-only repair: XM conversion
  emits a shared 0..128 `SetGlobalVolume`, but runtime divided XM commands by 64.
  Intermediate gain doubled. Changed the shared command handler to divide by 128
  for both formats; native-range slide handling and IT invalid-command behavior
  remain unchanged. The old runtime-only test asserted the wrong XM denominator;
  it is corrected and supplemented with a real XmModule-to-runtime test.
- `converted_xm_global_volume_preserves_authored_gain` failed before at G01
  (0.03125 instead of 0.015625) and passes after for all 256 source values.
  Runtime 399 passed, 1 ignored. The persistent `--xm-global-volume-probe` now
  checks seven fresh source/cart captures, including quarter and half gain, against
  within-player full-scale energy ratios in Nethercore and independently libopenmpt.
  Saturation equivalence, finite playback and restart checks still pass.
- Fresh `GlobalVolume.xm` remains FAILED with unchanged clipping/pan/activity
  diagnostics: this fixture does not establish acceptance of broader XM playback.
- Bounded review `deleg_6ee54423` completed with PASS: both producers emit the
  normalized command range; IT invalid values remain ignored; header and native
  slide ranges are separate and unchanged. Focused integrated test passed.
  Converter44, CLI80 and helper self-check also passed; diff check clean.
  Overall IN PROGRESS / NOT ACCEPTED.

## XM global-volume boundary checkpoint

- Confirmed XM Gxx conversion multiplied u8 before clamping, producing values
  above internal full volume and overflow for high parameters. Pinned OpenMPT
  f83cedb0cd5446e4dfaa83ac97e3087107e26767 `Snd_fx.cpp` CMD_GLOBALVOLUME clamps
  XM above 64 rather than ignoring it like IT. Conversion now uses `param.min(64)*2`.
- All-256-parameter regression failed before at 65 (130 rather than 128), passes
  after. Converter 44 passed; runtime 398 passed/1 ignored; CLI 80 passed.
- Repeat fresh source/cart/mixer check:
  `python tools/tracker-debug/compatibility.py --xm-global-volume-probe`.
  Original one-channel G40/G41/G7F/G80/GFF fixtures have identical PCM within
  Nethercore and independently within libopenmpt. Every cart completes finite
  playback and restart assertions; no cross-engine PCM identity claim. Generated
  sources, carts, renderer and audio are temporary, not modified historical assets.
  Existing helper self-check passes; rustfmt/diff checks pass.
- Fresh existing `GlobalVolume.xm` remains FAILED: mixer peak 1.585185 trips the
  existing clipping assertion; five activity disagreements, pan MAE
  0.7640860670621493, crossing MAE 5.090909090909091. The clamp repair does not
  claim to resolve this separate historical playback mismatch.
- Bounded independent review `deleg_ed651d7a` completed: clamp-before-scaling
  accepted and all-parameter focused test passed (1 passed, 43 filtered out).
  Parent's pinned source inspection and five fresh reference-paired cases provide
  the semantic check; this is not a wider playback verdict. Overall NOT ACCEPTED.

## IT volume-column portamento speed checkpoint

- Confirmed linear `(vol-193)*4` conversion was incorrect. Pinned OpenMPT
  f83cedb0cd5446e4dfaa83ac97e3087107e26767 `Tables.cpp:287` and
  `Snd_fx.cpp:4509` use `[0,1,4,8,16,32,64,96,128,255]` for volume G0-G9.
  The all-digit converter regression failed before (G1: 4 instead of 1), passes
  after. Conversion is the only production change in this slice.
- Repeat with `python tools/tracker-debug/compatibility.py --volume-porta-probe`.
  `nether-it/examples/volume_porta_probe.rs` generates original paired sources,
  including seeded G0 memory and repeated target notes. All ten pairs matched
  exactly within Nethercore and independently within libopenmpt (not across
  engines). CLI/runtime/renderer were rebuilt; all twenty fresh carts completed
  finite playback and restart assertions. Temporary sources/carts/audio removed.
- Verification: converter 43 passed; full runtime 398 passed, 1 ignored;
  CLI 80 passed; IT 60 passed;
  compatibility-helper pytest 2 passed via `uv run --with pytest python -m pytest`;
  rustfmt and `git diff --check` passed. Native Python had no pytest; uv recovered.
- `DoubleSlide.it` fresh capture still has 114 activity-disagreement windows,
  pan MAE 0.00447152141259186, crossing MAE 7.370932754880694. Decoder equivalence
  remains true in both players. Simultaneous-column memory/ordering is still an
  open repair obligation, not accepted by these paired-column checks.
- Independent bounded mapping review `deleg_b53b0176` completed with no defect:
  table/indexing, u16-to-u8 range, existing IT scaling and target setup checked;
  focused converter regression passed. This verdict does not accept dual-column
  precedence or full playback. `DoubleSlideCompatGxx.it` still reports 111 activity
  disagreements (pan MAE 0.003859682368657703; crossing MAE 3.5703395703395704),
  with decoder equivalence true. Overall NOT ACCEPTED.

## IT panning-envelope control checkpoint

- Confirmed S79/S7A were dropped by conversion. Added `SetPanningEnvelope(bool)`
  and the existing tick-zero dispatch now toggles the current channel's pan-envelope
  flag only. Pinned OpenMPT `ModChannel.cpp:243-244` at
  f83cedb0cd5446e4dfaa83ac97e3087107e26767 specifies this behavior.
- Raw S79/S7A conversion assertions failed before and pass after. Runtime regression
  checks held position while disabled, resume without reset, unaffected volume/NNA
  envelope clocks, and snapshot-restored audible versus silent advancement parity.
  Converter42/runtime398 tests pass (one timing probe ignored); rustfmt/diff pass.
- An original synthetic IT was written using ItWriter, freshly CLI-packed, and
  played through the existing shared renderer. Row interiors produced L/R peaks
  (0,750), (505,505), (0,750): right-only before, centered during S79, right-only
  after S7A. Finite termination and packed-restart assertions pass. Source SHA-256:
  `6c722e07db5f9844fcc5e0a1b5ce2646d0525ce8038d3160feda28610fbc8df6`.
  Temporary source/cart/audio/compiler output was removed by its temporary context.
- Existing `s77.it` also repacks/captures with per-player sample equivalence; source
  inspection confirms it exercises S77/S78, not the new pan commands, and its large
  activity/pan discrepancies remain unaccepted. Other missing S7 controls remain
  repair obligations. Bounded review `deleg_7a477ca9` source inspection agreed
  with the pinned reference, but its separate-target runtime rebuild stalled;
  reviewer ended interrupted after parent full-runtime verification succeeded.
  Completion event confirms no final review summary was produced.
  No completed independent S79 runtime verdict is claimed; no full acceptance.

## Long-held envelope boundary checkpoint

- Reproduced unchecked u16 envelope-position overflow in `advance_envelopes`.
  All four position increments now saturate rather than panicking in debug or
  wrapping to the start of the envelope in release. Existing sustain/loop handling
  is unchanged; this does not certify its broader format-specific behavior.
- `long_held_envelopes_saturate_instead_of_wrapping` fails before on overflow and
  passes after, checking the final representable tick and subsequent ticks for all
  four envelope positions. Runtime 397 passed, 1 timing probe ignored; rustfmt and
  diff checks pass. Fresh packed `PanSustainRelease.xm` capture/restart checks pass
  with unchanged diagnostics (three activity disagreements still unaccepted).
- This additional boundary fix is not covered by a completed independent verdict;
  the earlier clock review had already stopped accepting steering at this checkpoint.

## Envelope clock boundary checkpoint

- Confirmed envelope/fadeout advancement was inside `process_tick`, which is not
  called at row boundaries. Speed-one songs never advanced envelopes; other speeds
  missed one advancement per row. Pinned OpenMPT `Sndmix.cpp`
  `IncrementEnvelopePositions` calls (2227/2457) run on the tick path, including
  XM/IT first ticks; the explicit first-tick exclusion there is for MED only.
- Extracted the existing envelope/fadeout block into `advance_envelopes`, reused
  by `process_tick` and the row-ending clock branch. No extra volume/pitch/tempo
  slide is run at row end, and no envelope loop/sustain rule was changed here.
- Fail-before/pass-after regression checks all four envelope clocks and fadeout
  at every completed tick, for speeds 1/2/6 in XM/IT, foreground/NNA voices,
  and exact audible-versus-silent channel-state parity. Runtime suite 396 passed,
  one timing probe ignored; the expanded format regression also passes.
- Fresh packed `PanSustainRelease.xm` and `SampleSustainAfterPortaInstrMode.it`
  complete capture and restart checks. XM pan MAE changes from 0.419 to 0.384;
  remaining activity/pitch/pan differences are not accepted. This is not a full
  envelope-lifecycle verdict. Review `deleg_9d5836ac` completed with no bounded
  clock/caller defect found: both envelope-clock and no-extra-slide regressions
  passed; foreground/NNA coverage and pinned OpenMPT tick behavior were confirmed.
  This verdict does not cover the later saturation change or broader envelope
  lifecycle semantics. Rustfmt/diff checks pass.

## Panning-envelope checkpoint

- Confirmed two representation/mixing defects: XM pan envelopes were copied as
  unsigned 0..64 despite the unified signed -32..32 contract; the mixer replaced
  base pan with envelope output, including treating neutral IT zero as hard left.
- XM conversion now centers only the pan envelope; volume values retain 0..64.
  Mixer uses `pan + envelope / 32 * (1 - abs(pan))`, clamped to [-1,1]. This is
  the normalized equivalent of pinned OpenMPT `Sndmix.cpp` lines 1135-1160,
  `ProcessPanningEnvelope` (commit f83cedb0cd5446e4dfaa83ac97e3087107e26767).
  IT parser/converter already retain signed pan points and need no conversion.
- Two fail-before/pass-after regressions cover XM endpoints/neutral and unchanged
  volume data, plus neutral/partial/extreme modulation for foreground and NNA voices.
  Mixer comparisons use identical sample state and explicit expected base pan,
  avoiding assumptions about interpolation amplitude. Base channel pan is unchanged.
- Affected suites: tracker 42 passed; runtime 395 passed, 1 ignored timing probe;
  rustfmt/diff checks pass. Bounded independent review `deleg_aba96513` completed:
  no defect reported in signed conversion or the mixer formula against pinned OpenMPT;
  both focused regressions passed. It also confirmed unchanged signed IT values,
  volume-envelope range, and the output-only placement of pan work after the silent
  guard. This closes the bounded formula review, not full envelope lifecycle acceptance.
- Freshly rebuilt/packed `PanSustainRelease.xm` completes capture and the packed
  restart assertion. Three activity disagreements and pan MAE 0.419 remain;
  complete envelope lifecycle is NOT accepted. The IT instrument-sustain fixture
  also captures with per-player sample equivalence unchanged. An initial combined
  invocation incorrectly requested IT-only sample-equivalence for the XM; the XM
  was rerun without that flag and exited zero. No gain/fixture workaround applied.

## XM extracted-ID handoff checkpoint

- Confirmed XM extraction sanitized names/deduplicated PCM but `load_tracker` still
  used raw instrument labels. Duplicate labels also failed as apparent explicit-sound
  collisions. This caused missing-sample errors and blank-name silent handles.
- Reused the existing extracted-ID map for XM instrument slots as well as IT sample
  slots. XM now retains IDs through PCM aliasing and collision renaming; packed lookup
  prioritizes those IDs, then explicit raw labels, then silent/sample-name fallback.
  The existing one-slot-per-XM-instrument representation still lacks full keymaps;
  preserving the first extracted sample is not a multi-sample compatibility repair.
- `xm_extracted_ids_preserve_blank_names_collisions_and_pcm_aliases` fails before on
  duplicate `Perc !` PCM and passes after, checking blank names, distinct PCM with
  equal labels, and equal-PCM aliases by resolved sound data. CLI suite: 79 passed;
  rustfmt and `git diff --check` pass. Review `deleg_b41c5ee5` validated this
  handoff but found a zero-length-payload edge case; owner reproduced and repaired
  it below.
- Rebuilt/repacked `PatternDelays.xm`, `PatternDelaysRetrig.xm`, `GlobalVolume.xm`,
  and CC0 `napping on a cloud.xm`. All now reach audible mixer capture rather than
  name-resolution/collision failure or the prior blank-handle silence. None passes
  capture acceptance: measured peaks 1.3241502, 1.3241502, 1.585185 and 1.9446778
  respectively exceed the unchanged loudness gate. Musical/pan/pitch differences
  also remain. Batch is `target/tracker-compatibility/xm-identity-results.json`;
  no gain workaround, fixture edits, or SpecCade changes were applied.

## Empty XM payload review repair

- Review `deleg_b41c5ee5` correctly identified that `num_samples > 0` can describe
  zero bytes of PCM. The new regression reproduced the missing `music_inst0` error.
- Full XM parsing now retains its already-validated aggregate payload-byte count
  as `source_sample_bytes`; minimal ROM metadata leaves it unknown (`None`). No
  packed format change or second PCM decode is needed. Loader maps known-empty
  source slots to silence after checking extracted IDs and explicit named sounds.
- Regression covers blank/named instruments with zero, one and two empty sample
  headers, explicit sound overrides, and truncated-header rejection. CLI 80,
  XM 50 and converter 42 tests pass; rustfmt/diff checks pass.
- A disposable original two-row XM was freshly packed through the CLI and the
  existing shared renderer: empty first slot produced exactly zero PCM for its
  entire row, populated second slot produced audio, playback terminated, and the
  packed restart assertion passed. Source SHA-256
  `bb513459fd1673361707cd61f16db9bcd3c14d5f0ec46afc5518aee39c78ffa7`.
  Temporary cart/audio/source were removed by the temporary-directory context;
  the permanent regression remains in CLI asset tests. Full XM keymaps and wider
  playback acceptance remain open; no reference fixture was modified.

## Independent silent-output/cache review received

- `deleg_122dc3ce` found no concrete defect in the shared silent-output state path
  or handle-scoped row-cache lookup. Its four focused checks passed: cache isolation,
  the explicitly enabled silent-mixer parity/cost probe, sustain/release advancement
  parity, and nearest-cache lookup. Owner rechecked the current output-only guard
  and handle-bounded range after receipt. This verdict does not cover subsequent
  reset/delay repairs or establish full tracker compatibility.
- Background capture `proc_f64a5ae541a9` exited zero: freshly packed
  `PatternDelays.it` and `SampleSustainAfterPortaInstrMode.it` completed with
  original/decoded sample equivalence true within both players. Reported activity
  disagreements were respectively zero and five; the sustain pitch/pan differences
  remain unaccepted. These are run-specific results; shared `results.json` can be
  replaced by subsequent diagnostic runs.

## Reset and pattern-delay checkpoint

- `reset()` now clears cached rollback states, sync identity, row/fine delays,
  global-slide memory, tempo slide and format flags, while retaining loaded modules
  and handle allocation. `music_play` and uncached seek both call this shared path.
  `restart_clears_delay_and_cached_playback_state` failed before (cache survived)
  and passes after, including audible first-note restart.
- The compatibility runner adapts the existing shared renderer in a temporary
  directory, without editing SpecCade. Intentional empty sample IDs resolve to handle
  zero like production. Every capture asserts exact first-frame PCM after 18 frames
  of playback and reset. Adapted/shared helper hashes are recorded separately.
- Instrument-mode `SampleSustainAfterPortaInstrMode.it` now completes a fresh
  packed render; the prior capture-helper blocker is removed. Restart assertions
  and original/decoded sample equivalence pass within both players. Its five
  activity disagreements and pitch/pan discrepancies remain unaccepted.
- Pinned suite IT/XM README `PatternDelays` entries specify first SEx (including
  SE0) for IT and last EEx (including EE0) for XM. S6x sums across channels.
  Row processing now resolves those rules and retains fine delays across repeats;
  the accumulator/snapshot is u16 so 64 S6F channels do not truncate at 255.
  `pattern_delay_precedence_and_tick_sum_follow_format` failed before and passes
  after, covering zero precedence, both formats, 64-channel summation and repeats.
- Fresh packed `PatternDelays.it` activity disagreements fell from 150 to 0;
  crossing MAE is still 0.380 and no musical acceptance is inferred. IT retrigger
  delay fixture still differs (5 activity windows, crossing MAE 1.652).
- Fresh XM delay probes expose existing unrelated failures: `PatternDelays.xm`
  cannot resolve sanitized instrument labels, and `PatternDelaysRetrig.xm` renders
  silently. Both are explicitly failed, not excluded. Batch: target/tracker-compatibility/delay-results.json.
- IT main-column E/F fine and extra-fine slides now repeat at delayed-row first
  ticks via the existing effect handler, without replaying notes or volume-column
  fine slides. A fail-before/pass-after regression covers both directions/modes.
  Fresh `PatternDelaysRetrig.it` crossing MAE improved from 1.652 to 0.890; five
  activity disagreements remain. Both delay IT fixtures pass per-player sample
  equivalence and packed-restart assertions. Other repeated-row effect behavior,
  notably delayed-note retrigger and fine volume/global/channel slides, remains open.
- Full command `cargo test -p nethercore-zx -p nether-tracker -p nether-it -p nether-xm
  -p nether-cli --no-fail-fast` passes: runtime library 393 (1 ignored timing probe),
  CLI 78, IT 60, tracker 41, XM 50; binary/integration and enabled doc tests also pass.
  Python quiet-fixture/adapter checks pass (compiler failures propagate, original
  helper path is restored, source drift fails closed); rustfmt passes.
- `deleg_a5292358` returned a truncated review: no reset/adapter regression found,
  but its adjacent recalled-down-slide claim required owner verification. Inspection
  of every production E/F memory assignment shows both fields updated together in
  IT; Compatible Gxx separates G memory, not E from F. Therefore choosing
  `last_porta_up` in the IT-only repeat path does not establish a defect.
  New `it_delayed_opposite_pitch_recall_keeps_shared_ef_memory` passes on unchanged
  runtime code, exercising both recall directions, fine/extra-fine modes, both Gxx
  flag settings and three delayed-row first ticks, with both memory fields checked.
  Current control tests: 32 passed, 1 ignored; rustfmt/diff checks pass. The review's
  linker failure did not recur in this owner run. No speculative runtime fix applied.
  Full compatibility remains NOT ACCEPTED; uncached reconstruction,
  repeated-tick effect semantics and XM mappings remain open.

## Latest runtime and rate checkpoint

- IT E/F conversion now preserves the complete raw parameter byte, including
  E/F fine-mode nibbles. E00/F00 replay fine/extra-fine mode only on tick zero.
  Boundary regressions cover E0/E1/EF/F0/F1/FF and opposite-direction recall.
- Two regular pitch slides in the volume/main columns now both run instead of
  collapsing into one boolean activation. Up/down counts reset each row;
  regressions cover both directions and a following blank row.
- IT channel header panning, surround/mute and channel volumes now survive the
  unified conversion and full playback reset. Header global volume is applied.
  Packed NCIT -> converter -> playback/reset/snapshot regression passes.
- `deleg_e0474857` completed; its logged review confirmed the three earlier
  instrument-only findings fixed and found no concrete envelope-metadata defect.
  Full final text delivery is pending; do not treat that as broad acceptance.
- `deleg_a28b44e5` completed: identity/collision/dedup/order/callers had no functional
  finding. It identified C5Speed=0 reaching division by zero in the resampler.
  Parent confirmed source flow without attempting an unbounded allocation.
- Pinned OpenMPT ITTools.cpp applies C5=0 -> 8363 and clamps nonzero rates below
  256 to 256. `ItSample::playback_c5_speed()` now supplies that same effective rate
  to extraction and loop-coordinate conversion while preserving the raw header.
  Zero/1/255/256/8363 regression failed before, passes after. IT tests: 60 passed.
- Full affected-suite rerun: runtime 385, IT 60, converter 41, XM 48, CLI 77
  passed. All six named fixtures were freshly rebuilt, repacked and rendered;
  original/decoded PCM equivalence remains true within both players for each.
  Activity disagreements: DoubleSlide 113; DoubleSlideCompatGxx 112;
  PortaReset 1; PortaResetAfterRetrigger 12; PortaSample and Compat 5 each.
  These remain diagnostic and unaccepted. Review `deleg_c4ca3d11` is pending.
- Double-column memory initialization ordering, fine-as-regular volume-column
  interactions, non-linear period semantics and remaining inventory are still
  unresolved. These changes do NOT establish full compatibility.

## Latest packer checkpoint

- IT extraction now retains sample-index-to-sound-ID mappings. Duplicate display
  names get collision-free IDs without replacing explicit sounds; identical PCM
  reuses storage while every original sample index retains its mapping.
- Sound-map ordering is deterministic. Regression covers conflicting explicit
  names, generated-name collisions, identical-PCM aliases, distinct sample PCM,
  and sorted output IDs. The prior implementation failed this regression.
- A truncated embedded PCM payload previously packed successfully by falling back
  to a matching explicit sound with different audio. IT extraction errors now
  propagate. Sample-less modules still use the successful empty extraction path.
- `cargo test -p nether-cli`: 77 passed. Freshly rebuilt/repacked DoubleSlide.it
  and DoubleSlideCompatGxx.it both finish in the native mixer; original/decoded
  sample equivalence passes within each player. They are NOT playback-accepted:
  activity disagreements remain 113/112 windows and crossing MAE 7.417/3.518.
- Independent packer review dispatched as `deleg_a28b44e5`, pending. Runtime
  follow-up review `deleg_e0474857` is also separate from full acceptance.
- Known unresolved: double-column effect ordering, fine-slide memory, broad
  envelopes/voices/filters, XM mapping, initial panning, corpus/rollback gates.

## Silent-advance optimization checkpoint

- `process_channels` now has a compile-time output flag. Both modes retain the
  same sample interpolation, pitch/filter envelope state, filter history and
  voice lifecycle. Silent advancement skips only output gain, volume/pan envelope
  evaluation, panning and stereo accumulation; it does not duplicate loop logic.
- Runnable check: `cargo test -p nethercore-zx --release --lib
  silent_mixer_state_parity_and_cost -- --ignored --nocapture`.
  Sixteen active voices, half filtered, five repetitions of 44100 samples.
  Full channel-state debug values and 128 subsequent stereo samples match exactly.
- Within-run median full-mix/silent cost ratio was 1.350 after optimization,
  versus 1.038 when both modes performed full mixing. This is a bounded microbenchmark,
  not an end-to-end main-thread speedup or the former 10–20x claim. Absolute times
  varied between builds, so cross-build absolute speed ratios are not asserted.
- Runtime: 390 passed, one timing probe ignored normally and explicitly passed in
  release. Formatting and diff checks passed. The same three restart/sustain
  fixtures were freshly rebuilt, packed and rendered; diagnostic values and
  per-player sample equivalence are unchanged. These remain unaccepted musically.
- Review `deleg_122dc3ce` covers this optimization and the previously unreviewed
  row-cache handle isolation. Pending verdict; full compatibility remains open.

## Latest review-repair checkpoint

- Invalid instrument-only cells now ignore the instrument field but execute both
  effect columns. A regression covers S77/S78, volume, global volume, preserved
  identity and sample position; it failed before and passed after the repair.
- Sustain-loop sample advancement now uses the same selected loop bounds as
  interpolation. Held/released regression covers a sustain loop both with and
  without a regular loop.
- Silent advancement now reuses mixer transitions, deleting its duplicate loop
  advancement implementation. A rendered-versus-silent held/released state check
  failed before and passes after. Cost: silent advancement performs a mix and
  discards output; no performance acceptance is inferred from unit timing.
- Row caches are now keyed by tracker handle as well as order/row. Alternating
  between modules with different volume/pan defaults failed before and passes after.
- Runtime suite: 390 passed. CLI suite after named-empty-slot repair: 78 passed.
- Freshly rebuilt/repacked PortaResetAfterRetrigger and the two sample-mode
  SampleSustainAfterPorta variants render finitely; per-player original/decoded PCM
  equivalence remains true. Disagreement windows are respectively 12, 7 and 5;
  these are not musical acceptance results.
- Instrument-mode sustain fixture exposed a named empty slot treated as a missing
  sample. The packer now preserves its silent slot (or an explicitly supplied sound),
  with a fail-before/pass-after regression. Fresh packing succeeds, but the shared
  SpecCade diagnostic renderer rejects the empty ID as `tracker sample absent`.
  That helper limitation is NOT a native mixer failure or a passed capture. No
  SpecCade files were changed. This fixture's end-to-end capture remains blocked.
- Independent review `deleg_055128ea` found no correctness mismatch in the
  invalid-cell and sustain/advance changes. Focused control/engine/audio suites
  passed (28/8/12). Row-cache repair was outside that review brief.
- The review confirmed a main-thread performance regression in execution scope:
  active trackers now perform full mixing while discarding PCM every frame. The
  former 10–20x lightweight-advance documentation was removed. No before/after
  runtime benchmark was supplied, so the slowdown factor is unknown. Performance
  remains unresolved; retaining state parity is mandatory for any optimization.
- Uncached mid-row reconstruction and complete song/loop/rollback acceptance remain
  unresolved, along with the prior XM mapping and broader format inventory.

## Integrated asynchronous reviews

- `deleg_e0474857` confirmed the three instrument-only restart repairs and bounded
  terminal-envelope/S77/S78 behavior. Its remaining finding: an invalid instrument-only
  cell returns before processing either effect column. A co-located effect regression
  and repair remain required.
- `deleg_a28b44e5` found no identity-map/deduplication/collision-ordering defect. Its
  zero-C5 allocation finding was subsequently addressed by the source-backed playback
  rate normalization; `deleg_c4ca3d11` independently confirmed raw-rate preservation
  and effective-rate handling. This closes that specific finding, not allocation safety
  for every possible malformed input.
- `deleg_c4ca3d11` confirmed the bounded raw slide/count/header/rate repairs, but found
  sustain-loop bounds inconsistent between selection and advancement, missing tick
  replay during uncached mid-row synchronization, and row-cache state not keyed by
  module handle. These remain concrete review findings requiring focused reproduction
  and repair. The complete summary was read, including the truncated middle.
- Review runtime execution reported 384 passing tests and one replay-initialization
  failure. The later owner run passed all 386 runtime tests; this discrepancy is not
  silently treated as an independent clean-suite verdict.
- Reviews are integrated as bounded evidence. Overall status remains NOT ACCEPTED.

## Historical-module checkpoint

- XM Gxx global volume was incorrectly divided by 128 instead of 64; IT invalid
  V81..VFF was incorrectly clamped rather than ignored. Pinned OpenMPT
  `Snd_fx.cpp` CMD_GLOBALVOLUME is the reference. Boundary regression fails before
  and passes after the format-specific fix; global-slide tests still pass.
- Fresh complete captures of both CC0 modules exposed independent failures:
  `Brute VII 2.it` is still playing after the reference duration (order 1, row 2,
  wraps 1). Loop intent versus finite termination still needs source-level resolution.
- `napping on a cloud.xm` initially failed at instrument 3 because the XM parser
  skipped PCM between sample headers. XM stores ALL sample headers before PCM.
  A two-sample/next-instrument regression fails before and passes after correction.
  Header underflow/size, sample count/header size, and PCM extent are now checked.
  XM suite: 50 passed. Its fresh pack now reaches a duplicate `perc` sample collision.
- XM multi-sample note maps, distinct sample metadata, flattening/handle mappings,
  and minimal serialization remain a concrete repair obligation: the current
  per-instrument metadata representation still collapses multiple samples.
- Neither CC0 module is accepted. No fixture changes or asset compensation applied.

## Resume / current findings

- Starting checkout: main at `7dac9fb`; one worktree. Existing unrelated core,
  audio, rendering and tracker runtime changes, plus the previous panning fix.
  Inspect live status before edits; never restore whole files from HEAD.
- Audits returned; worker summaries are leads, not acceptance. Parser/writer
  changes and clock controls compile and pass local tests; still need full
  source-bound review and feature-specific packed playback.
- Both columns now survive conversion and execute at runtime. Explicit volume
  zero is carried as `SetVolume(0)`. IT S77/S78 now survive conversion;
  other S7x controls remain incomplete.
- Integration caught a regression: removing the effect handler's raw-note
  target assignment left tone portamento with no target at all. Row processing
  now assigns the continuing note's target, retaining sample position and
  tick-zero pitch. The previously failing runtime test now passes for XM/IT,
  main/volume columns, and combined tone-portamento + volume slide.
  This does NOT establish IT note-map/transposition or highest-note fidelity.
- Global-slide row activation and parameter memory are now separate per-channel
  fields, included by existing channel snapshots. Normal/fine slide scale,
  fine-command recall and IT invalid mixed nibbles follow OpenMPT
  `f83cedb0cd5446e4dfaa83ac97e3087107e26767`, `soundlib/Snd_fx.cpp`,
  `GlobalVolSlide`. Unit coverage includes blank-row hold, H00/W00 recall,
  snapshot restore and fine-slide recall. Cross-channel interactions still need
  explicit regression coverage.
- IT packer handle list was independently confirmed to use instrument names.
  Sample-only IT therefore packed an empty list even with embedded PCM. The IT
  branch now enumerates samples, using extraction's sanitized IDs unless an
  exact explicit sound exists. Regression failed with `[]` vs `["saw"]` before
  the fix; `cargo test -p nether-cli` now passes all 76 tests. Coverage includes
  sample mode and an instrument name differing from its sample name.
  Remaining: name collisions/content-dedup aliases, unnamed/empty references,
  and correct runtime instrument keymap selection are not accepted yet.
- IT sample-map/default metadata and NNA/loop paths remain incomplete.
  A narrowed worker owns sample-only trigger repair after a real silent render.
- Prior packed panning evidence: `target/it-panning-check/{before,after}.json`.
  This is only panning evidence; no general compatibility claim.

## Inventory (all entries require real-path evidence before PASS)

| Surface | State | Evidence / next check |
|---|---|---|
| Parsing and pattern encoding | AUDITING | full/minimal round trips and external fixtures |
| Packing and sample extraction | AUDITING | metadata/data correspondence |
| Instrument/sample mapping | AUDITING | multisample instruments and IT sample mode |
| Simultaneous volume/main effects | UNIT VERIFIED | both columns preserved; packed interactions pending |
| Explicit zero volume | UNIT VERIFIED | `SetVolume(0)` reaches runtime; packed verification pending |
| Effect memory and ordering | AUDITING | tick traces / interaction fixtures |
| Speed, tempo and tuning | AUDITING | reference event timing and pitch |
| Envelopes and key-off/fade/cut | AUDITING | sustain/loop/release fixtures |
| Panning and surround | PARTIAL | X00/X80/XFF repaired; other interactions unverified |
| Sample and sustain loops | AUDITING | forward/ping-pong/release boundaries |
| NNA/DCT/DCA | AUDITING | S7x conversion drops command; voice lifecycle |
| Filters | AUDITING | IT pitch/filter envelope distinction |
| Song control | AUDITING | order markers, loops, delay, break/jump |
| Format compatibility flags | AUDITING | full/minimal preservation and runtime use |
| Seeking/rollback/audio thread | UNVERIFIED | equivalence to uninterrupted playback |
| Historical-module corpus | PENDING RIGHTS | no implicit relicensing of repository assets |

## Verification checkpoint (2026-09-06)

- `cargo test -p nether-tracker -p nether-it -p nether-xm`: IT 47,
  tracker 38, XM 48 unit tests pass; one runnable doctest passes.
- `cargo test -p nethercore-zx --lib`: 368 pass, including the expanded
  tone-portamento regression and seven clock-control tests. These are not
  feature-complete compatibility tests.
- Real source -> fresh pack -> mixer attempted for `PortaReset.it` and
  `TonePortamentoMemory.xm` using `tools/tracker-debug/compatibility.py`.
  IT produced silence while the libopenmpt reference is audible (1.62 seconds).
  XM capture stopped at `tracker sample absent`; helper/sample-reference
  semantics need investigation, not a premature verdict on XM playback.
- Rebuilt `target/debug/nether.exe`, the debug `nethercore_zx` library, and
  `target/tracker-compatibility/render_tracker.exe`. The diagnostic stores exact
  runtime/helper hashes and case outcomes in `target/tracker-compatibility/results.json`.
  Later runs replace this report, so do not infer historical results from it.
- The diagnostic now explicitly labels capture errors FAILED and exits nonzero;
  aggregate metrics and a zero process exit are never compatibility approval.
- Windows linker recovery: a stale MSYS `TMP` value caused LNK1104. Shell exports
  alone did not fix the inherited native environment. Run Cargo as a Python
  subprocess with `env` explicitly setting `TEMP`, `TMP`, `TMPDIR` to
  `C:/Users/rdave/AppData/Local/Temp`. No global environment change is needed.

## Latest integration checkpoint (2026-09-06)

- Recovered from the repetitive-command guard by separately checking compilation,
  then running the corrected probe through the Python executor; no user action
  or additional authorization was required.
- IT sample-mode trigger/retention regression independently passes. Real packing
  exposed two additional losses: NCIT's unknown sample length clamped loops to
  zero, and absent IT volume fields became `SetVolume(0)` on blank rows. The
  latter also confused absent notes with genuine C-0. Corrected unknown loop
  bounds and explicit internal absence sentinels in `ItNote`, full/NCIT writers,
  and conversion. Full -> NCIT presence regression covers blank, C-0 + volume
  zero, and effect-only rows. Old cached files cannot regain values previously
  discarded by their writer: source modules must be repacked.
- `target/global-slide-check/check.py` reuses the existing SpecCade renderer
  unchanged and freshly packs two diagnostic modules with identical authored
  uncompressed mono PCM. `W01 / blank / W00` per-tick gain ratios PASS against
  libopenmpt: native approximately 1.00000 -> 0.99241 (holds through blank)
  -> 0.98448; reference 1.00000 -> 0.99202 -> 0.98444. Tolerance 0.002 absolute.
  Both complete finite playback; report: `target/global-slide-check/results.json`.
  Rebuilt `target/debug/nether.exe` and runtime; capture executable is
  `target/global-slide-check/render_tracker.exe` (exact hashes in report).
- The compressed-sample discrepancy was subsequently source-confirmed and
  repaired (checkpoint below). The authored PCM global-slide diagnostic still
  isolates global-slide behavior only; it does not certify historical playback.
- Latest tests: `nether-it` 47, `nether-tracker` 40, `nether-xm` 48,
  `nether-cli` 76, `nethercore-zx --lib` 370 pass; one runnable doctest passes.
  General compatibility, historical corpus, pitch/voice/seek behavior and
  independent source-bound review remain NOT ACCEPTED.

## Compressed-sample checkpoint (2026-09-06)

- Source-confirmed defects in `nether-it/src/compression.rs`: ignored packed
  block sizes, width-change commands consumed output sample slots, wrong
  mode-C width decoding, missing sign extension/delta accumulation, and silent
  zero-padding on truncated input. Sample Cvt flags were discarded, so IT214
  and IT215 could not be distinguished. `extract_samples` also swallowed errors.
- Repaired the shared 8/16-bit block decoder with bounded reads, actual emitted
  sample counts, byte-aligned declared block extents, correct width changes,
  and independent integrators reset per block. Cvt bit 2 selects IT215 double
  delta; IT214 uses single delta. Stereo channels use the consumed block extent.
  Extraction now propagates corrupt-header/data errors rather than silently
  dropping samples. The PCM writer clears compression and writes Cvt=1 to match
  its actual payload. NCIT needs no storage change: it references decoded PCM.
- Reference: OpenMPT commit `f83cedb0cd5446e4dfaa83ac97e3087107e26767`,
  `soundlib/ITCompression.cpp` (`Uncompress`, `ChangeWidth`, `Write`) and
  `soundlib/ITTools.cpp` (`ITSample::GetSampleFormat`).
- Fail-before evidence: two independent hand-packed regressions failed with
  decoded `[2,-29]` instead of `[1,1]`, and consumed bytes 4 instead of 5.
  Both pass after repair. Further checks cover every width at 8/16 bit,
  IT214/215 integration, mode-A/B/C transitions, invalid widths, short blocks,
  block-integrator resets and stereo channel placement. Compressor roundtrips
  now assert sample values rather than only lengths.
- Fresh `PortaReset.it` extraction converted to PCM produces EXACTLY the same
  reference-player render as the untouched compressed source. Freshly packed
  native compressed and decoded variants also match each other and terminate.
  A derived IT215 version of the same PCM additionally matches IT214 in both
  players. This establishes sample-decoding equivalence, NOT native-vs-reference
  tracker playback fidelity: PortaReset retains 19 activity-window disagreements.
- Repeatable retained integration command:
  `python tools/tracker-debug/compatibility.py --case PortaReset.it --sample-equivalence`
  rebuilds the CLI/runtime, packs both variants, compares PCM within each player,
  and removes the temporary transformed source. `nether-it/examples/decode_to_pcm.rs`
  is the minimal sample-boundary diagnostic used by this existing capture route.
  Report: `target/tracker-compatibility/results.json`. Extra IT215 experiment:
  `target/global-slide-check/compression-results.json`.
- Current regression checkpoint: IT 53, tracker 40, XM 48, CLI 76, runtime 370,
  and one runnable doctest pass. Independent source review is in flight.
- Remaining sample obligations: unsigned/endian/delta uncompressed Cvt handling,
  stereo preservation through the packer/mixer (currently mono), historical
  format quirks, instrument-mode sample mapping/defaults and complete musical
  expectations. None are waived by the compressed-sample checks.

## Follow-up parser and runtime checkpoint

- Independently reproduced all three NCIT panic paths before repair: channel
  count 65+ slicing fixed arrays, legacy channel marker 127 indexing mask memory,
  and sparse note-map signed addition overflowing before valid exceptions apply.
  Channel bounds now fail with errors; sparse offset addition wraps identically
  in debug/release before exception overrides. Three focused regressions pass.
- Empty writer payloads no longer advertise HAS_DATA (tested for initially set
  and initially clear flags); one zero-valued PCM sample remains present.
- IT cut notes and non-portamento target reset have failing-before/passing-after
  runtime regressions. NNA cut additionally erased the already selected incoming
  sample volume, silencing repeated instrument/sample numbers. The sample-mode
  regression now includes such a repeat and reproduced the volume-zero failure.
  NNA cut/fallback kills the outgoing voice without zeroing incoming volume.
- Fresh source -> pack -> native PortaReset.it after that volume repair:
  activity disagreements fell from 19 to 1 twenty-millisecond windows, and
  crossing-count mean error fell from 2.0411 to 0.2466. These are diagnostics,
  not full acceptance. Compressed/decoded-PCM equivalence still passes separately
  within libopenmpt and within Nethercore. Remaining boundary window is under
  investigation; reference song length is 1.62 seconds.
- Current bounded independent review covers NCIT bounds, writer presence and NNA
  cut volume changes. Full NNA/DCT/DCA (especially old/new instrument ordering),
  fades, seek reconstruction, historical corpus and overall compatibility are
  still NOT ACCEPTED.

## Instrument-mode follow-up (incomplete)

- Added fail-before/pass-after coverage for mapped IT sample handles, metadata,
  transposed pitch, and instrument-only restart after a voice stops.
- Mapped sample default panning is recalled on instrument-bearing triggers and
  stopped-note restarts. The channel representation is -1..1 (effect input is
  0..64); a first incorrect scale was caught by real capture and repaired.
- Full runtime library: 374 passing tests. Fresh `PortaResetAfterRetrigger.it`
  pack/render succeeds and original/decoded PCM equivalence passes in both
  players. Latest diagnostic: 56 activity disagreements of 105 audible windows,
  pan MAE 0.0688768931, zero-crossing MAE 17.60952381. NOT ACCEPTED.
- Remaining known gaps include compatible-G sample selection, instrument-only
  recall while a voice remains active, old/new NNA state ordering, sample global
  volume, and full envelope/filter lifecycle. Reverse lookup of a sample by sound
  handle on restart can be ambiguous when handles are shared; retain exact sample
  identity rather than treating that lookup as fully correct.
- Previous independent review failed with HTTP 429; no passing verdict exists.

## Exact sample identity and volume-column pitch checkpoint

- IT restart now retains the sample-table index instead of reverse-looking up a
  shared PCM handle. Regression failed with the wrong default volume before the
  repair; snapshot restoration and NNA copy retain the exact index.
- Pinned OpenMPT `Load_it.cpp` maps volume bytes 105..124 to normal pitch slides;
  `Snd_fx.cpp` passes the digit shifted by two to PortamentoUp/Down. Removed the
  incorrect fine-slide conversion, with all ten digits in both directions tested.
- Tracker suite: 41 pass; runtime suite: 375 pass. Fresh packed
  `PortaResetAfterRetrigger.it` remains unaccepted: 56 activity disagreements/105
  audible windows; original/decoded equivalence passes in both players.
- The earlier reverse-lookup limitation above is resolved. Other listed gaps
  remain open; these narrow repairs do not constitute corpus acceptance.

## Restart pitch and terminal envelope checkpoint

- Independently reproduced instrument-only restart retaining a previously slid
  period. The row path now remembers the original note and reconstructs its mapped
  pitch. Existing regression fails before and passes after the change.
- Source: pinned OpenMPT `Snd_fx.cpp`, `ProcessEffects` next to the
  `PortaResetAfterRetrigger.it` comment (`note = chn.nNote`), and `NoteChange`
  `kITRealNoteMapping` original-note memory.
- The same fixture has a volume envelope `(0,64) -> (7,0)`. Native playback left
  that silent voice active indefinitely, preventing later instrument-only restarts.
  Terminal-zero metadata is now retained in channel snapshots/NNA copies, and IT
  tick processing retires it past the final node unless an envelope loop is active.
  Regression failed before and passes after; a looping envelope stays alive.
  Reference: pinned OpenMPT `Sndmix.cpp`, `IncrementEnvelopePosition` endReached
  silent-final-node handling. Full envelope timing/sustain/carry remains unaccepted.
- Further regressions reproduced a tone-portamento target ignoring the IT note
  map (period 3840 instead of 3072), and S77 being discarded by conversion.
  Targets now use the mapped pitch without resetting playback position; S77/S78
  pause/resume volume envelopes without resetting position. Source: pinned
  OpenMPT `ModChannel.cpp::InstrumentControl`. Other S7x commands remain owed.
  Sustain hold/release, snapshot restoration and NNA metadata copying are covered
  by the terminal-envelope regression.
- Sample-mode portamento now switches samples without restarting when Compatible
  Gxx is disabled, and retains the old sample when enabled. Regression failed
  with handle 7 instead of 8 before repair. The legacy `LINK_G_MEMORY` name is
  misleading: source IT header bit 0x20 means Compatible Gxx; broader effect-memory
  polarity remains a separate repair obligation. `PortaSample.it` crossing MAE
  decreased 6.82759 -> 2.79310; still not accepted.
- Independent review `deleg_7f7c0196` confirmed original-note pitch restoration and
  identified changed-instrument active restart, mapped-sample identity and invalid
  instrument-only-number defects. A new failing regression reproduced these;
  selection is now unified through incoming instrument + remembered original note,
  valid changed instruments retrigger active voices, and invalid instrument-only
  rows leave the old voice untouched. All three repaired locally; source-bound
  repair-wave review `deleg_e0474857` is pending.
- Runtime library: 380 tests pass. Fresh packed `PortaResetAfterRetrigger.it`
  finishes its 3.2-second arrangement; compressed/decoded equivalence passes in
  both players. Activity disagreements decreased 56 -> 12 and crossing MAE
  17.98095 -> 0.01493 after S77/S78 support. These threshold-sensitive metrics are not acceptance.
- Correct playback is unusually quiet: native PCM16 peak 73, reference peak 54.
  The unchanged shared song-pack capture helper rejects peaks <=100 after checking
  termination. Local compatibility tooling now labels nonzero peaks <=100 as a
  quiet-fixture warning; zero, clipping, other assertions and nontermination still
  fail. `python -B tools/tracker-debug/test_compatibility.py` verifies boundaries.
- `git diff --check` passes. Independent repair-wave review is in flight. Remaining
  portamento, voice/envelope, stereo, filters, corpus and rollback obligations are
  unchanged: overall status remains IN PROGRESS / NOT ACCEPTED.

## Reference and verification record

Local suite pinned to libxmp commit
`6ec0ba21b1b28f91e22b68a51d59207c6bbf6139`: 205 files verified against Git blob
identities, with SHA-256/provenance in `target/tracker-compatibility/sources.json`.
The XM/IT `00_README` explicitly invites testing other players. Use locally for
that purpose, not as game assets or an assumed redistribution grant. Two CC0
external song candidates are separately recorded in `historical.json`; a diverse
historical corpus remains outstanding. Reference binary/library version and
settings need full pinning; an FFmpeg executable hash alone is not sufficient.

Existing read-only real packed-cart mixer helper:
`../speccade/examples/rc1-signal-relay/tools/render_tracker.rs` (relative to repo).
Its build/pack integration is in
`../speccade/packs/tracker_starter_v1/native.py`. Do not modify those files.

No completion or musical-quality verdict has been issued for this contract.

## IT connected continuation — 2026-09-07

IT remains NOT ACCEPTED. Continued from the accepted bounded XM checkpoint; no
XM implementation was changed in this continuation.

- `python C:/Users/rdave/AppData/Local/Temp/it-connected-inventory.py it-connected-inventory-v2`
  exited 0: 25/25 groups pass through fresh source/cart/native and the installed
  hash-verified official reference. V1 failed only `note_fade_probe(cut=True)`:
  the reference SC0 output transient measured 12 PCM16 units in .16–.18 seconds.
  The revised check preserves pre-cut audibility, asserts strong attenuation in
  .145–.155 seconds following the .14-second cut, and silence in .18–.20 seconds.
  No native runtime fix is attributed to this fixture assertion correction.
- `python C:/Users/rdave/AppData/Local/Temp/it-historical-capture.py it-historical-after-v3`
  exited 0, but is DIAGNOSTIC, not acceptance. Original `Brute VII 2.it` SHA-256
  `6466d54bb2f873170f7ef6f762a866a7221f531724393b1fe336ec5998a77d74`;
  official executable SHA-256
  `9d809056e40e3d004b1ab9275081ee8ddb9d1874999369189d0ba172e7c3131c`.
  174-second native capture: 2115 row transitions, 4570 pitched events checked
  late in their rows, zero note mismatches, zero clipping, peak .54998779.
  Reference peak .90063477, zero clipping. Native ends order 1 row 2, wraps 1,
  still playing: this is a looping arrangement, NOT successful termination.
  Full source-order/event-count and loop-end assertions are still owed.
  V1 sampled some row states before tick-zero processing; V2 assumed fixed
  speed six and omitted faster rows. Neither is full musical-state evidence.
- `python C:/Users/rdave/AppData/Local/Temp/it-historical-rollback.py it-historical-rollback-v1`
  exited 0: three historical checkpoints (frames 7,4000,10290), snapshot handoff
  plus cold reconstruction, each compares 64 subsequent frames of PCM, tracker
  state and channel snapshots. Full native path traversed the loop boundary.
- Newly added synthetic IT rollback cases passed the focused rollback suite in
  this continuation; broader final affected suites and independent review remain.
- CONFIRMED GAP: instrument 8 (`snare`) in the historical source has random volume
  20. Full IT and NCIT retain it, but `convert_it_instrument` and
  `TrackerInstrument` omit random volume/pan and runtime never applies them.
  Reference renders vary, as expected for random volume; native is deterministic
  but presently omits the authored variation. This omission must be repaired
  with rollback-safe original code, not hidden by a global gain adjustment.
  The much larger overall native/reference peak difference is not yet explained.

No commits, source fixture/SpecCade changes, new dependencies, downloads, or
licensing changes. Existing retained diagnostic stages remain intact.

### IT random swing repair and stronger historical assertions

The missing random volume/pan conversion and live-note consumption are now repaired with an original deterministic per-channel generator. Offsets and generator state belong to the channel snapshot/NNA state. Note portamento must retain a draw rather than retrigger it; zero authored swing preserves previous behavior. XM instruments explicitly initialize zero swing. Source reference was consulted only for behavioral boundaries (new-note versus porta, bounded instrument/sample gain and pan); no generator, tables or arithmetic implementation was copied.

- `it-swing-regression-v1/before.log`: the original live-note regression fails with the application hook omitted; `after.log` passes all 9 rollback checks after restoring the hook. `build.log` records successful offline rebuild before packing.
- `it-swing-capture-v1/report.json`: fresh authored source/cart/native/official reference for no swing, 20% volume swing, and pan swing 8. Both players show bounded repeated-note variation. Native volume ratios 0.8074866–1.1764706; official 0.8423763–1.1569346. Random sequences intentionally differ, not their range contract. Pan varies both ways without changing summed level.
- `it-timing-v2/report.json`: source/cart/native/reference speed 3, tempo 150, octave pitch, cut, restart and termination checks pass. The first v1 incorrectly expected the official CLI to suppress its default 100ms end fade. v2 explicitly verifies that tail/file duration, separately from native termination at 0.82s; no runtime change or weakened cut assertion was used.
- `it-historical-after-v4/report.json`: asserts all 2,112 authored rows in exact source order, 4,570 observed source pitched events (including completed loop rows), no missing observations, zero held-note-state mismatches, explicit loop to order 1, and zero clipping. The unfinished last captured loop row is not claimed as observed.
- Serialized offline suites in `it-swing-regression-v1/{nether-tracker,nether-xm,nether-it,nethercore-zx}.log`: 53 / 67 / 62 / 478 passed, runtime 1 ignored. Includes actual predictive audio-thread IT control handoff with 4,000 subsequent PCM samples per restart/jump.

The historical amplitude difference remains unresolved; random swing did not remove it and IT is still NOT ACCEPTED. A fresh all-IT/all-XM inventory and IT historical rollback refresh is running under `proc_399a7368dc8d` with named stages `it-connected-inventory-v3`, `xm-after-it-inventory-v1`, and `it-historical-rollback-v2`. Exact commands use the retained Temp orchestration scripts with those stage arguments; no generic --case or reference download path was used.


### Explicit IT RC3 balance mixing repair (preliminary wave; superseded below)

The historical gain gap is traced to explicit `.MMP=3` in the framed `STPM`
song properties of `Brute VII 2.it`, after the `XTPM` instrument properties.
`.APS=48` agrees with its header. This is NOT evidence for replacing the ordinary
IT `/128` preamp denominator with `/64`. Standard IT and explicit mode4 retain
linear amplitude distribution. Mode3 uses balance (unity per side at center,
attenuation only toward the far side); hard-pan amplitude is unchanged.

- Original Nethercore parser locates properties after bounded structural/sample
  payload ends, not by searching arbitrary PCM for tags. Explicit preamp is
  preserved; unsupported explicit mixer values error, opaque unrelated trailing
  data is not blanket rejected. NCIT reserved byte17 (reserved[1]) distinguishes
  balance=1 from existing zero=ordinary. Pattern encoding byte16 is unchanged.
  Converter `IT_BALANCE_MIX` and the actual supplied module handle control mixing;
  snapshots retain the loaded module identity. No creator heuristics added.
- `it-balance-v1/report.json`: fresh source/cart/native/official-reference tests
  for modes4 and3 at pans0,16,32,48,64 plus mode3/preamp24/center. All11 controls
  agree within1 PCM16 unit, with no generic gain change. The source samples are
  original locally generated fixtures, not historical edits or reference assets.
- `it-balance-regressions-v1/before.json` runs the retained pre-repair renderer
  against the identical new balance cart: center187 versus required375. Old
  executable and cart SHA256 are recorded. The rebuilt renderer yields375.
  The original supplied-handle regression verifies center doubling only in
  balance mode, unchanged hard pans, and fresh snapshot recipient playback.
- Serialized offline tests in `it-balance-regressions-v1/*.log`: converter53,
  IT64, XM67, runtime479 passed (runtime1 ignored). All six TEMP/TMP/TMPDIR and
  lowercase aliases set to C:/Users/rdave/AppData/Local/Temp; CLI/runtime rebuilt
  offline before controls and repacking.
- `it-historical-balance-v1/report.json`: native peak0.89251708984375, official
  0.90997314453125; zero clipping both. Source hash unchanged, all2112 authored
  rows in exact order,4570 completed-row pitched events, zero held-note mismatch,
  explicit loop to order1. Reference random sequence/interpolation differs;
  peak proximity is supplemental, not a musical-state criterion.
- Corrected historical rollback orchestration: checkpoint10370 now compares64
  subsequent frames across the actual loop near10423. Earlier checkpoint10290
  did NOT compare across that boundary (although the whole render traversed it).
  The new run uses freshly packed `it-historical-balance-v1/capture.nczx`, not an
  old cart missing the balance bit.
- In progress: `proc_8bc0509d9e05` serially runs `it-balance-inventory-v1`,
  `xm-after-it-balance-v1`, `it-historical-balance-rollback-v1`, and
  `xm-after-it-balance-historical-v1 --require-pass` via their retained Temp
  orchestration scripts. Fresh independent source-bound review `deleg_cf5eefce`
  is also in progress. IT remains NOT ACCEPTED until those results are inspected.

External behavioral oracle source pin remains
`f83cedb0cd5446e4dfaa83ac97e3087107e26767`; cached mix configuration and complete
caller pan/preamp flow were consulted, not copied. Official executable remains
SHA256 `9d809056e40e3d004b1ab9275081ee8ddb9d1874999369189d0ba172e7c3131c`;
source-pin provenance is separate from executable provenance. No dependencies,
reference downloads, licensing changes, SpecCade edits, fixture edits or commits.

## Final connected XM → IT scoped acceptance (2026-09-07)

**XM and IT VALIDATED/WORKING for the following tested scope.** This completes
the connected, explicitly bounded validation requested in the active continuation;
it is not universal XM/IT compatibility or acceptance of the broader corpus goal.
No human listening verdict, release, merge or network-play claim is made.

### Tested scope and real-path results

- XM: `xm-after-it-balance-v1/inventory.json` and `captures.json` verify 25/25
  groups, 72 fresh source/cart/native/official-reference captures. The previously
  accepted timing/tuning, sample-map/loop, simultaneous-effect/memory, envelope,
  mode-aware pan/preamp and flow scope is preserved. Fresh historical artifact
  `xm-after-it-balance-historical-v1/report.json`: 1,344 source rows, 2,954 pitched
  events, exact row order, zero note/held-state mismatches, clean end at order21
  row0 without wraps, zero clipping; native peak .9616088867, reference .9602966309.
  Existing XM historical mid-tick/handoff/cold/repeated rollback evidence remains
  `xm-historical-rollback-v3`; current 479-test runtime suite passes its XM tests,
  including rate-change reconstruction and actual audio-thread control handoff.
- IT: `it-inventory-final-v1/inventory.json` and `captures.json` verify 25/25
  groups, 126 fresh captures. This includes the named instrument/sample selection,
  portamento/memory, sample restart/loop, panning/default/envelope, NNA/DCT/DCA,
  note-fade/cut/invalid-map and flow probes, not every possible interaction.
  `it-timing-final-v1/report.json` independently verifies speed3/tempo150, octave
  pitch, note cut/restart and finite end; native and reference checks both pass.
  `it-swing-final-v1/report.json` verifies original deterministic volume/pan swing
  through freshly packed playback; exact random sequences are not expected to
  match the official player's independently seeded random variation.
- `it-balance-final-v1/report.json`: all 11 mode4/mode3, five-position pan and
  intermediate-preamp controls pass within one PCM16 unit against the official
  player. Mode3 center is375 per side versus ordinary187; hard-pan375 is unchanged.
  Original mode-aware mixer uses the supplied loaded-module handle. Snapshot
  recipient and mode-selection regression passes. No blanket gain change.
- `it-historical-final-v1/report.json`: original source SHA256
  `6466d54bb2f873170f7ef6f762a866a7221f531724393b1fe336ec5998a77d74`, all2,112
  authored rows in exact order,4,570 pitched events, zero note/held-state mismatches,
  explicit loop to order1 and zero clipping. Native peak .89251708984375;
  reference .908935546875. The final incomplete loop row is not falsely counted.
  This looping song remains playing at order1,row2 after174s: not a finite stop.
  Absolute gain/pan is established by discriminating controls, not peak proximity.
- `it-rollback-final-v1/native.log` and `rollback-renderer.rs`: actual final
  NCIT-v2 historical cart; checkpoints7,4000,10370; handoff and cold reconstruction
  compare64 subsequent PCM/state frames each with uninterrupted execution. Actual
  loop transition is frame10422, within compared interval[10370,10434), verified
  from the row log. Earlier10290 checkpoint did not prove this crossing. Current
  synthetic IT tests additionally cover deterministic NNA/swing, seeks/restarts,
  repeated reconstruction and predictive audio-thread control handoff.

### Final review repairs and regression evidence

Independent source review `deleg_cf5eefce` identified packed count/row/marker
bounds, legacy reserved-byte interpretation, and populated sample offset0 defects.
Owner inspected and repaired those concrete findings, then exercised the revised
source and real path rather than accepting the worker summary. Final semantics:

- **NCIT version2** (byte16) adds the balance discriminator in byte17. Versions0
  and1 remain readable with their former marker encodings and ignored reserved
  bytes; unrelated reserved byte values cannot retroactively change their mixing.
  Preliminary version1 balance carts above are experimental evidence only, not
  the final format: final acceptance uses newly repacked version2 artifacts.
- Count/channel/row bounds are checked; modern invalid channel markers no longer
  alias valid channels. Populated sample offset0 errors, while absent sample data
  remains valid. PCM width and compressed-stereo block boundaries are checked.
  Tagged instrument sections cannot be mistaken for song mixing properties;
  opaque trailing data is not blanket rejected. Legacy sample stripping now
  preserves explicitly parsed balance metadata as well.
- `it-review-final-v1/*-before.log`: restoring the reviewed legacy-reserved and
  offset0 behavior makes the new focused regressions fail. Corrected behavior
  passes. Original mixer fail-before evidence remains
  `it-balance-regressions-v1/before.json` (retained old renderer187 vs required375).
- `it-review-final-v1/{nether-it,nether-tracker,nether-xm,nethercore-zx}.log`:
  **66 / 53 / 67 / 479 pass**, runtime1 ignored; `build.log` confirms offline
  CLI/runtime rebuild before final packing. Python compatibility harness check
  and `git diff --check` also passed. No new dependency or framework.

### Exact commands, environment and provenance

All Cargo calls used the following six explicit environment assignments; Python
orchestration assigns the same six variables to its subprocess environment:

```sh
env TEMP=C:/Users/rdave/AppData/Local/Temp TMP=C:/Users/rdave/AppData/Local/Temp TMPDIR=C:/Users/rdave/AppData/Local/Temp temp=C:/Users/rdave/AppData/Local/Temp tmp=C:/Users/rdave/AppData/Local/Temp tmpdir=C:/Users/rdave/AppData/Local/Temp cargo test --offline -p nether-it --lib -- --test-threads=1
```

The same invocation was run serially with `-p nether-tracker`, `-p nether-xm`,
and `-p nethercore-zx`; then the same environment with
`cargo build --offline -p nether-cli -p nethercore-zx`. All exited0.
From this repository, each following command exited0:

```sh
python C:/Users/rdave/AppData/Local/Temp/it-balance-capture.py it-balance-final-v1
python C:/Users/rdave/AppData/Local/Temp/it-historical-capture.py it-historical-final-v1
python C:/Users/rdave/AppData/Local/Temp/it-historical-rollback.py it-rollback-final-v1
python C:/Users/rdave/AppData/Local/Temp/it-connected-inventory.py it-inventory-final-v1
python C:/Users/rdave/AppData/Local/Temp/it-swing-capture.py it-swing-final-v1
python C:/Users/rdave/AppData/Local/Temp/it-timing-capture.py it-timing-final-v1
python C:/Users/rdave/AppData/Local/Temp/xm-connected-inventory.py xm-after-it-balance-v1
python C:/Users/rdave/AppData/Local/Temp/xm-connected-capture.py xm-after-it-balance-historical-v1 --require-pass
python -B tools/tracker-debug/test_compatibility.py
git diff --check
```

Artifacts above are under `target/tracker-compatibility/`; the Temp orchestration
scripts reuse inspected `capture_case`, `compile_compat_renderer` and the existing
read-only SpecCade helper. They are orchestration, not standalone proof.
Final IT runtime SHA256:
`85e4c654e2eb218b7aea86272414b413048cce5641244d0a405c72c6f189c48a`.
Working tree remains dirty on HEAD `7dac9fb81efa0a539c9211d38b690fbb986f5a1d`;
HEAD alone is not the build identity. Artifact reports retain source/cart/helper
and runtime hashes. Official executable SHA256 remains
`9d809056e40e3d004b1ab9275081ee8ddb9d1874999369189d0ba172e7c3131c`;
external behavioral-source pin remains
`f83cedb0cd5446e4dfaa83ac97e3087107e26767`, independently identified from that binary.

### Explicit limits and preserved boundaries

The inventory files enumerate what passed. No universal creator/extension support,
every effect interaction, arbitrary stereo sample fidelity, all filter modes,
all sample rates, every historical tracker revision, diverse historical corpus,
or user listening verdict is inferred. Those untested surfaces are not silently
marked compatible; a demonstrated standard defect still requires repair.
Ordinary generated controls, these two historical fixtures and the named native
rollback paths have no known unresolved blocking discrepancy in the tested scope.
No general gain workaround, music/SpecCade edits, downloaded reference/fixtures,
license/attribution changes, dependencies, commits, pushes, releases, or deletion
of existing artifacts. Existing unrelated dirty work is preserved.
