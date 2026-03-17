# COMPLIANCE Command - Post-QA Hardening & PRD Revalidation

This command runs the 7-step compliance workflow: re-validate changes vs PRD/task, simplify code (Code Simplifier), check references/unused, coverage, linters/formatters, tests, optional hardening. Produces a compliance report and chat summary.

## Context (optional, when present in project)

Reads from (if project has memory-bank): activeContext.md, tasks.md, prd/. Workflow is defined in .cursor rules (compliance-command.mdc); no external spec file required.

Writes: If project has `memory-bank/reports/`, write compliance-report-[task_id]-[date].md there; otherwise output report in chat only.

## Progressive Rule Loading

### Step 1: Load Core Rules
```
Load: .cursor/rules/isolation_rules/main.mdc
Load: .cursor/rules/isolation_rules/Core/memory-bank-paths.mdc
```

### Step 2: Load COMPLIANCE Mode Map and Command
```
Load: .cursor/rules/isolation_rules/visual-maps/compliance-mode-map.mdc
Load: .cursor/rules/isolation_rules/visual-maps/compliance-code-simplifier.mdc
Load: .cursor/rules/cursor-commands/compliance-command.mdc
```

### MANDATORY Load Rules:
```
Load: .cursor/rules/sys8-mcp-usage.mdc
```

## Workflow

1. **Workflow definition** — In compliance-command.mdc and compliance-mode-map.mdc (self-contained in .cursor).
2. **Get context** — If project has memory-bank: activeContext.md, tasks.md, PRD. Else use Git diff or task description. Get changed files from Git diff or context.
3. **Execute steps 1–7** in order. For step 2, apply compliance-code-simplifier.mdc to recently modified code.
4. **Write report** — If `memory-bank/reports/` exists, write file there (task_id, date from sys8 MCP). Else output report in chat.
5. **Summarize in chat** — Key results and any failures; next step: /reflect.

## Usage

Type `/compliance` after implementation (/do) and QA to run post-QA hardening and PRD revalidation.

## Next Steps

After compliance complete, proceed to **/reflect** for task review.
