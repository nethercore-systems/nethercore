---
description: "Improve codebase documentation"
---

Review and improve: README accuracy, doc comments on public API, inline comments for complex logic.

Rules:
- Do NOT change code behavior
- Only add docs that genuinely help understanding

Commit and push: `git add -A && git reset .claude/settings.local.json 2>/dev/null; git commit -m "Docs: Improve documentation" && git push`
