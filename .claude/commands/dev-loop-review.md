---
description: "Review codebase for stability issues"
---

Check for: untested code, unhandled errors (unwrap/panic), missing docs, DRY violations, dead code, TODOs/FIXMEs.

If issues found, add to TODO with [STABILITY] prefix. Be conservative - real issues only.

Commit and push: `git add -A && git reset .claude/settings.local.json 2>/dev/null; git commit -m "Review: Add stability tasks" && git push`
