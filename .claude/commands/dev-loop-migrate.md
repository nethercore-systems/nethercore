---
description: "Convert task file to dev-loop section format"
---

Convert an existing task file to dev-loop's required format.

## Target Format

```markdown
## TODO
- Task items go here

## In Progress

## Done
- Completed tasks go here
```

## Process

1. **READ**: Read the task file (TASKS.md, TODO.md, or specified file)

2. **DETECT FORMAT**: Identify current format:
   - Checkbox format: `- [ ]` (unchecked), `- [x]`/`- [X]` (checked)
   - Plain list: `- item` or `* item`
   - Numbered list: `1. item`
   - Other custom formats

3. **CONVERT**:
   - Unchecked items (`- [ ]`, plain `-`, `*`, numbered) → `## TODO`
   - Checked items (`- [x]`, `- [X]`) → `## Done`
   - Items marked "in progress", "WIP", "started" → `## In Progress`
   - Strip checkbox markers, keep task text
   - Preserve any `[STABILITY]` or other prefixes

4. **PRESERVE**:
   - Keep any file header/title (e.g., `# Project Tasks`)
   - Keep any description text before the task lists
   - Keep task ordering within each section

5. **WRITE**: Write the converted format back to the file

6. **BACKUP**: Create a `.backup` copy of the original file

Example conversion:
```markdown
# Project Tasks
- [ ] Add user auth
- [x] Setup database
- [ ] Write tests
```
Becomes:
```markdown
# Project Tasks

## TODO
- Add user auth
- Write tests

## In Progress

## Done
- Setup database
```

Commit: "Migrate: Convert task file to section format"
