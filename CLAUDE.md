# Comparable - Deterministic Change Detection Library

This document provides essential context for Claude instances working on the `comparable` codebase.

## Development Commands

### Building
```bash
# Build all workspace crates
make build
# Or directly:
cargo build

# Clean generated files
make clean
```

### Testing
```bash
# Run all tests (including doctests)
make test

# Run tests for a specific crate
cargo test -p comparable
cargo test -p comparable_derive
cargo test -p comparable_test

# Run a single test
cargo test test_struct_2_unnamed_fields_scalar

# Run doctests only
cargo test --doc
```

### Linting and Validation
```bash
# Full validation (tests + docs + clippy with zero warnings)
make clippy

# Format check
cargo fmt --check

# Clippy with all lints as errors (uses 40 parallel jobs)
cargo clippy -j40 --tests -- -D clippy::all
```

### Documentation
```bash
# Generate and test documentation
make docs

# Open documentation in browser
make docs-open
```

## Critical Architecture Patterns

### 1. Auto-Generated lib.rs
**WARNING**: The file `comparable/src/lib.rs` is **auto-generated** from `README.md`. Never edit it directly.

To modify library documentation:
1. Edit `README.md`
2. Run `make build` or `make comparable/src/lib.rs`
3. The Makefile concatenates README.md (prefixed with `//!`) with `comparable/src/lib.rs.in`

### 2. Workspace Structure and Dependencies

The project is a 4-crate workspace with specific dependency relationships:

```
comparable/              [Main library - provides trait and impls]
    ├── depends on → comparable_derive (optional, via "derive" feature)
    └── depends on → comparable_helper (for tuple macros)

comparable_derive/       [Procedural macro for #[derive(Comparable)]]
    └── standalone, processes AST to generate code

comparable_helper/       [Helper procedural macros]
    └── generates tuple implementations (tuple_impl!)

comparable_test/         [Test suite]
    └── depends on → comparable (with derive feature)
```

### 3. Type-Level Mirror Pattern

The core innovation requires understanding how the derive macro creates **parallel type hierarchies**:

```rust
// Original type
struct Foo {
    x: u32,
    y: String,
}

// Generated Description type (mirrors structure)
struct FooDesc {
    x: <u32 as Comparable>::Desc,      // = u32
    y: <String as Comparable>::Desc,   // = String
}

// Generated Change type (represents differences)
enum FooChange {
    X(<u32 as Comparable>::Change),    // = U32Change
    Y(<String as Comparable>::Change), // = StringChange
}
```

This pattern is implemented across multiple files:
- Trait definition: `comparable/src/types.rs`
- Derive macro logic: `comparable_derive/src/structs.rs` and `enums.rs`
- Code generation: `comparable_derive/src/definition.rs`

### 4. Procedural Macro Architecture

The derive macro (`comparable_derive/`) follows a specific flow:

1. **Parse** (`inputs.rs`): Extract struct/enum definition
2. **Analyze attributes** (`attrs.rs`): Process `#[comparable_*]` attributes
3. **Generate types** (`definition.rs`): Create Desc and Change types
4. **Generate impl** (`structs.rs`/`enums.rs`): Create Comparable trait impl
5. **Output** (`outputs.rs`): Combine into final TokenStream

Key attributes that affect generation:
- `#[comparable_ignore]` - Skip field in comparisons
- `#[comparable_synthetic {...}]` - Add computed fields
- `#[self_describing]` - Use Self as Desc type
- `#[compare_default]` - Describe as diff from Default
- `#[variant_struct_fields]` - Treat enum variants as structs

### 5. Change Representation Strategy

Different types use different change representations:

- **Scalars**: Store old and new values (e.g., `I32Change(100, 200)`)
- **Structs (multi-field)**: `Vec<EnumOfFieldChanges>`
- **Structs (single field)**: Direct change type (no Vec)
- **Enums**: `Both<Variant>` or `Different(desc1, desc2)`
- **Collections**: Position-based changes (Vec) or Add/Remove (Set/Map)

### 6. Determinism Trade-offs

The library prioritizes deterministic output over performance:

- `HashMap` comparison sorts keys first (O(n log n) overhead)
- `HashSet` comparison requires `Ord` for sorting
- This is intentional for reliable test assertions

## Working with the Codebase

### Adding New Scalar Types
1. Implement in `comparable/src/scalar.rs`
2. Follow the pattern: self-describing, Change type holds (old, new)
3. Add tests in `comparable_test/test/scalar.rs`

### Modifying the Derive Macro
1. Changes to parsing: `comparable_derive/src/inputs.rs`
2. Changes to attributes: `comparable_derive/src/attrs.rs`
3. Changes to generation: `comparable_derive/src/structs.rs` or `enums.rs`
4. Test via `comparable_test/test/structs.rs` or `enums.rs`

### Understanding Generated Code
To see what the derive macro generates, use `cargo expand`:
```bash
cd comparable_test
cargo expand --test structs
```

### Key Invariants
1. All `Desc` and `Change` types must implement `PartialEq + Debug`
2. The `Changed` enum is semantically equivalent to `Option` but clearer
3. Collections must produce deterministic output (may require sorting)
4. No unsafe code in the entire codebase

## Notable Implementation Details

- **Edition**: Uses Rust 2018 (could be upgraded to 2021)
- **Parallelism**: Makefile assumes 40+ cores (`-j40`)
- **Float comparison**: Uses bitwise equality without epsilon tolerance
- **Error handling**: Procedural macros panic instead of emitting compile errors (known issue)
- **No async**: Purely synchronous, stateless, thread-safe by default