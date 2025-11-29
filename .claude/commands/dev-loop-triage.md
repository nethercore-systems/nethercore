---
description: "Validate and repair task file format, prioritize tasks"
---

## 1. VALIDATE FORMAT

Check that the task file has the required structure:
```markdown
## TODO
- Task items here

## In Progress
- Active tasks here

## Done
- Completed tasks here
```

### Common Issues to Fix:
- Missing `## TODO`, `## In Progress`, or `## Done` sections → Add them
- Tasks not starting with `- ` → Fix the prefix
- Empty lines within task items → Remove them
- Duplicate tasks → Remove duplicates (keep in highest-priority section)
- Tasks in wrong section (e.g., completed task in TODO) → Move appropriately
- Malformed section headers (e.g., `##TODO` without space) → Fix spacing
- Mixed formats (checkboxes `- [ ]` mixed with plain `- `) → Normalize to plain `-`

## 2. REPAIR

If format issues found:
1. Create backup: copy file to `{filename}.backup`
2. Fix all detected issues
3. Preserve task content and any prefixes like `[STABILITY]`

## 3. PRIORITIZE

Reorder `## TODO` section:
1. `[STABILITY]` tasks first (critical fixes)
2. `[NEEDS CLARIFICATION]` tasks last (blocked)
3. Group related tasks together
4. Quick wins before complex tasks

## 4. REPORT

Output a summary:
- Format issues found and fixed
- Task counts per section
- Any warnings (e.g., stale In Progress items)

## 5. COMMIT

If changes made:
```bash
git add -A && git reset .claude/settings.local.json 2>/dev/null; git commit -m "Triage: Validate and reorganize tasks"
```
