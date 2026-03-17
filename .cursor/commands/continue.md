# Continue Current Task

You are working with the Memory Bank System. Resume work on the current task from where it was left off.

## Instructions

1. **Read current state**:
   - `memory-bank/activeContext.md` - Current active task
   - `memory-bank/tasks.md` - Task details
   - `memory-bank/progress.md` - Current phase and progress

2. **Determine current phase**:
   - VAN (Initialization)
   - PLAN (Planning)
   - CREATIVE (Design exploration) - Level 3-4 only
   - DO (Implementation)
   - REFLECT (Review)
   - ARCHIVE (Archival)

3. **Resume work based on phase**:
   - If no active task → Inform user to run `/van`
   - If in VAN → Complete task initialization
   - If in PLAN → Continue or complete planning
   - If in CREATIVE → Continue design exploration
   - If in DO → Continue implementation
   - If in REFLECT → Complete reflection
   - If ready for ARCHIVE → Suggest running `/archive`

4. **Show context**:
   - Brief summary of where we are
   - What's been done
   - What needs to be done next

5. **Execute next appropriate action** automatically

## Memory Bank Integration

- Read from: `memory-bank/activeContext.md`, `memory-bank/tasks.md`, `memory-bank/progress.md`
- Write to: Depends on current phase
- MCP servers: Use sys8 for timestamps

## Rules to Follow

Load these rules in order:
1. Core rules: `.cursor/rules/isolation_rules/main.mdc`
2. Memory Bank paths: `.cursor/rules/isolation_rules/Core/memory-bank-paths.mdc`
3. MCP integration: `.cursor/rules/sys8-mcp-usage.mdc`, `.cursor/rules/context7-mcp-usage.mdc`
4. AI Quality principles: `.cursor/rules/isolation_rules/Core/AI-Quality/_principles.mdc`
5. Phase-specific rules (loaded dynamically based on current phase)

## Example Usage

```
/continue
```

## Example Output

```
Resuming Task: TASK-20250116-001 - Add user authentication
Current Phase: DO (Implementation)
Progress: 40% complete - AuthService implemented, tests passing

Continuing implementation:
- Next: Implement AuthProvider component
- Following: Add integration tests
- TDD: Writing tests first...
```
