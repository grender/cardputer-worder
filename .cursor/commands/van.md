# VAN Command - Initialization & Entry Point

This command initializes the Memory Bank system, performs platform detection, determines task complexity, and routes to appropriate workflows.

## Memory Bank Integration

**CRITICAL:** All Memory Bank files are located in `memory-bank/` directory:
- `memory-bank/tasks.md` - Source of truth for task tracking
- `memory-bank/activeContext.md` - Current focus
- `memory-bank/progress.md` - Implementation status
- `memory-bank/projectbrief.md` - Project foundation
- `memory-bank/backlog.md` - Pending task queue (NEW)

## Progressive Rule Loading

This command loads rules progressively to optimize context usage:

### Step 1: Load Core Rules (Always Required)
```
Load: .cursor/rules/isolation_rules/main.mdc
Load: .cursor/rules/isolation_rules/Core/memory-bank-paths.mdc
Load: .cursor/rules/isolation_rules/Core/platform-awareness.mdc
Load: .cursor/rules/isolation_rules/Core/file-verification.mdc
Load: .cursor/rules/isolation_rules/Core/backlog-management.mdc
```

### Step 2: Load VAN Mode Map
```
Load: .cursor/rules/isolation_rules/visual-maps/van_mode_split/van-mode-map.mdc
```

### Step 3: Load Complexity-Specific Rules (Based on Task Analysis)
After determining complexity level, load:
- **Level 1:** `.cursor/rules/isolation_rules/Level1/workflow-level1.mdc`
- **Level 2-4:** Load plan mode rules (transition to PLAN command)

### MANDATORY Load Rules:
```
Load: .cursor/rules/sys8-mcp-usage.mdc
```

## Workflow

1. **Platform Detection**
   - Detect operating system
   - Adapt commands for platform
   - Set path separators

2. **Memory Bank Verification**
   - Check if `memory-bank/` directory exists
   - If not, create Memory Bank structure
   - Verify essential files exist

3. **Backlog Check** (NEW)
   - Check if `memory-bank/backlog.md` exists
   - If exists, parse for pending items
   - If pending items found, display summary and offer selection
   - User can select from Backlog OR start new task

4. **Task Analysis**
   - Read `memory-bank/tasks.md` if exists
   - Analyze task requirements (from input or selected Backlog item)
   - Determine complexity level (1-4)
   
   **Context Gathering Note:**
   If starting a new task, consider if you need to run `/prd` first to gather context and explore solutions, especially for complex features (Level 3-4).

5. **Route Based on Complexity**
   - **Level 1:** Continue in VAN mode, proceed to implementation
   - **Level 2-4:** Transition to `/plan` command

6. **Update Memory Bank**
   - Update `memory-bank/tasks.md` with complexity determination
   - Update `memory-bank/activeContext.md` with current focus
   - If from Backlog: Update `memory-bank/backlog.md` item status to `in_progress`

## Backlog Integration

When `/van` is called, the command MUST:

1. **Check Backlog existence:**
   ```
   if exists(memory-bank/backlog.md):
       pending_items = parse_pending_items()
       if pending_items.length > 0:
           display_backlog_summary()
           prompt_selection()
   ```

2. **Display Backlog summary:**
   ```
   📋 Backlog Status: N pending items
   
   | ID | Title | Priority | Complexity |
   |----|-------|----------|------------|
   | BACKLOG-0001 | Task title | high | Level 2 |
   | BACKLOG-0002 | Another task | medium | Level 1 |
   
   Options:
   1. Select from Backlog: Enter ID (e.g., BACKLOG-0001)
   2. Start new task: Describe your task
   3. Skip Backlog: /van [task description]
   ```

3. **When selecting from Backlog:**
   - Find item by BACKLOG-XXXX ID
   - Update item status to `in_progress`
   - Extract task details (title, description, complexity)
   - Create entry in `memory-bank/tasks.md`
   - Set `memory-bank/activeContext.md`
   - Continue with normal workflow

4. **When starting new task:**
   - Continue normal VAN workflow
   - Optionally offer: "Add to Backlog for later?"

## Usage

Type `/van` followed by your task description or initialization request.

**Start new task:**
```
/van Initialize project for adding user authentication feature
```

**Select from Backlog:**
```
/van
(System shows Backlog items, user selects BACKLOG-0001)
```

**Use specific Backlog item:**
```
/van BACKLOG-0001
```

**Skip Backlog check:**
```
/van --new Add user authentication feature
```

## Next Steps

- **Level 1 tasks:** Proceed directly to `/do` command
- **Level 2-4 tasks:** Use `/plan` command for detailed planning
- **Complex Features:** Consider running `/prd` before `/plan` for detailed requirements and solution exploration.
