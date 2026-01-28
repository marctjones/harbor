# Harbor - Claude Code Guide

**WORKSPACE NOTE**: Harbor is part of the `harbor-workspace`. For cross-project guidance, see `../CLAUDE.md` and `../.claude/rules/workspace.md`.

## Quick Links

- **Workspace Overview**: [../CLAUDE.md](../CLAUDE.md)
- **Rigging (dependency)**: [../rigging/CLAUDE.md](../rigging/CLAUDE.md)
- **Workspace Rules**: [../.claude/rules/workspace.md](../.claude/rules/workspace.md)

---

## Before Starting Any Work

**ALWAYS read `IMPLEMENTATION_PLAN.md` first** to understand:
- Current project status (what's complete, what's in progress)
- What phases are blocked and why
- The specific next tasks to work on
- Detailed step-by-step implementation plans

The implementation plan has checkboxes showing exactly where we left off.

## Project Overview

**Harbor** is a local desktop application framework - an **Electron alternative** built in Rust. It embeds Servo's rendering engine to display web UIs that communicate with local backends over Unix Domain Sockets.

**This is NOT a web browser.** Harbor cannot access the internet. It renders web content from local applications only.

```
Your Web App (Flask/etc) <--> Unix Socket <--> Harbor (Servo rendering) <--> User
```

## Build and Test Commands

**Workspace commands** (run from `harbor-workspace/`):
```bash
# Build Harbor (and Rigging if needed)
cargo build -p harbor

# Test Harbor
cargo test -p harbor

# Test entire workspace
cargo test

# Run Harbor CLI
cargo run -p harbor -- app.toml
cargo run -p harbor -- --example hello-flask
```

**Harbor-only commands** (run from `harbor/`):
```bash
# Build library
cargo build

# Run tests
cargo test

# Run specific test
cargo test test_backend_start

# Run tests in a module
cargo test config::tests

# Run the CLI
cargo run -- app.toml
cargo run -- --example hello-flask
cargo run -- --backend-only app.toml
```

## Architecture

Harbor depends on **Rigging** (a fork of servoshell's core embedding code):

```
Harbor (this repo)
  - app.toml configuration
  - BackendManager (gunicorn, Flask, etc.)
  - UdsConnector (blocks TCP)
      |
      v
Rigging (../rigging)
  - WebView API
  - Window management (winit/surfman)
  - Servo embedding
  - Pluggable Connector trait
      |
      v
Servo (upstream, minimal patches)
  - WebRender, Stylo, Layout, Script, etc.
```

### Key Source Files

- `src/config.rs` - TOML configuration parsing (AppConfig, BackendConfig, FrontendConfig)
- `src/backend.rs` - Backend process management (start, stop, socket waiting)
- `src/app.rs` - HarborApp main runner
- `src/main.rs` - CLI with clap (run, init, check, examples)
- `src/lib.rs` - Library exports, re-exports Rigging's Browser API

### Transport-Aware URLs

Harbor uses Rigging's URL format for Unix sockets:
```
http::unix///tmp/app.sock/path    # Absolute socket path (3 slashes)
http::unix//relative.sock/path    # Relative socket path (2 slashes)
http::pipe//pipename/path         # Windows named pipe
```

## Common Mistakes to Avoid

1. **DO NOT suggest WebKit/WRY/Tauri** - We embed Servo deliberately
2. **DO NOT add TCP networking** - UDS only, no `http://localhost:8000`
3. **DO NOT import Servo types directly** - Use `rigging::embed::*` API
4. **DO NOT add browser chrome** - No URL bar, tabs, bookmarks (this is NOT a browser)
5. **DO NOT use wrong URL format** - Use `http::unix:` not `unix://` or `http://localhost`

## Development Workflow

- **Write tests alongside code** - Do not defer testing
- **Commit after each successful test run** - Small, frequent commits
- **Use thiserror for library errors, anyhow for binaries**
- **Run `cargo test` before considering any task complete**

## Knowledge Management Strategy

This project uses a **four-tier content organization system** (same as llmfp, klareco, pdfe):

| Content Type | Where It Goes | Why |
|--------------|---------------|-----|
| **Concepts, algorithms, theory** | [Wiki](https://github.com/marctjones/harbor/wiki) | Educational, timeless reference |
| **Research, ideas, lab notes** | [Discussions](https://github.com/marctjones/harbor/discussions) | Unstructured exploration, feedback |
| **Bugs, features, tasks** | [Issues](https://github.com/marctjones/harbor/issues) | Actionable items with completion criteria |
| **Code documentation, setup** | Markdown files | Version-controlled, code-specific |

### Key Rules

1. **DO NOT create markdown files for educational content** - Use the Wiki
2. **DO NOT add TODO comments in code** - Create GitHub issues instead
3. **DO reference issue numbers** in commits: `Fixes #17` or `See issue #25`
4. **DO create issues proactively** when discovering bugs or enhancements

### GitHub CLI Commands

```bash
# Issues
gh issue list                              # List open issues
gh issue create --title "X" --label "bug"  # Create issue
gh issue close 123 --comment "Fixed"       # Close with comment

# Wiki (separate git repo)
git clone https://github.com/marctjones/harbor.wiki.git
# Edit .md files, then: git add . && git commit -m "msg" && git push
```

### Issue Labels

- **Priority**: `priority: critical|high|medium|low`
- **Type**: `bug`, `enhancement`, `documentation`, `research`
- **Component**: `component: backend`, `component: config`, `component: servo-integration`
- **Effort**: `effort: small|medium|large`

## Working with Rigging Dependency

Harbor depends on **Rigging** which is a workspace member at `../rigging/`.

### When Rigging's API Changes

1. Check Rigging's CLAUDE.md for API changes
2. Update Harbor's usage of Rigging APIs
3. Test workspace: `cargo test` (from workspace root)
4. Coordinate commits/PRs if needed

### Checking Rigging's API

```bash
# Read Rigging's public API
cat ../rigging/src/lib.rs

# Search for usage in Harbor
grep -r "rigging::" src/

# Check Rigging's implementation
cat ../rigging/src/embed/backend.rs
```

### When You Need Rigging Changes

1. Make changes in `../rigging/src/`
2. Test Rigging: `cargo test -p rigging`
3. Update Harbor to use new API
4. Test both: `cargo test` (workspace)
5. Commit both repos

See [workspace CLAUDE.md](../CLAUDE.md) for cross-project workflow details.

## Related Projects

- [Rigging](../rigging/) - Servo embedding API (workspace member, Harbor's main dependency)
- [Rigging GitHub](https://github.com/marctjones/rigging) - Rigging repository
- [Compass](https://github.com/marctjones/compass) - Privacy browser (also uses Rigging)
- [Servo](https://github.com/servo/servo) - Browser engine (embedded via Rigging)
