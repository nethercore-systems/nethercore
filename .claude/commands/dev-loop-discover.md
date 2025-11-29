---
description: "Discover new tasks from codebase analysis"
---

Analyze: unimplemented features (README/docs), TODO/FIXME comments, untested code, weak error handling.

Add specific tasks to TODO section. Reference files/functions. No vague tasks.

Commit and push: `git add -A && git reset .claude/settings.local.json 2>/dev/null; git commit -m "Discover: Add tasks from codebase analysis" && git push`
