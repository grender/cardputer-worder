# PLAN Command - Task Planning

This command creates detailed implementation plans based on complexity level determined in VAN mode, strictly following the **Enhanced Design Process** (Phases 4-6).

## Memory Bank Integration

Reads from:
- `memory-bank/tasks.md` - Task requirements and complexity level
- `memory-bank/activeContext.md` - Current project context
- `memory-bank/projectbrief.md` - Project foundation (if exists)

Updates:
- `memory-bank/tasks.md` - Adds detailed implementation plan

## Progressive Rule Loading

### Step 1: Load Core Rules
```
Load: .cursor/rules/isolation_rules/main.mdc
Load: .cursor/rules/isolation_rules/Core/memory-bank-paths.mdc
```

### Step 2: Load PLAN Mode Map
```
Load: .cursor/rules/isolation_rules/visual-maps/plan-mode-map.mdc
```

### Step 3: Load Complexity-Specific Planning Rules
Based on complexity level from `memory-bank/tasks.md`:

**Level 2:**
```
Load: .cursor/rules/isolation_rules/Level2/task-tracking-basic.mdc
Load: .cursor/rules/isolation_rules/Level2/workflow-level2.mdc
```

**Level 3:**
```
Load: .cursor/rules/isolation_rules/Level3/task-tracking-intermediate.mdc
Load: .cursor/rules/isolation_rules/Level3/planning-comprehensive.mdc
Load: .cursor/rules/isolation_rules/Level3/workflow-level3.mdc
```

**Level 4:**
```
Load: .cursor/rules/isolation_rules/Level4/task-tracking-advanced.mdc
Load: .cursor/rules/isolation_rules/Level4/architectural-planning.mdc
Load: .cursor/rules/isolation_rules/Level4/workflow-level4.mdc
```

### MANDATORY Load Rules:
```
Load: .cursor/rules/sys8-mcp-usage.mdc
Load: .cursor/rules/context7-mcp-usage.mdc
```

## Workflow

1.  **Read Task Context**
    -   Read `memory-bank/tasks.md` to get complexity level and requirements.
    -   Read `memory-bank/activeContext.md` for current context.
    -   Review `memory-bank/prd/*.md` if available.

2.  **Phase 4: Detailed Design**
    -   **Component Breakdown**: Expand selected approach into specific file changes.
    -   **Interface Design**: Define function signatures, API contracts.
    -   **Data Flow**: Trace input -> processing -> output.
    -   **Security Design**: Perform **Threat Modeling** and map to **Security Controls** (Appendix A).

3.  **Phase 5: Implementation Plan Creation**
    -   Create a comprehensive plan in `memory-bank/tasks.md` using the **Design Document Template**.
    -   Include **Rollback Strategy** and **Validation Commands**.

4.  **Phase 6: Documentation Updates**
    -   Identify which project docs need updating (Architecture, Security, API, etc.).

5.  **Technology Validation** (Level 2-4)
    -   Document technology stack selection.
    -   Verify dependencies and build configuration.

6.  **Update Memory Bank**
    -   Update `memory-bank/tasks.md` with the complete plan.
    -   Mark planning phase as complete.

## Implementation Plan Template (Design Document)

The plan in `memory-bank/tasks.md` MUST follow this structure (adapted from *Enhanced Design Process* Phase 5):

```markdown
# [Feature Name] Implementation Plan

## 1. Overview
[Problem statement, goals, success criteria]

## 2. Security Summary
- **Attack Surface**: [Increased/Decreased/Unchanged]
- **New Permissions**: [List or "None"]
- **Sensitive Data**: [Yes/No - describe handling]
- **Risks**: [List identified risks]

## 3. Architecture Impact
- **Components**: [List affected components]
- **Integration**: [Diagram or description of integration points]

## 4. Detailed Design

### 4.1 Component Changes
- **File**: `path/to/file`
- **Changes**: [Description]
- **Reason**: [Why needed]

### 4.2 New Components
- **File**: `path/to/new_file`
- **Purpose**: [What it does]
- **Dependencies**: [Imports/Uses]

### 4.3 API Changes
- **Endpoint**: [METHOD /path]
- **Auth**: [Required/Optional]
- **Validation**: [Rules applied]

### 4.4 Database Changes
- **Table**: [Name]
- **Migration**: [Description]

## 5. Security Design (Appendix A)

### 5.1 Threat Model
- **Assets**: [Data/Resources at risk]
- **Threats**: [Malicious actors/vectors]
- **Mitigations**: [Prevention strategies]

### 5.2 Security Controls Checklist
- [ ] Input Validation (All boundaries)
- [ ] Output Encoding (HTML, SQL, Shell)
- [ ] Access Control (AuthN/AuthZ, Least Privilege)
- [ ] Secrets Protection (No hardcoded secrets)
- [ ] Infrastructure (Minimal privileges)

## 6. Implementation Steps

### Step 1: [Name]
**Files:**
- `path/to/file`
**Changes:**
```php
// Specific code example
```
**Rationale:** [Why this step first]

### Step 2: [Name]
...

## 7. Test Plan
- **Unit Tests**: [Files/Scenarios]
- **Integration Tests**: [Cross-component]
- **Security Tests**: [Input validation, Injection, Auth bypass]

## 8. Rollback Strategy
```bash
# Git revert or migration down commands
php yii migrate/down
git checkout HEAD~1
```

## 9. Validation Checklist
- [ ] All implementation steps completed
- [ ] Unit & Integration tests passing
- [ ] Security controls verified (Appendix A)
- [ ] Feature works as specified
- [ ] No regressions
- [ ] Documentation updated

## 10. Next Steps
- Proceed to `/do` command.
```

## Security Requirements (Appendix A Reference)

**Principles:**
- Fail-closed (deny by default).
- Least privilege.
- No secrets in code/logs.

**Anti-Patterns (Reject these):**
- Trusting user input.
- Logging sensitive data.
- Hardcoding secrets.
- SQL string concatenation.
- Unvalidated file paths.

## Usage

Type `/plan` to generate this detailed plan in `memory-bank/tasks.md`.
