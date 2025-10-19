# Ledger Bridge - Iterative Development Plan

> Step-by-step plan based on [@vision.md](../vision.md) and [@conventions.md](../conventions.md)

---

## 📊 Progress Report

| Iteration | Status | Tasks | Focus | Testable Output |
|-----------|--------|-------|-------|-----------------|
| **Iteration 0** | ✅ Complete | 2/2 | Setup | Workspace builds |
| **Iteration 1** | ✅ Complete | 3/3 | Foundation | Core types compile |
| **Iteration 2** | ✅ Complete | 4/4 | CSV | Parse & write CSV |
| **Iteration 3** | ✅ Complete | 4/4 | MT940 | Parse & write MT940 |
| **Iteration 4** | ✅ Complete | 4/4 | CAMT.053 | Parse & write XML |
| **Iteration 5** | ✅ Complete | 3/3 | Conversions | From trait works |
| **Iteration 6** | ✅ Complete | 3/3 | CLI | End-to-end conversions |
| **Iteration 7** | ⏳ Pending | 0/3 | Polish | Production ready |

**Legend:** ⏳ Pending | 🔄 In Progress | ✅ Complete | ❌ Blocked

**Overall Progress:** 23/25 tasks complete (92%)

## 🔍 Current Status Analysis

**✅ Completed Iterations:**
- **Iteration 0-6**: Full workspace setup, foundation types, CSV parsing, MT940 parsing, CAMT.053 format, format conversions, and CLI application are complete
- **64 tests passing** - All implemented functionality is well-tested (47 unit + 17 integration tests)
- **Real file parsing**: CSV (Sberbank), MT940 (Goldman Sachs, ASN Bank), and CAMT.053 examples work
- **Round-trip tested**: Parse → Write → Parse works for all formats
- **Format conversions**: All `From` trait implementations working with comprehensive integration tests
- **CLI working**: End-to-end conversions via command-line interface with proper error handling

**❌ Missing Components:**
1. **Documentation polish** - Add comprehensive doc comments to all public items
2. **Code quality checks** - Run clippy and fmt checks, fix any issues

**🎯 Next Priority:** Iteration 7 (Polish & Validation) - Production-ready code with full documentation and quality checks.

---

## Iteration 0: Workspace Setup

**Goal:** Create project structure

**Testable:** `cargo build` succeeds

### Tasks
- [x] **0.1** Create Cargo workspace with `ledger-parser` and `ledger-bridge-cli` crates
- [x] **0.2** Add dependencies to library: `serde`, `csv`, `quick-xml`

**Test Command:**
```bash
cargo build
```

---

## Iteration 1: Foundation - Shared Types

**Goal:** Define shared data structures used across all formats

**Testable:** Types compile, unit tests pass

### Tasks
- [x] **1.1** Create `model.rs` with `Transaction`, `BalanceType`, `TransactionType`
- [x] **1.2** Create `error.rs` with `ParseError` enum (Display, Error, From<io::Error>)
- [x] **1.3** Create `lib.rs` with public API exports

**Test Command:**
```bash
cargo test --lib
cargo doc --no-deps
```

**Test Code:**
```rust
#[test]
fn test_transaction_creation() {
    let tx = Transaction {
        booking_date: "2025-01-15".to_string(),
        amount: 100.50,
        transaction_type: TransactionType::Credit,
        // ...
    };
    assert_eq!(tx.amount, 100.50);
}
```

---

## Iteration 2: CSV Format Implementation

**Goal:** Complete CSV parsing and writing

**Testable:** Parse CSV, write CSV, round-trip works

### Tasks
- [x] **2.1** Create `formats/csv.rs` with `CsvStatement` struct (identical fields to future Mt940/Camt053)
- [x] **2.2** Implement `CsvStatement::from_read<R: Read>()`  using `csv` crate
- [x] **2.3** Implement `CsvStatement::write_to<W: Write>()` using `csv` crate
- [x] **2.4** Add unit tests (parse, write, error cases)

**Test Command:**
```bash
cargo test csv
```

**Test Code:**
```rust
#[test]
fn test_csv_parse() {
    let csv_data = "Account,Currency,...\n...";
    let mut reader = csv_data.as_bytes();
    let result = CsvStatement::from_read(&mut reader);
    assert!(result.is_ok());
}

#[test]
fn test_csv_write() {
    let statement = CsvStatement { /* ... */ };
    let mut output = Vec::new();
    statement.write_to(&mut output).unwrap();
    assert!(!output.is_empty());
}
```

---

## Iteration 3: MT940 Format Implementation

**Goal:** Complete MT940 parsing and writing (manual)

**Testable:** Parse MT940, write MT940, round-trip works

### Tasks
- [x] **3.1** Create `formats/mt940.rs` with `Mt940` struct (same fields as CsvStatement)
- [x] **3.2** Implement `Mt940::from_read<R: Read>()` - manual tag-based parsing
  - Block 4 extraction, tags: `:25:`, `:60F:`, `:61:`, `:86:`, `:62F:`
  - YYMMDD date conversion (century inference)
- [x] **3.3** Implement `Mt940::write_to<W: Write>()` - generate MT940 format
- [x] **3.4** Add unit tests (parse, write, multi-line `:86:` handling)

**Test Command:**
```bash
cargo test mt940
```

**Test Code:**
```rust
#[test]
fn test_mt940_parse() {
    let mt940_data = "{4:\n:20:REF\n:25:ACC123\n:60F:C250101USD1000,00\n...";
    let mut reader = mt940_data.as_bytes();
    let result = Mt940::from_read(&mut reader);
    assert!(result.is_ok());
}
```

---

## Iteration 4: CAMT.053 Format Implementation

**Goal:** Complete CAMT.053 parsing and writing

**Testable:** Parse XML, write XML, round-trip works

### Tasks
- [x] **4.1** Create `formats/camt053.rs` with `Camt053` struct (same fields as Mt940/CsvStatement)
- [x] **4.2** Implement `Camt053::from_read<R: Read>()` using `quick-xml` event parsing
  - Extract `<Acct>`, `<Bal>` (OPBD/CLBD), `<Ntry>` elements
  - Handle namespaces, attributes (`Ccy="XXX"`)
- [x] **4.3** Implement `Camt053::write_to<W: Write>()` using `quick-xml` writer
- [x] **4.4** Add unit tests (parse, write, balance type filtering)

**Test Command:**
```bash
cargo test camt053
```

**Test Code:**
```rust
#[test]
fn test_camt053_parse() {
    let xml = r#"<Document xmlns="..."><BkToCstmrStmt>..."#;
    let mut reader = xml.as_bytes();
    let result = Camt053::from_read(&mut reader);
    assert!(result.is_ok());
}
```

---

## Iteration 5: Format Conversions

**Goal:** Implement `From` trait for all format pairs

**Testable:** Type conversions work, data preserved

### Tasks
- [x] **5.1** Implement `From<Mt940> for Camt053` and `From<Camt053> for Mt940`
- [x] **5.2** Implement `From<CsvStatement> for Mt940` and `From<Mt940> for CsvStatement`
- [x] **5.3** Implement `From<CsvStatement> for Camt053` and `From<Camt053> for CsvStatement`

**Test Command:**
```bash
cargo test --test integration_test
```

**Test Code:**
```rust
#[test]
fn test_mt940_to_camt053_conversion() {
    let mt940 = Mt940 { account_number: "123".to_string(), /* ... */ };
    let camt053: Camt053 = mt940.into();
    assert_eq!(camt053.account_number, "123");
}

#[test]
fn test_round_trip_via_conversion() {
    let original = Mt940 { /* ... */ };
    let camt: Camt053 = original.clone().into();
    let back: Mt940 = camt.into();
    assert_eq!(original.account_number, back.account_number);
}
```

---

## Iteration 6: CLI Application

**Goal:** Build working command-line interface

**Testable:** End-to-end format conversions via CLI

### Tasks
- [x] **6.1** Create `main.rs` with clap `Cli` struct (`--in-format`, `--out-format`, `--input`, `--output`)
- [x] **6.2** Implement `main()`: Read from file/stdin, parse, convert using `From` trait, write to file/stdout
- [x] **6.3** Add error handling (print to stderr, exit code 1)

**Test Command:**
```bash
cargo run -- --in-format csv --out-format mt940 --input example_files/example_of_account_statement.csv
cargo run -- --in-format mt940 --out-format camt053 --input example_files/mt\ 940\ gs.mt940
cargo run -- --help
```

**Expected:** Successful conversions, proper error messages, help output

**✅ Verified Working:**
- CSV → MT940 conversion with real Sberbank file
- MT940 → CAMT.053 conversion with real Goldman Sachs file  
- Help command displays proper usage
- Error handling works (invalid formats, missing files)

---

## Iteration 7: Polish & Validation

**Goal:** Production-ready code

**Testable:** All quality checks pass

### Tasks
- [ ] **7.1** Add doc comments to all public items, test with real example files
- [ ] **7.2** Run `cargo clippy -- -D warnings`, `cargo fmt --check`, fix all issues
- [ ] **7.3** Add README.md for all projects. With a description of the project and examples of use

**Test Command:**
```bash
cargo test --all
cargo clippy -- -D warnings
cargo fmt --check
cargo doc --no-deps --open
```

---

## Quick Reference

### After Each Iteration
1. ✅ Run test command
2. ✅ Verify testable output
3. ✅ Update progress table
4. ✅ Commit working code

### Key Principles
- **Incremental:** Each iteration adds new, testable functionality
- **KISS:** Simple implementations first
- **Tests:** Every iteration has clear test criteria
- **Dependencies:** Use `csv` and `quick-xml` as specified in conventions.md

### Conversion Flow
```
CSV ←→ Mt940 ←→ Camt053
 ↑               ↓
 └───────────────┘
```

All conversions use `From` trait:
```rust
let mt940 = Mt940::from_read(&mut reader)?;
let camt053: Camt053 = mt940.into();
camt053.write_to(&mut writer)?;
```

---

## Completion Criteria

✅ **Project complete when:**
- All 25 tasks checked
- `cargo test --all` passes
- `cargo clippy` clean
- All format pairs convert successfully via CLI
- Documentation generated

**Total Iterations:** 8 (Setup → Foundation → 3 Formats → Conversions → CLI → Polish)
