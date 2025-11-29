---
description: "Refactor code for maintainability"
---

Target: DRY violations, long functions, unclear names, dead code, magic numbers.

Process:
1. Make ONE small refactoring
2. Run tests
3. If pass: commit and push `git add -A && git reset .claude/settings.local.json 2>/dev/null; git commit -m "Refactor: [desc]" && git push`
4. Repeat 2-3 times

Rules:
- Tests MUST pass after each change
- Do NOT add features or change behavior
- If tests fail, revert and try different approach
