# Development Workflow - Ledger Bridge

> Instructions for code assistants working on [@tasklist.md](tasklist.md) using [@vision.md](../vision.md)

---

## Workflow Rules

### 1. Before Starting Work
- Read current iteration from [@tasklist.md](tasklist.md)
- Review requirements from [@vision.md](../vision.md) and [@conventions.md](../conventions.md)
- Identify which tasks in the iteration are pending
- Check the testable output criteria for the iteration

### 2. Propose Solution
**BEFORE any implementation:**
- Present proposed approach with code snippets
- Show key struct definitions and method signatures
- Explain design decisions
- Reference relevant sections from vision.md/conventions.md
- **Wait for user approval**

Example:
```
📋 Iteration 2, Task 2.1: Create CsvStatement struct

🎯 Approach:
Define CsvStatement with fields identical to Mt940/Camt053 for simple From trait conversions.

💻 Code Structure:
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CsvStatement {
    pub account_number: String,
    pub currency: String,
    pub opening_balance: f64,
    pub opening_date: String,
    pub opening_indicator: BalanceType,
    pub closing_balance: f64,
    pub closing_date: String,
    pub closing_indicator: BalanceType,
    pub transactions: Vec<Transaction>,
}

impl CsvStatement {
    pub fn from_read<R: std::io::Read>(reader: &mut R) -> Result<Self, ParseError> {
        // Use csv::Reader::from_reader()
        // Parse with Serde deserialize
    }
    
    pub fn write_to<W: std::io::Write>(&self, writer: &mut W) -> Result<(), ParseError> {
        // Use csv::Writer::from_writer()
        // Write with Serde serialize
    }
}

❓ Agree to proceed with this structure?
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
Update [@tasklist.md](tasklist.md):
- Mark completed tasks: `- [x]` → `- [X]` (capital X)
- Update iteration task counters (e.g., "0/4" → "1/4")
- Update iteration status when starting: ⏳ → 🔄
- Update iteration status when complete: 🔄 → ✅
- Update overall progress percentage

### 6. Wait for Confirmation
- Present what was completed
- Show test results
- **Wait for user to confirm** before proceeding

### 7. Commit Changes (Optional)
After user confirmation, if requested:
```bash
git add .
git commit -m "Iteration X, Task X.Y: [Description]

- Implemented [feature]
- Added tests for [functionality]
- Tests passing: build, test, clippy
- Updated tasklist progress"
```

### 8. Move to Next Task
- Complete all tasks in current iteration before moving to next
- Get explicit agreement to proceed to next task/iteration
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
📋 Iteration X, Task X.Y: [Task name]

🎯 Approach:
[Brief explanation of what will be implemented]

💻 Code Structure:
[Key struct definitions, method signatures, snippets]

🧪 Tests:
[What will be tested and how]

❓ Agree to proceed?
```

### When Complete
```
✅ Completed: Iteration X, Task X.Y - [Task name]

📝 Changes:
- [List of files created/modified]
- [List of functionality added]

🧪 Test Results:
✓ cargo build: PASS
✓ cargo test: PASS (X tests)
✓ cargo clippy: PASS (no warnings)
✓ cargo fmt: Code formatted

📊 Progress: Updated tasklist (Iteration X: Y/Z tasks complete)

⏸️  Awaiting confirmation to proceed
```

---

## Example Session

**Assistant:**
```
📋 Iteration 0, Task 0.1: Create Cargo workspace

🎯 Approach:
Create Cargo workspace with two crates: ledger-parser (library) and ledger-bridge-cli (binary).
Add required dependencies: serde, csv, quick-xml for library; clap for CLI.

💻 Structure:
Cargo.toml (workspace root):
[workspace]
members = ["ledger-parser", "ledger-bridge-cli"]
resolver = "2"

ledger-parser/Cargo.toml:
[package]
name = "ledger-parser"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
csv = "1.3"
quick-xml = "0.31"

ledger-bridge-cli/Cargo.toml:
[package]
name = "ledger-bridge-cli"
edition = "2021"

[dependencies]
ledger-parser = { path = "../ledger-parser" }
clap = { version = "4.0", features = ["derive"] }

🧪 Tests:
Run `cargo build` to verify workspace structure

❓ Agree to proceed?
```

**User:** "Yes, proceed"

**Assistant:**
```
[Creates files, runs tests]

✅ Completed: Iteration 0, Task 0.1 - Workspace Setup

📝 Changes:
- Created Cargo.toml (workspace root)
- Created ledger-parser/ crate with Cargo.toml and src/lib.rs
- Created ledger-bridge-cli/ crate with Cargo.toml and src/main.rs
- Added all required dependencies

🧪 Test Results:
✓ cargo build: PASS (compiles both crates)

📊 Progress: Updated tasklist (Iteration 0: 1/2 tasks complete)

⏸️  Awaiting confirmation to proceed to Task 0.2
```

**User:** "Good, continue to next task"

**Assistant:**
```
📋 Iteration 0, Task 0.2: [Next task proposal...]
```

---

## Iteration Completion

### Before Moving to Next Iteration
Each iteration has specific **Testable Output** criteria in [@tasklist.md](tasklist.md).

**Verify all criteria met:**
- [ ] All tasks in iteration completed and checked
- [ ] Test command from tasklist runs successfully
- [ ] Expected functionality demonstrated
- [ ] Progress table updated (status, task count, percentage)
- [ ] User confirms iteration complete

**Example from Iteration 2 (CSV):**
```
Testable: Parse CSV, write CSV, round-trip works

Must demonstrate:
1. Parse a CSV string into CsvStatement
2. Write CsvStatement back to CSV format
3. Round-trip: CSV → CsvStatement → CSV (data preserved)
```

Only proceed to next iteration after:
✅ All tests passing
✅ Functionality demonstrated
✅ User approval received

---

## Quick Reference

| Checkpoint | Action | Wait? |
|------------|--------|-------|
| Before coding | Propose solution with code snippets | ✋ YES |
| After implementation | Show test results | ✋ YES |
| Before next task | Get approval | ✋ YES |
| Before next iteration | Verify all criteria + get approval | ✋ YES |

### Key Principles
- **Propose first** - Always show code before implementing
- **Test everything** - build, test, clippy must pass
- **Update progress** - Keep tasklist.md current
- **Wait at checkpoints** - Don't skip ahead without approval
- **One iteration at a time** - Complete fully before moving on

**Remember:** KISS principle - Keep workflows simple and effective.

