# Publishing Comparable to crates.io

This document provides step-by-step instructions for publishing the `comparable` workspace to crates.io.

## Workspace Structure and Dependencies

The workspace contains 4 crates with the following dependency relationships:

```
comparable_helper/       [proc-macro, no internal dependencies]
comparable_derive/       [proc-macro, no internal dependencies]
comparable/              [main library, depends on helper + derive]
comparable_test/         [test suite, internal use only - DO NOT PUBLISH]
```

## Publishing Order

Crates must be published in dependency order:

1. **comparable_helper** (no dependencies)
2. **comparable_derive** (no dependencies)
3. **comparable** (depends on both helper and derive)
4. **comparable_test** - **SKIP** (internal testing only)

## Prerequisites

- [ ] Ensure you have a crates.io account
- [ ] Log in with your API token: `cargo login`
- [ ] Verify Cargo version: `cargo --version` (1.90+ recommended for workspace publishing)

## Option 1: Modern Workspace Publishing (Cargo 1.90+)

If you have Cargo 1.90 or later (released September 2024), you can publish the entire workspace in one command using automatic dependency ordering.

### Steps

```bash
# 1. Regenerate lib.rs from README.md
make build

# 2. Ensure all tests pass
make clippy

# 3. Dry-run to preview (recommended)
cargo publish --workspace --exclude comparable_test --dry-run

# 4. Publish all crates
cargo publish --workspace --exclude comparable_test

# 5. Tag the release in git
git tag v0.5.6
git push --tags
```

**How it works:** Cargo 1.90+ uses a local registry overlay to handle inter-workspace dependencies without waiting for crates.io indexing between publishes.

## Option 2: Traditional Sequential Publishing

Use this approach if you have Cargo < 1.90 or prefer manual control.

### Pre-Publication Checklist

```bash
# 1. Regenerate lib.rs from README.md
make build

# 2. Run full test suite
make clippy

# 3. Preview what will be published
cd comparable_helper && cargo package --list
cd ../comparable_derive && cargo package --list
cd ../comparable && cargo package --list

# 4. Dry-run publish for each crate
cd comparable_helper && cargo publish --dry-run
cd ../comparable_derive && cargo publish --dry-run
cd ../comparable && cargo publish --dry-run
```

### Publishing Sequence

#### Step 1: Publish comparable_helper

```bash
cd comparable_helper
cargo publish
```

**Wait 2-5 minutes** for crates.io to index the package.

Verify publication:
```bash
# Check via cargo search
cargo search comparable_helper --limit 1

# Or visit in browser
open https://crates.io/crates/comparable_helper
```

#### Step 2: Publish comparable_derive

```bash
cd ../comparable_derive
cargo publish
```

**Wait 2-5 minutes** for crates.io to index the package.

Verify publication:
```bash
# Check via cargo search
cargo search comparable_derive --limit 1

# Or visit in browser
open https://crates.io/crates/comparable_derive
```

#### Step 3: Publish comparable (main crate)

```bash
cd ../comparable
cargo publish
```

Verify publication:
```bash
# Check via cargo search
cargo search comparable --limit 1

# Or visit in browser
open https://crates.io/crates/comparable
```

#### Step 4: Tag the release

```bash
git tag v0.5.6
git push --tags
```

## Important Notes

### Cargo.toml Path Dependencies

**No manual modifications needed!** Your current Cargo.toml files are correctly configured:

```toml
comparable_derive = { version = "0.5.6", optional = true, path = "../comparable_derive" }
comparable_helper = { version = "0.5.6", path = "../comparable_helper" }
```

When you run `cargo publish`, Cargo automatically:
- Strips the `path = "../comparable_*"` attributes
- Publishes with just `version = "0.5.6"` pointing to crates.io
- Requires that the specified versions exist on crates.io (hence dependency order)

### Generated Files

The file `comparable/src/lib.rs` is **auto-generated** from `README.md`. Always run `make build` before publishing to ensure it's up-to-date.

### Proc-Macro Crates

Both `comparable_helper` and `comparable_derive` are proc-macro crates (`proc-macro = true`). They have no special publishing requirements beyond normal crates.

### Indexing Wait Times

- **Modern Cargo (1.90+)**: `cargo publish` waits until the crate is downloadable before exiting
- **Traditional approach**: Still recommended to wait 2-5 minutes between publishes
- Indexing can take up to 10 minutes during high load on crates.io

### Verification Methods

Check if a crate version is available:

```bash
# Via cargo search
cargo search <crate-name> --limit 1

# Via temporary test project
mkdir /tmp/test-comparable && cd /tmp/test-comparable
cargo init
cargo add comparable_helper@0.5.6  # Should resolve immediately if indexed
```

## Version Bumping Workflow

When releasing a new version (e.g., 0.5.6 → 0.5.7):

1. Update version in all Cargo.toml files:
   - `comparable_helper/Cargo.toml`
   - `comparable_derive/Cargo.toml`
   - `comparable/Cargo.toml` (update both `[package]` version and dependency versions)
   - `comparable_test/Cargo.toml` (update comparable dependency version)

2. Update version in git commit history:
   ```bash
   git commit -am "Update version to 0.5.7"
   ```

3. Follow the publishing steps above

4. Update the git tag to match:
   ```bash
   git tag v0.5.7
   git push --tags
   ```

## Troubleshooting

### "crate version X.Y.Z already uploaded"

You've already published this version. Increment the version number - crates.io does not allow overwriting published versions.

### "failed to verify package tarball"

Run `cargo package` to see what's being included. Check that all necessary files are listed in the `include` directive.

### "no matching package found" when publishing dependent crate

The dependency hasn't finished indexing on crates.io. Wait a few more minutes and try again.

### "uncommitted changes"

Either commit your changes or use `cargo publish --allow-dirty` (not recommended for releases).

## References

- [Cargo Publishing Documentation](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [Workspace Publishing (Cargo 1.90+)](https://blog.rust-lang.org/2024/09/05/Rust-1.81.0.html#cargo-publish-in-workspaces)
- [crates.io Publishing Guidelines](https://doc.rust-lang.org/cargo/reference/manifest.html)
