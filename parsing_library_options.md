# Parsing Library Options for Ledger Bridge

> Keeping KISS principles in mind - these are training tasks

## Core Requirement
**All parsers must read data using `std::io::BufRead` or `std::io::Read` traits**

## Recommended Libraries (Simple & KISS-focused)

### 1. CSV Parsing

#### Option A: Manual (KISS - Recommended for Learning)
- **Library**: None (use standard library only)
- **Approach**: `BufRead::lines()` + `str::split(',')`
- **Pros**: Maximum learning, no dependencies, simple
- **Cons**: Need to handle quoted fields manually

#### Option B: `csv` crate
- **Crate**: [`csv`](https://crates.io/crates/csv) v1.3+
- **Why**: Industry standard, works with `BufRead`, minimal complexity
- **Usage**: 
  ```rust
  use csv::Reader;
  let mut reader = Reader::from_reader(buf_reader);
  for result in reader.records() { ... }
  ```
- **Pros**: Handles quoted fields, RFC 4180 compliant, works with BufRead
- **Cons**: Adds a dependency (but it's tiny and well-maintained)

### 2. MT940 Parsing

#### Option A: Manual String Processing (KISS - Recommended)
- **Library**: None (use standard library only)
- **Approach**: Read all with `BufRead::read_to_string()`, then parse tags manually
- **Pros**: Simple format, good learning exercise, no dependencies
- **Cons**: More code to write

#### Option B: `nom` (Parser Combinators)
- **Crate**: [`nom`](https://crates.io/crates/nom) v7+
- **Why**: If you want to learn parser combinators
- **Usage**: Build parsers for each MT940 tag
- **Pros**: Powerful, composable, good error messages
- **Cons**: Steeper learning curve, might be overkill for this task
- **Verdict**: **Skip this** - too complex for a training task

#### Option C: `winnow` (Modern Parser Combinators)
- **Crate**: [`winnow`](https://crates.io/crates/winnow) v0.6+
- **Why**: Successor to nom, simpler API
- **Pros**: More beginner-friendly than nom, works with streams
- **Cons**: Still complex for a simple task
- **Verdict**: **Skip this** - stick with manual parsing

### 3. CAMT.053 (XML) Parsing

#### Option A: Manual String Parsing (KISS - Recommended)
- **Library**: None (use standard library only)
- **Approach**: `str::find()` and string slicing for simple tag extraction
- **Pros**: Good for learning, no dependencies, sufficient for this task
- **Cons**: Won't handle all edge cases (but we don't need to!)

#### Option B: `quick-xml` (Minimal XML Parser)
- **Crate**: [`quick-xml`](https://crates.io/crates/quick-xml) v0.36+
- **Why**: Lightweight, event-based, works with `BufRead`
- **Usage**:
  ```rust
  use quick_xml::Reader;
  let mut reader = Reader::from_reader(buf_reader);
  loop {
      match reader.read_event() {
          Ok(Event::Start(e)) => { ... }
          ...
      }
  }
  ```
- **Pros**: Small, fast, handles namespaces, works with BufRead
- **Cons**: Adds dependency, event-based API requires state management
- **Verdict**: **Optional** - use if you want proper XML handling

#### Option C: Full XML Parsers (serde-xml-rs, roxmltree)
- **Verdict**: **Skip these** - too heavy for a training task

## Recommended Approach (KISS)

### For Maximum Learning (Recommended):
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
# That's it! Parse everything manually
```

**Parse with BufRead:**
- CSV: Read line-by-line with `lines()`, split on commas
- MT940: Read all with `read_to_string()`, parse tags manually
- CAMT.053: Read all with `read_to_string()`, use `str::find()` for tags

### For Slightly Less Manual Work:
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
csv = "1.3"  # Only if you want to avoid manual CSV edge cases
```

### If You Want to Learn Event-Based XML:
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
quick-xml = "0.36"  # For CAMT.053 parsing
```

## Why BufRead?

All these approaches work with `BufRead`:

1. **Manual parsing**: `BufRead::lines()` or `BufRead::read_to_string()`
2. **`csv` crate**: `Reader::from_reader(buf_read)`  
3. **`quick-xml`**: `Reader::from_reader(buf_read)`

This allows your parsers to work with:
- Files: `BufReader::new(File::open(path)?)`
- Stdin: `BufReader::new(io::stdin())`
- In-memory: `BufReader::new(bytes.as_slice())`
- Network: `BufReader::new(tcp_stream)`

## Final Recommendation

**Start with manual parsing for everything.** This is a training task, and the formats are simple enough that manual parsing will teach you more. You can always add libraries later if needed.

```toml
[package]
name = "ledger-parser"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
# Add others only if you really need them
```

**Remember**: The goal is to learn traits and BufRead, not to build production-grade parsers. Keep it simple!

