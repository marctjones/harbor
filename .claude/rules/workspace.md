# Workspace-Level Rules for Harbor Ecosystem

These rules apply to ALL projects in the harbor-workspace.

## Cross-Project Coordination

### When Changing Rigging's Public API

1. **Before making the change**:
   - Check Harbor's usage: `grep -r "rigging::" harbor/src/`
   - Identify breaking changes
   - Plan Harbor updates

2. **Making the change**:
   - Update Rigging code
   - Update Rigging's CLAUDE.md if API surface changes
   - Test Rigging: `cargo test -p rigging`

3. **After the change**:
   - Update Harbor to use new API
   - Test workspace: `cargo test`
   - Update both repos' documentation

### When Adding New Features to Harbor

1. **Check if Rigging needs updates**:
   - Does Harbor need new transport support?
   - Does Harbor need new Servo integration?
   - If yes, update Rigging first

2. **Implement in correct layer**:
   - Backend management → Harbor
   - Transport/networking → Rigging
   - Servo embedding → Rigging

## Shared Coding Standards

### Error Handling
- Libraries (Rigging): Use `thiserror` for custom errors
- Binaries (Harbor CLI): Use `anyhow` for error propagation
- Both: Provide context with `.context()` or `.with_context()`

### Logging
```rust
// Use structured logging
log::info!("Starting backend: {}", backend_name);
log::debug!("Socket ready: {}", socket_path);
log::error!("Failed to connect: {}", err);
```

### Testing
- Unit tests in same file: `#[cfg(test)] mod tests { ... }`
- Integration tests: `tests/` directory
- Test workspace before committing: `cargo test`

### Documentation
- Public APIs: Always add doc comments
- Examples: Include usage examples in doc comments
- README: Keep project READMEs updated

## Dependency Management

### Adding Workspace Dependencies

```toml
# Add to workspace Cargo.toml [workspace.dependencies]
new-crate = "1.0"

# Use in projects
[dependencies]
new-crate = { workspace = true }
```

### Project-Specific Dependencies

Only add if truly project-specific (not shared).

### Updating Dependencies

```bash
# Update all workspace dependencies
cargo update

# Check for outdated dependencies
cargo outdated  # (requires cargo-outdated)
```

## Git Commit Guidelines

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**: feat, fix, docs, refactor, test, chore

**Examples**:
```
feat(harbor): Add PTY support for terminal emulation
fix(rigging): Correct URL parsing for named pipes
docs(workspace): Update cross-project workflow guide
refactor(harbor): Simplify backend manager error handling
```

### Co-Authoring with Claude

```
git commit -m "feat(harbor): Add widget protocol support

Implemented JSON-based protocol for TUI widget serialization.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

## Security Practices

### Code Review Checklist

- [ ] No hardcoded credentials or API keys
- [ ] No SQL injection vulnerabilities
- [ ] No command injection (especially in Harbor's backend spawning)
- [ ] No path traversal vulnerabilities
- [ ] Input validation on all user-provided data
- [ ] Proper error messages (don't leak sensitive info)

### Harbor-Specific Security

- [ ] Backend MUST only use Unix sockets (never TCP)
- [ ] Socket permissions set to user-only (0600)
- [ ] External links delegated to OS browser (never navigate Harbor window)
- [ ] No network access from Servo (UdsConnector blocks TCP)

### Rigging-Specific Security

- [ ] Connector trait properly enforces transport restrictions
- [ ] URL parsing rejects invalid transport schemes
- [ ] No arbitrary code execution via transport URLs

## Performance Guidelines

### Build Times

- Keep compilation units small
- Use workspace dependencies (shared builds)
- Avoid heavy proc macros if possible

### Runtime Performance

- Async where appropriate (I/O operations)
- Avoid unnecessary allocations
- Profile before optimizing: `cargo flamegraph`

### Binary Size

- Use `strip = true` in release profile (already configured)
- Consider `opt-level = "z"` for size-critical builds
- Check binary size: `cargo build --release && ls -lh target/release/`

## Release Coordination

### Version Numbering (Semantic Versioning)

- **Major**: Breaking API changes (0.x → 1.0, 1.0 → 2.0)
- **Minor**: New features, backward compatible
- **Patch**: Bug fixes, no API changes

### Release Order

1. **Rigging** releases first (it's the dependency)
2. **Harbor** releases after (depends on Rigging)
3. **Compass/Corsair** release when ready (depend on both)

### Release Checklist

- [ ] All tests pass: `cargo test`
- [ ] Update CHANGELOG.md
- [ ] Update version in Cargo.toml
- [ ] Tag release: `git tag v0.x.0`
- [ ] Push tag: `git push origin v0.x.0`
- [ ] GitHub release with notes
- [ ] (Future) Publish to crates.io: `cargo publish -p rigging`

## IDE and Tooling

### Recommended Tools

```bash
# Linting
cargo clippy

# Formatting
cargo fmt

# Security audit
cargo audit

# Outdated dependencies
cargo outdated

# Benchmarking
cargo bench

# Documentation
cargo doc --open
```

### Editor Config

- **rust-analyzer**: Workspace mode works automatically
- **VSCode**: Open `harbor-workspace/` folder
- **IntelliJ**: Import workspace Cargo.toml

## Continuous Integration

### GitHub Actions Workflow (Future)

```yaml
# .github/workflows/ci.yml (workspace-level)
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build workspace
        run: cargo build --workspace
      - name: Test workspace
        run: cargo test --workspace
      - name: Clippy
        run: cargo clippy --workspace -- -D warnings
```

## Troubleshooting

### Build Errors After Rigging Changes

```bash
# Clean and rebuild
cargo clean
cargo build
```

### Dependency Resolution Issues

```bash
# Update Cargo.lock
cargo update
```

### "Package not found" Errors

Check workspace Cargo.toml `members` array includes all projects.

## Questions or Issues?

- **Harbor issues**: https://github.com/marctjones/harbor/issues
- **Rigging issues**: https://github.com/marctjones/rigging/issues
- **Workspace discussion**: Create issue in primary project (Harbor)
