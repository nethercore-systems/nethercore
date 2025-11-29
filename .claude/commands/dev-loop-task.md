---
description: "Implement tasks from the task list"
---

## Process

1. **SELECT**: Pick 1 task (or 2-3 related) from `## TODO`. [STABILITY] tasks first.

2. **PLAN**: Before coding, consider approaches, trade-offs, existing patterns, edge cases.

3. **CHECK AMBIGUITY**: If task is vague or needs architectural decisions you can't make:
   - Add `[NEEDS CLARIFICATION]` prefix with note: `- [NEEDS CLARIFICATION] Task (needs: what?)`
   - Skip to a clearer task

4. **START**: IMMEDIATELY move selected task(s) from `## TODO` to `## In Progress`. This shows what you're actively working on.

5. **IMPLEMENT**: Write the code, run tests.

6. **COMPLETE**: If tests pass, move task(s) from `## In Progress` to `## Done`, then commit and push: `git add -A && git reset .claude/settings.local.json 2>/dev/null; git commit -m "Complete: [desc]" && git push`

If tests fail, move task back to TODO and revert code changes. Quality over speed.
