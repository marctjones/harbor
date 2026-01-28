# Harbor - Local Desktop App Framework

**Harbor** is both a Rust package (local desktop app framework) and a Cargo workspace that includes Rigging (Servo embedding library).

## Quick Reference

```bash
# Build everything
cd /home/marc/Projects/harbor
cargo build

# Test everything
cargo test

# Run Harbor CLI
cargo run -- app.toml
cargo run -- --example hello-flask
```

## Repository Structure

```
/home/marc/Projects/
├── harbor/                  # Harbor repo + workspace root (THIS REPO)
│   ├── .git/                # Harbor GitHub repo
│   ├── Cargo.toml           # Workspace config + Harbor package
│   ├── src/                 # Harbor source code
│   ├── examples/            # Harbor examples
│   ├── rigging/             # Symlink → ../rigging/
│   └── .claude/rules/       # Workspace rules
│
└── rigging/                 # Rigging repo (separate)
    └── .git/                # Rigging GitHub repo
```

**Important**: `rigging/` in Harbor is a **symlink** to `../rigging/`. Both repos are on GitHub.

## Project Relationships

```
Harbor (Desktop Apps)
  ↓ depends on
Rigging (Servo Embedding + Transport)
  ↓ uses
Servo (Rendering Engine)

Compass (Privacy Browser) - sibling project, not in this workspace
  ↓ depends on
Rigging + Corsair (Tor Daemon)
```

**Note**: Compass is a sibling to Harbor, not a child. Both depend on Rigging.

## Before Starting Any Work

**ALWAYS read `IMPLEMENTATION_PLAN.md` first** to understand:
- Current project status (what's complete, what's in progress)
- What phases are blocked and why
- The specific next tasks to work on
- Detailed step-by-step implementation plans

## What is Harbor?

**Harbor** is a local desktop application framework - an **Electron alternative** built in Rust. It embeds Servo's rendering engine to display web UIs that communicate with local backends over Unix Domain Sockets.

**This is NOT a web browser.** Harbor cannot access the internet. It renders web content from local applications only.

```
Your Web App (Flask/etc) <--> Unix Socket <--> Harbor (Servo rendering) <--> User
```

## Build and Test Commands

```bash
# Build entire workspace (Harbor + Rigging)
cargo build

# Build specific package
cargo build -p harbor
cargo build -p rigging

# Test entire workspace
cargo test

# Test specific package
cargo test -p harbor
cargo test -p rigging

# Run Harbor CLI
cargo run -- app.toml
cargo run -- --example hello-flask
cargo run -- --backend-only app.toml
```

## Working Across Projects

### Making Changes to Rigging

```bash
# Option 1: Via symlink (from Harbor)
cd /home/marc/Projects/harbor/rigging
git status  # Shows Rigging repo
# Edit, commit, push

# Option 2: Direct (clearer)
cd /home/marc/Projects/rigging
git status  # Shows Rigging repo
# Edit, commit, push
```

After Rigging changes:
```bash
cd /home/marc/Projects/harbor
cargo test -p rigging  # Test Rigging
cargo test -p harbor   # Verify Harbor still works
```

### Making Changes to Harbor

```bash
cd /home/marc/Projects/harbor
# Edit Harbor files
cargo test -p harbor
git add src/ examples/
git commit -m "feat: ..."
git push
```

### API Changes Affecting Both

1. Update Rigging API first (in `/home/marc/Projects/rigging/`)
2. Update Harbor to use new API
3. Test workspace: `cargo test`
4. Commit both repos (coordinate PRs if needed)

## Architecture

Harbor depends on **Rigging** (a fork of servoshell's core embedding code):

```
Harbor (this repo)
  - app.toml configuration
  - BackendManager (gunicorn, Flask, etc.)
  - UdsConnector (blocks TCP)
      |
      v
Rigging (rigging/ symlink → ../rigging/)
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

## Key Design Principles

### Security First
- **Harbor**: Unix sockets ONLY - no TCP networking allowed
- **Rigging**: Pluggable transport layer with connectors
- **Compass** (sibling project): Tor-only browsing via Corsair integration

### Local First
- All applications run entirely on user's machine
- No cloud dependencies
- Data stays local

### Servo Ecosystem
- Built on Servo rendering engine
- Support independent web platform implementation
- Full control over rendering and networking

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

## Working with Rigging Dependency

Harbor depends on **Rigging** which is a workspace member via symlink at `rigging/` → `../rigging/`.

### When Rigging's API Changes

1. Check Rigging's CLAUDE.md for API changes
2. Update Harbor's usage of Rigging APIs
3. Test workspace: `cargo test`
4. Coordinate commits/PRs if needed

### Checking Rigging's API

```bash
# Read Rigging's public API
cat rigging/src/lib.rs

# Search for usage in Harbor
grep -r "rigging::" src/

# Check Rigging's implementation
cat rigging/src/embed/backend.rs
```

### When You Need Rigging Changes

1. Make changes in `/home/marc/Projects/rigging/src/`
2. Test Rigging: `cargo test -p rigging`
3. Update Harbor to use new API
4. Test both: `cargo test`
5. Commit both repos

## Git Workflow

See [.claude/rules/git-workflow.md](./.claude/rules/git-workflow.md) for complete details.

### Quick Reference

**Harbor files** - Pushed to Harbor's GitHub:
```bash
cd /home/marc/Projects/harbor
git add src/ Cargo.toml CLAUDE.md
git commit -m "feat: add feature"
git push origin main
```

**Rigging files** - Pushed to Rigging's GitHub:
```bash
cd /home/marc/Projects/rigging
# OR: cd /home/marc/Projects/harbor/rigging
git add src/
git commit -m "feat: add feature"
git push origin main
```

## Knowledge Management Strategy

This project uses a **four-tier content organization system**:

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

## Adding New Dependencies

### Workspace-wide dependency

```toml
# In workspace Cargo.toml [workspace.dependencies]
new-crate = "1.0"

# In [dependencies] section
new-crate = { workspace = true }
```

### Project-specific dependency

```toml
# In [dependencies] section (not in workspace)
harbor-only-crate = "1.0"
```

## Related Projects

- [Rigging](https://github.com/marctjones/rigging) - Servo embedding API (workspace member, Harbor's main dependency)
- [Compass](https://github.com/marctjones/compass) - Privacy browser (sibling project, also uses Rigging)
- [Servo](https://github.com/servo/servo) - Browser engine (embedded via Rigging)

## Current Status

- **Harbor**: Phase 1 complete (backend management), Phase 5 blocked (Servo integration)
- **Rigging**: In development (forking servoshell, adding Connector trait)

See `IMPLEMENTATION_PLAN.md` for detailed status and next steps.
