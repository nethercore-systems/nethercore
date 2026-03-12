# Review Pack

Use this pack for adversarial screenshot review only.

Treat review runs as fresh-context passes. Rehydrate from the listed files and the newest screenshots instead of trusting prior session memory.

## Read Order

1. `agent/session-protocol.md`
2. `agent/shared/06-open-gaps.md`
3. `agent/shared/10-program-runbook.md`
4. `agent/shared/08-benchmark-suite.md`
5. `agent/shared/09-benchmark-log.md`
6. `agent/shared/07-preset-briefs.md`
7. `agent/shared/04-capture-log.md`
8. `agent/shared/05-review-log.md`
9. `agent/shared/03-validation-playbook.md`
10. `agent/shared/12-live-workbench.md`
11. `agent/shared/01-roster.md`
12. `agent/shared/00-charter.md`

## Objective

- inspect the newest screenshot batch directly
- if the batch comes from the live workbench, judge the full-frame, background-first, and probe-first capture set before deciding whether the direction deserves replay promotion
- if the batch is a benchmark batch, judge the benchmark contract first and decide whether the change deserves promotion to a full showcase pass
- score presets harshly
- remember that the capture contains both a direct background read and a reflective hero probe read; judge both, and do not mistake the probe for HUD/UI or for a separate artifact
- judge against the preset brief and the preset's `Text prompt:` line, not just overall attractiveness
- judge EPU as metaphor-first procedural world art for ambient/reflection/direct-view shipping use, not as a literal prop renderer
- judge distinctness against the rest of the roster, not just the single preset in isolation
- verify that the preset is visibly animated across the reviewed frames
- fail any visible artifact or obvious rendering error
- call out looping or repeated patterning explicitly
- separate likely content defects from opcode-surface/tooling gaps and suspected engine or rendering defects
- fail anything that looks uncertain
- record concrete reasons for each failure

## Boundaries

- no code edits
- no replay edits
- no silent overwriting of prior reviews

## Required Output

Append dated review entries to `05-review-log.md` and update `06-open-gaps.md` with concrete fix targets, opcode-surface/tooling gap notes, or suspected engine-bug notes. Each review entry should state which text-prompt and brief reads were achieved, which were missed, whether any hard-fail trigger appeared, whether the result is shippable for EPU's ambient/reflection/direct-view role, and whether roster-level distinctness survived.

If the batch is a benchmark batch, update `09-benchmark-log.md` with whether the change is promoted, blocked, or still ambiguous.

If the batch comes from the live workbench, update mutable state with whether the direction is worth exporting and replay-promoting, blocked, or still ambiguous.

When writing review notes, explicitly call out whether the key read survives:

- in direct background view
- in the reflective probe
- in both
