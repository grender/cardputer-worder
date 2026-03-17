# Check Current Task Status

You are working with the Memory Bank System. Show the current task status, Backlog summary, and suggest next steps.

## Instructions

1. **Read current state**:
   - `memory-bank/activeContext.md` - Current active task
   - `memory-bank/tasks.md` - Task details and progress
   - `memory-bank/progress.md` - Current phase
   - `memory-bank/backlog.md` - Pending tasks queue (NEW)

2. **Display status information**:
   - Task ID and description
   - Complexity level
   - Current phase in workflow
   - Progress summary
   - What's been completed
   - What's remaining

3. **Display Backlog summary** (NEW):
   - Count by status (pending, in_progress, completed)
   - List pending items (ID, Title, Priority)
   - Highlight high-priority items

4. **Suggest next steps**:
   - What should be done next?
   - Which command to run?
   - Any blockers or issues?
   - Available Backlog items to work on

5. **Show recent activity** (if available):
   - Last updated timestamp
   - Recent changes
   - Files modified

## Memory Bank Integration

- Read from: `memory-bank/activeContext.md`, `memory-bank/tasks.md`, `memory-bank/progress.md`, `memory-bank/backlog.md`
- Write to: None (read-only command)
- MCP servers: Use sys8 for current timestamp comparison

## Rules to Follow

Load these rules in order:
1. Core rules: `.cursor/rules/isolation_rules/main.mdc`
2. Memory Bank paths: `.cursor/rules/isolation_rules/Core/memory-bank-paths.mdc`
3. Backlog rules: `.cursor/rules/isolation_rules/Core/backlog-management.mdc`
4. MCP integration: `.cursor/rules/sys8-mcp-usage.mdc`, `.cursor/rules/context7-mcp-usage.mdc`

## Example Usage

```
/status
```

## Example Output

```
📊 Memory Bank Status

═══════════════════════════════════════════════
CURRENT TASK
═══════════════════════════════════════════════
Task: Add user authentication
Complexity: Level 3 (Complex feature with design exploration)
Phase: CREATIVE (Exploring design options)
Progress: 60% complete

Completed:
✓ VAN - Task initialized
✓ PLAN - Implementation plan created
⚠ CREATIVE - In progress (2 of 3 components designed)

═══════════════════════════════════════════════
BACKLOG SUMMARY
═══════════════════════════════════════════════
📋 Total: 5 items
├── Pending: 3
├── In Progress: 1 (current task)
└── Completed: 1

Pending Items:
│ ID           │ Title                    │ Priority │ Complexity │
├──────────────┼──────────────────────────┼──────────┼────────────┤
│ BACKLOG-0004 │ Database migration       │ high     │ Level 2    │
│ BACKLOG-0005 │ Caching layer            │ medium   │ Level 3    │
│ BACKLOG-0006 │ Logging improvements     │ low      │ Level 1    │

Archive Summary (from backlog-archive.md):
📦 Total Archived: {n} items
├── Completed: {n}
└── Cancelled: {n}

═══════════════════════════════════════════════
NEXT STEPS
═══════════════════════════════════════════════
1. Complete design exploration for AuthProvider component
2. Run /creative to finalize designs
3. Proceed to /do for implementation

After completing current task:
→ Use /van to select from Backlog (3 pending items)
```
