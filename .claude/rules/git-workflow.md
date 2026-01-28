# Git Workflow for Harbor Workspace

## Repository Structure

The workspace has TWO git repositories:

```
/home/marc/Projects/
├── harbor/                  # Harbor GitHub repo + workspace root
│   ├── .git/                # Harbor's git repo (ON GITHUB)
│   │   └── tracks: Cargo.toml, CLAUDE.md, .claude/, src/, examples/, etc.
│   └── rigging/             # Symlink → ../rigging/
│
└── rigging/                 # Rigging GitHub repo (separate)
    └── .git/                # Rigging's git repo (ON GITHUB)
        └── tracks: rigging source code
```

### Harbor Repo (ON GITHUB)

**Location**: `/home/marc/Projects/harbor/.git`

**Remote**: `git@github.com:marctjones/harbor.git`

**What it tracks**:
- `Cargo.toml` - Workspace configuration + Harbor package
- `CLAUDE.md` - Harbor/workspace guide
- `.claude/` - Harbor rules and hooks
- `src/` - Harbor source code
- `examples/` - Harbor examples
- `target/` - Ignored (build artifacts)
- `rigging/` - Ignored (symlink to ../rigging/)

**What it ignores**:
- `rigging/` - Symlink to separate repo
- `target/` - Build artifacts
- `Cargo.lock` - Generated (for libraries)

**GitHub**: Pushed to GitHub (main repo)

### Rigging Repo (ON GITHUB)

**Location**: `/home/marc/Projects/rigging/.git`

**Remote**: `git@github.com:marctjones/rigging.git`

**What it tracks**: All Rigging source code, patches, docs

**Accessed via**: Symlink at `/home/marc/Projects/harbor/rigging` → `../rigging/`

## Committing Changes

### Harbor Files

```bash
# Harbor source code, examples, workspace config
cd /home/marc/Projects/harbor
git status                    # Check harbor repo
git add src/ Cargo.toml CLAUDE.md .claude/
git commit -m "feat: add feature"
git push origin main          # Push to GitHub
```

### Rigging Files

```bash
# Rigging source code (accessed via symlink or directly)
cd /home/marc/Projects/rigging
git status                    # Check rigging repo
git add src/
git commit -m "feat: add feature"
git push origin main          # Push to GitHub
```

## Claude Code Behavior

### Files Claude Can Create

**Harbor directory** (`/home/marc/Projects/harbor/`):
- ✅ Harbor source files (tracked in Harbor GitHub repo)
- ✅ Harbor examples, docs, tests
- ✅ Workspace config files (Cargo.toml, CLAUDE.md, .claude/)
- ⚠️ IMPORTANT: These ARE on GitHub (Harbor repo)

**Rigging directory** (`/home/marc/Projects/rigging/`):
- ✅ Rigging source files (tracked in Rigging GitHub repo)
- ✅ Rigging examples, docs, tests

### Where to Put Different Files

| File Type | Location | Tracked By | On GitHub? |
|-----------|----------|------------|------------|
| Workspace Cargo.toml | `/home/marc/Projects/harbor/` | Harbor repo | ✅ Yes |
| Harbor CLAUDE.md | `/home/marc/Projects/harbor/` | Harbor repo | ✅ Yes |
| Harbor rules | `/home/marc/Projects/harbor/.claude/` | Harbor repo | ✅ Yes |
| Integration tests (cross-project) | `/home/marc/Projects/harbor/tests/` | Harbor repo | ✅ Yes |
| Harbor source code | `/home/marc/Projects/harbor/src/` | Harbor repo | ✅ Yes |
| Rigging source code | `/home/marc/Projects/rigging/src/` | Rigging repo | ✅ Yes |

## Best Practices

### 1. Check Which Repo You're In

```bash
# Show current git repo
pwd
git remote -v  # Shows GitHub remote
```

### 2. Be Explicit About Where to Create Files

When asking Claude to create files:
- ❌ "Create a config file" (ambiguous location)
- ✅ "Create src/config.rs" (explicit, in Harbor)
- ✅ "Create an integration test at tests/integration.rs" (explicit, in Harbor)

### 3. Working with Rigging via Symlink

The symlink at `harbor/rigging/` points to `../rigging/`:

```bash
# Option 1: Work through symlink (from Harbor)
cd /home/marc/Projects/harbor/rigging
git status  # Shows Rigging repo status
# Edit files...
git commit -m "Update"

# Option 2: Work directly (clearer)
cd /home/marc/Projects/rigging
git status
# Edit files...
git commit -m "Update"
```

Both work the same way - the symlink just makes it convenient to access Rigging from Harbor's workspace.

### 4. Integration Tests

Cross-project tests go in Harbor's tests directory:

```bash
# Create Harbor-level integration test
/home/marc/Projects/harbor/tests/
└── integration/
    ├── harbor_with_rigging.rs
    └── uds_connector.rs
```

These are tracked in Harbor repo (on GitHub).

## Checking File Tracking

```bash
# Which repo tracks this file?

# Harbor files
cd /home/marc/Projects/harbor
git status Cargo.toml          # Shows Harbor repo status
git ls-files Cargo.toml        # Listed if tracked by Harbor

# Rigging files
cd /home/marc/Projects/rigging
git status src/lib.rs          # Shows Rigging repo status
git ls-files src/lib.rs        # Listed if tracked by Rigging
```

## Recovery Scenarios

### Lost Harbor Config

If Harbor config is lost (deleted, corrupted):

```bash
cd /home/marc/Projects/harbor
git log                        # View Harbor history
git checkout Cargo.toml        # Restore from Harbor repo
```

### Fresh Clone of Harbor/Rigging

To recreate workspace from scratch:

```bash
cd /home/marc/Projects/

# Clone Harbor (includes workspace config)
git clone git@github.com:marctjones/harbor.git

# Clone Rigging
git clone git@github.com:marctjones/rigging.git

# Create symlink
cd harbor
ln -s ../rigging rigging

# Build
cargo build
```

## Summary

- **2 git repos**: Harbor (GitHub), Rigging (GitHub)
- **Harbor is the workspace root**: Workspace config on Harbor's GitHub
- **Rigging accessed via symlink**: `harbor/rigging/` → `../rigging/`
- **All files tracked on GitHub**: No local-only files
- **When committing**: Check which repo directory you're in
