---
description: "Resolve tasks marked [NEEDS CLARIFICATION]"
---

For each `[NEEDS CLARIFICATION]` task:

1. Analyze codebase for context (check ## Done for patterns, CLAUDE.md for decisions)
2. If resolvable: remove prefix, rewrite as specific actionable task
3. If needs human input: add `<!-- QUESTION: ... -->` comment

Examples:
- Before: `[NEEDS CLARIFICATION] Add caching (needs: what? TTL?)`
- After: `Add Redis cache for API responses, 5min TTL (see src/cache/)`

Commit and push: `git add -A && git reset .claude/settings.local.json 2>/dev/null; git commit -m "Clarify: Resolve ambiguous tasks" && git push`
