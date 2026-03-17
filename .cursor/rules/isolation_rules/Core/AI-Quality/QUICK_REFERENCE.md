# AI QUALITY RULES - QUICK REFERENCE

> **15 Rules for Better AI-Assisted Development**
> Improve code quality by 30-50%, reduce bugs by 40-50%

---

## 🚀 QUICK START

### The 5 Pillars

1. **DECOMPOSITION** - Break into small units (max 50 lines)
2. **TEST-FIRST** - Tests before code (hallucination filters)
3. **ARCHITECTURE-FIRST** - Approve structure before coding
4. **FOCUSED WORK** - One method at a time
5. **CONTEXT MANAGEMENT** - Right info at right time

---

## 📋 RULES BY MODE

### VAN Mode (Initialization)
- **Rule #4:** Requirements - Gather context BEFORE coding
- **Rule #12:** Complexity Check - Verify AI can solve this
- **Rule #14:** MB Structure - Organize hierarchically

### PLAN Mode (Planning)
- **Rule #1:** Stubbing - Break into max 50-line stubs
- **Rule #5:** DoD - Create testable done criteria
- **Rule #6:** Corner Cases - List boundaries before coding
- **Rule #7:** Skeleton-First - Create stubs before implementation
- **Rule #11:** Boundaries - State what we DON'T do

### CREATIVE Mode (Design)
- **Rule #6:** Corner Cases - Document boundaries upfront
- **Rule #7:** Skeleton-First - Create architecture before code
- **Rule #9:** Cognitive Load - Keep designs simple (≤9 components)
- **Rule #13:** Transaction - Explicit isolation levels

### DO Mode (Implementation)
- **Rule #2:** TDD - Write tests BEFORE code
- **Rule #3:** Method Size - Max 50 lines, 7-9 objects
- **Rule #8:** Iterative Cycle - Tests → Code → Review → Next
- **Rule #9:** Cognitive Load - Max 7±2 objects in scope
- **Rule #15:** Prompts - Structure prompts carefully

### REFLECT Mode (Review)
- **Rule #8:** Iterative Cycle - Verify test→code→review cycle
- **Rule #10:** Focused Review - Review ONE method at a time

### ARCHIVE Mode (Documentation)
- **Rule #14:** MB Structure - Hierarchical archive
- **Rule #15:** Prompts - Clear, structured content

---

## 🎯 RULE CHEAT SHEET

| # | Rule | One-Liner | When |
|---|------|-----------|------|
| 1 | Stubbing | Max 50 lines per stub | Task breakdown |
| 2 | TDD | Tests before code | Implementation start |
| 3 | Method Size | Max 50 lines, 7-9 objects | All coding |
| 4 | Requirements | Context before coding | Task analysis |
| 5 | DoD | Testable done criteria | Planning |
| 6 | Corner Cases | List boundaries first | Planning/Design |
| 7 | Skeleton-First | Stubs before code | Architecture |
| 8 | Iterative | One method at a time | Implementation |
| 9 | Cognitive Load | Max 7±2 objects | Always |
| 10 | Focused Review | One method review | Code review |
| 11 | Boundaries | State exclusions | Scope definition |
| 12 | Complexity Check | Verify solvability | Before starting |
| 13 | Transaction | Explicit isolation | Data operations |
| 14 | MB Structure | Hierarchical context | Memory Bank |
| 15 | Prompts | Structured prompts | AI interactions |

---

## 📖 DETAILED RULES

For complete guidance on each rule:
- Navigate to `Core/AI-Quality/[category]/[rule-name]-rule.mdc`
- Or check mode maps for inline guidance

### Categories
- `foundation/` - Rules 1-3 (Decomposition, TDD, Size)
- `context/` - Rules 4-6 (Requirements, DoD, Corner Cases)
- `architecture/` - Rules 7-9 (Skeleton, Iterative, Cognitive)
- `quality/` - Rules 10-12 (Review, Boundaries, Complexity)
- `technical/` - Rules 13-15 (Transaction, MB, Prompts)

---

## ✅ QUALITY CHECKPOINTS

### Before Starting (VAN/PLAN)
- [ ] Requirements documented (Rule #4)
- [ ] Complexity verified (Rule #12)
- [ ] Task decomposed into stubs (Rule #1)
- [ ] DoD created (Rule #5)
- [ ] Boundaries defined (Rule #11)
- [ ] Corner cases listed (Rule #6)
- [ ] Architecture skeleton ready (Rule #7)

### During Implementation (BUILD)
- [ ] Tests written first (Rule #2)
- [ ] Methods < 50 lines (Rule #3)
- [ ] Objects ≤ 9 per method (Rule #9)
- [ ] Iterative cycle followed (Rule #8)
- [ ] Prompts structured (Rule #15)

### After Implementation (REFLECT)
- [ ] Each method reviewed individually (Rule #10)
- [ ] Iterative cycle verified (Rule #8)

---

## 🚩 COMMON MISTAKES

### ❌ DON'T
- Write code before tests
- Create methods > 50 lines
- Track > 9 objects per method
- Review entire features at once
- Start without clear requirements
- Skip corner case analysis
- Implement all methods at once

### ✅ DO
- Tests → Code → Review → Next
- Keep methods small and focused
- One method at a time
- Define boundaries explicitly
- Document requirements upfront
- List corner cases first
- Create skeleton before implementing

---

## 📈 EXPECTED IMPROVEMENTS

With consistent rule application:
- **Code Quality:** +30-50%
- **Bug Reduction:** -40-50%
- **Development Speed:** More consistent
- **AI Accuracy:** Significantly higher
- **Maintainability:** Much improved

---

## 🔗 RELATED FILES

- **Always Loaded:** `_principles.mdc` (Core concepts)
- **Navigation:** `_index.mdc` (Rule index)
- **Mode Maps:** See `visual-maps/[mode]-mode-map.mdc`
- **Detailed Rules:** See `[category]/[rule-name]-rule.mdc`

---

*For questions or updates, see Memory Bank documentation.*
