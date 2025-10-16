# Development Workflow - Ledger Bridge

> Instructions for code assistants working on this project

---

## Workflow Rules

### 1. Before Starting Work
- Read current task from `doc/tasklist.md`
- Understand requirements from `vision.md` and `conventions.md`
- Identify the current phase and subtask

### 2. Propose Solution
**BEFORE any implementation:**
- Present proposed approach with code snippets
- Show key function signatures and data structures
- Explain design decisions
- **Wait for user agreement**

Example:
```
I propose implementing CsvParser with:

struct CsvParser;

impl Parser<Statement> for CsvParser {
    type Error = ParseError;
    fn parse<R: BufRead>(&self, reader: R) -> Result<Statement, Self::Error> {
        // Parse line by line
        // Skip headers
        // Extract transactions
    }
}

Helper functions:
- parse_amount(s: &str) -> Result<f64>
- parse_date(s: &str) -> Result<String>

Agree to proceed?
```

### 3. Implement After Agreement
- Implement **only** what was agreed upon
- Follow `conventions.md` strictly
- No `.unwrap()` or panics in library code
- Add tests for the implemented functionality

### 4. Verify Implementation
- Run `cargo build` (must succeed)
- Run `cargo test` (all tests pass)
- Run `cargo clippy` (no warnings)
- Run `cargo fmt` (format code)

### 5. Update Progress
Update `doc/tasklist.md`:
- Mark completed subtasks: `- [x]`
- Update phase progress counters
- Update status: ⏳ → 🔄 → ✅

### 6. Wait for Confirmation
- Present what was completed
- Show test results
- **Wait for user to confirm** before proceeding

### 7. Commit Changes
After user confirmation:
```bash
git add .
git commit -m "Phase X.Y: [Description]

- Implemented [feature]
- Added tests for [functionality]
- Updated tasklist progress"
```

### 8. Move to Next Task
- Get explicit agreement to proceed to next subtask
- Return to step 1

---

## Task Execution Pattern

```
┌─────────────────────────────────────┐
│ 1. Read tasklist & requirements     │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│ 2. Propose solution + code snippets │
│    ⏸️  WAIT FOR AGREEMENT           │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│ 3. Implement agreed solution        │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│ 4. Test (build/test/clippy/fmt)     │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│ 5. Update tasklist progress         │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│ 6. Present results                  │
│    ⏸️  WAIT FOR CONFIRMATION        │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│ 7. Commit to repository             │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│ 8. Get agreement for next task      │
│    ⏸️  WAIT FOR GO-AHEAD            │
└─────────────────────────────────────┘
```

---

## Critical Requirements

### Must Do
✅ Propose before implementing  
✅ Wait for agreement at checkpoints  
✅ Update tasklist after each subtask  
✅ Test before marking complete  
✅ Follow conventions.md strictly  
✅ Commit only after confirmation  

### Must Not Do
❌ Implement without proposal  
❌ Skip ahead without agreement  
❌ Commit untested code  
❌ Use `.unwrap()` in library  
❌ Make breaking changes without discussion  

---

## Communication Format

### When Proposing
```
📋 Task: [Phase X.Y - Task name]

🎯 Approach:
[Brief explanation]

💻 Code Structure:
[Key signatures and snippets]

🧪 Tests:
[What will be tested]

❓ Ready to proceed?
```

### When Complete
```
✅ Completed: [Phase X.Y - Task name]

📝 Changes:
- [List of changes]

🧪 Tests:
✓ cargo build: PASS
✓ cargo test: PASS
✓ cargo clippy: PASS

📊 Updated tasklist progress

⏸️  Awaiting confirmation to commit
```

---

## Example Session

**Assistant:**
```
📋 Task: Phase 0.1 - Workspace Setup

🎯 Approach:
Create Cargo workspace with two crates using workspace.members

💻 Structure:
Cargo.toml (workspace root):
[workspace]
members = ["ledger-parser", "ledger-bridge-cli"]

ledger-parser/Cargo.toml:
[package]
name = "ledger-parser"
[dependencies]
serde = { version = "1.0", features = ["derive"] }

❓ Agree to create this structure?
```

**User:** "Yes, proceed"

**Assistant:**
```
[Creates files, runs tests]

✅ Completed: Phase 0.1 - Workspace Setup

📝 Changes:
- Created workspace Cargo.toml
- Created ledger-parser crate
- Created ledger-bridge-cli crate
- Added dependencies

🧪 Tests:
✓ cargo build: PASS

📊 Updated tasklist (Phase 0: 1/3 complete)

⏸️  Awaiting confirmation to commit
```

**User:** "Good, commit it"

**Assistant:**
```
[Commits changes]

✅ Committed: "Phase 0.1: Workspace Setup"

Ready for Phase 0.2?
```

---

## Quick Reference

| Checkpoint | Action | Wait? |
|------------|--------|-------|
| Before coding | Propose solution | ✋ YES |
| After implementation | Show results | ✋ YES |
| Before commit | Get confirmation | ✋ YES |
| Before next task | Get go-ahead | ✋ YES |

**Remember:** KISS principle - Keep workflows simple and effective.

