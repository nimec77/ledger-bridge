# Tasklist.md Updates Summary

## Overview
The `doc/tasklist.md` has been completely rewritten to reflect the new architecture and create a truly iterative, testable development plan.

---

## Major Changes

### 1. **Structure: Phases → Iterations**

**Before:**
- 6 phases (0-5)
- 19 subtasks
- Phase-based organization

**After:**
- 8 iterations (0-7)
- 25 tasks total
- Iteration-based organization (smaller, more testable chunks)

**Why:** True iterative development with clear milestones

---

### 2. **Progress Report Table**

**Before:**
```
| Phase | Status | Progress | Description |
```

**After:**
```
| Iteration | Status | Tasks | Focus | Testable Output |
```

**Added Columns:**
- **Focus:** What this iteration accomplishes
- **Testable Output:** Clear success criteria

**New Footer:**
- **Overall Progress:** 0/25 tasks complete (0%)

**Better Visual Tracking** with iteration numbers and specific task counts

---

### 3. **Architecture Alignment**

#### Removed (Old Architecture):
- ❌ Generic `Parser<T>` and `Formatter<T>` traits
- ❌ Zero-sized parser structs (CsvParser, Mt940Parser)
- ❌ BufRead trait references
- ❌ `parse()` and `format()` method names
- ❌ Formatters returning strings
- ❌ `traits.rs` file
- ❌ "Statement" as unified type

#### Added (New Architecture):
- ✅ Format-specific structs: `CsvStatement`, `Mt940`, `Camt053`
- ✅ `from_read<R: Read>()` method pattern
- ✅ `write_to<W: Write>()` method pattern
- ✅ `From` trait conversions (dedicated iteration)
- ✅ Read/Write trait usage
- ✅ Required libraries: `csv`, `quick-xml`

---

### 4. **Iteration Breakdown**

#### Iteration 0: Workspace Setup (2 tasks)
**Before:** 3 subtasks across workspace and dependencies
**After:** Streamlined to just workspace + dependencies
**Testable:** `cargo build` succeeds

#### Iteration 1: Foundation (3 tasks)
**Before:** Combined with setup
**After:** Separate iteration for shared types
**Focus:** Transaction, BalanceType, TransactionType, ParseError
**Testable:** Types compile, unit tests pass

**Key Change:** No traits.rs file, only model.rs and error.rs

#### Iteration 2: CSV Format (4 tasks)
**Before:** 4 subtasks with generic Parser/Formatter
**After:** 4 tasks with format-specific struct

**Old Pattern:**
```rust
impl Parser<Statement> for CsvParser
impl Formatter<Statement> for CsvParser
```

**New Pattern:**
```rust
struct CsvStatement { /* fields */ }
impl CsvStatement {
    fn from_read<R: Read>() -> Result<Self, ParseError>
    fn write_to<W: Write>() -> Result<(), ParseError>
}
```

**Testable:** Parse CSV, write CSV, round-trip works

#### Iteration 3: MT940 Format (4 tasks)
**Before:** Similar structure to CSV
**After:** Emphasizes **manual parsing** (educational value)

**Key Addition:**
- Explicit mention of tag-based parsing
- Block 4 extraction details
- YYMMDD date conversion notes
- Multi-line `:86:` handling

**Testable:** Parse MT940, write MT940, round-trip works

#### Iteration 4: CAMT.053 Format (4 tasks)
**Before:** Generic XML parsing mentioned
**After:** Specific `quick-xml` usage

**New Details:**
- Event-based parsing with `quick-xml`
- Namespace handling
- Attribute extraction (`Ccy="XXX"`)
- Balance type filtering (OPBD/CLBD)

**Testable:** Parse XML, write XML, round-trip works

#### Iteration 5: Format Conversions (3 tasks) ⭐ NEW
**This is completely new!**

**Purpose:** Implement all `From` trait conversions between formats

**Tasks:**
1. Mt940 ↔ Camt053
2. CsvStatement ↔ Mt940
3. CsvStatement ↔ Camt053

**Why Separate Iteration:** 
- From trait is a key learning objective
- Deserves focused testing
- All 6 conversion pairs in one place

**Testable:** Type conversions work, data preserved

**Test Example:**
```rust
#[test]
fn test_mt940_to_camt053_conversion() {
    let mt940 = Mt940 { /* ... */ };
    let camt053: Camt053 = mt940.into();
    assert_eq!(camt053.account_number, mt940.account_number);
}
```

#### Iteration 6: CLI Application (3 tasks)
**Before:** Called "Phase 4" with similar structure
**After:** Updated to use new architecture

**Key Changes:**
- Uses `Box<dyn Read>` instead of `Box<dyn BufRead>`
- Shows conversion using `From` trait in main logic
- Emphasizes Read/Write abstraction

**Example Flow:**
```rust
let mt940 = Mt940::from_read(&mut input)?;
let camt053: Camt053 = mt940.into();
camt053.write_to(&mut output)?;
```

**Testable:** End-to-end conversions via CLI

#### Iteration 7: Polish & Validation (2 tasks)
**Before:** Called "Phase 5" with 3 subtasks
**After:** Streamlined to 2 essential tasks

**Removed:** Separate coverage tracking (not essential)
**Focus:** Documentation + Quality checks

**Testable:** All quality checks pass

---

### 5. **Test Examples in Every Iteration**

**Before:** Tests mentioned but not always shown
**After:** Every iteration includes:
1. **Test Command** - Exact command to run
2. **Test Code** - Example test to write
3. **Expected Output** - What success looks like

**Example from Iteration 2:**
```rust
#[test]
fn test_csv_parse() {
    let csv_data = "Account,Currency,...\n...";
    let mut reader = csv_data.as_bytes();
    let result = CsvStatement::from_read(&mut reader);
    assert!(result.is_ok());
}
```

**Benefits:**
- Clear test-driven development
- Immediate validation
- Copy-paste ready test templates

---

### 6. **Quick Reference Section**

**New Addition:** Quick reference for workflow

**Includes:**
- After each iteration checklist
- Key principles (Incremental, KISS, Tests)
- Conversion flow diagram
- Code example for conversion pattern

**Conversion Flow Diagram:**
```
CSV ←→ Mt940 ←→ Camt053
 ↑               ↓
 └───────────────┘
```

**Shows:** All formats can convert to each other via From trait

---

### 7. **Conciseness Improvements**

**Line Count:**
- Before: 279 lines
- After: 240 lines
- **Reduction: 39 lines (14%)**

**Despite adding:**
- More detailed test examples
- New iteration for conversions
- Quick reference section

**How:**
- Removed redundant explanations
- Combined related tasks
- Streamlined descriptions
- KISS principle applied

---

## Task Count Changes

### Before (Old Architecture):
- Phase 0: 3 tasks (Setup)
- Phase 1: 4 tasks (CSV)
- Phase 2: 4 tasks (MT940)
- Phase 3: 4 tasks (CAMT.053)
- Phase 4: 3 tasks (CLI)
- Phase 5: 3 tasks (Testing)
- **Total: 21 tasks**

### After (New Architecture):
- Iteration 0: 2 tasks (Setup)
- Iteration 1: 3 tasks (Foundation)
- Iteration 2: 4 tasks (CSV)
- Iteration 3: 4 tasks (MT940)
- Iteration 4: 4 tasks (CAMT.053)
- Iteration 5: 3 tasks (Conversions) ⭐ NEW
- Iteration 6: 3 tasks (CLI)
- Iteration 7: 2 tasks (Polish)
- **Total: 25 tasks**

**Net Change:** +4 tasks
- Foundation separated from setup
- Conversions added as new iteration
- Tasks more granular and testable

---

## Testability Improvements

### Every Iteration Now Has:

1. **Clear Goal Statement**
   ```
   Goal: Complete CSV parsing and writing
   ```

2. **Testable Output**
   ```
   Testable: Parse CSV, write CSV, round-trip works
   ```

3. **Test Command**
   ```bash
   cargo test csv
   ```

4. **Test Code Example**
   ```rust
   #[test]
   fn test_csv_parse() { /* ... */ }
   ```

5. **Success Criteria**
   - Explicit expectations for each test

**Result:** Every iteration is completely self-contained and verifiable

---

## Alignment with Vision.md & Conventions.md

### References Architecture Correctly:

✅ **Format Structs:**
- CsvStatement, Mt940, Camt053 (not parsers)

✅ **Methods:**
- `from_read<R: Read>()`
- `write_to<W: Write>()`
- Not `parse()` or `format()`

✅ **Traits:**
- Read/Write for I/O
- From for conversions
- No generic Parser/Formatter traits

✅ **Libraries:**
- `csv` crate for CSV
- `quick-xml` for CAMT.053
- Manual parsing for MT940

✅ **Shared Types:**
- Transaction, BalanceType, TransactionType
- Not a unified Statement

✅ **Error Handling:**
- ParseError enum
- No unwrap/expect in library
- From<io::Error> implementation

---

## Progress Tracking Improvements

### Progress Table Features:

**Status Icons:**
- ⏳ Pending
- 🔄 In Progress
- ✅ Complete
- ❌ Blocked

**Task Tracking:**
- "0/2 tasks" shows exact progress per iteration
- Overall progress percentage at bottom
- Easy to update after each iteration

**Visual Clarity:**
- Testable Output column shows what success means
- Focus column shows purpose
- Clean table formatting

**Example Update After Iteration 0:**
```markdown
| Iteration 0 | ✅ Complete | 2/2 | Setup | Workspace builds |
```

---

## Developer Experience

### Before:
- Read phase description
- Figure out what to implement
- Search for test guidance
- Unclear when phase is "done"

### After:
- See iteration goal
- Read task list
- Copy test template
- Run test command
- See immediate success/failure
- Update checkbox
- Move to next iteration

**Result:** Clear path forward at every step

---

## KISS Principle Applied

### Simplified:
- ✅ Removed redundant explanations
- ✅ Combined related tasks
- ✅ Direct test examples (copy-paste ready)
- ✅ Clear success criteria

### Kept Essential:
- ✅ Architecture-specific details
- ✅ Library usage guidance
- ✅ Test commands
- ✅ Code examples

### References for Details:
- ✅ Points to vision.md for complete specs
- ✅ Points to conventions.md for rules
- ✅ Doesn't duplicate documentation

---

## Key Learning Points Emphasized

Each iteration focuses on one learning objective:

1. **Iteration 1:** Data structures with serde
2. **Iteration 2:** CSV crate + Read/Write traits
3. **Iteration 3:** Manual parsing + text processing
4. **Iteration 4:** XML parsing with quick-xml
5. **Iteration 5:** From trait for type conversions ⭐
6. **Iteration 6:** CLI with Read/Write abstraction
7. **Iteration 7:** Code quality tools

**Progressive Complexity:** Starts simple (setup), builds to complex (XML), then integrates (CLI)

---

## Ready for Implementation

The updated tasklist provides:

✅ **Clear roadmap** - 8 iterations with specific tasks
✅ **Testable milestones** - Every iteration has test criteria
✅ **Copy-paste templates** - Test code examples included
✅ **Progress tracking** - Visual table with checkboxes
✅ **Architecture alignment** - Matches vision.md and conventions.md
✅ **KISS compliance** - Concise, essential information only
✅ **Self-contained** - Each iteration independent

**Total Development Time:** Estimated 8 work sessions (one per iteration)

---

**Document Updated:** October 18, 2025  
**Status:** Complete, ready for iterative development  
**Total Tasks:** 25 across 8 iterations  
**Line Count:** 240 (14% reduction with more content)

