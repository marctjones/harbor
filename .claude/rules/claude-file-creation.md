# Claude File Creation Rules

## CRITICAL: Understanding the Two Git Repositories

This workspace has **TWO separate git repositories**:

1. **Harbor repo** (`/home/marc/Projects/harbor/.git`) - Synced to GitHub
2. **Rigging repo** (`/home/marc/Projects/rigging/.git`) - Synced to GitHub

Harbor is both a package AND the workspace root. Rigging is accessed via symlink.

## Where Claude Can Create Files

### ✅ Harbor Directory (GitHub Tracking)

**Location**: `/home/marc/Projects/harbor/`

**Tracked by**: Harbor git repo (pushed to GitHub)

**Examples**:
- `Cargo.toml` - Workspace configuration + Harbor package
- `CLAUDE.md` - Harbor/workspace documentation
- `.claude/rules/*.md` - Harbor/workspace rules
- `src/*.rs` - Harbor source code
- `examples/*/` - Harbor examples
- `tests/integration/*.rs` - Integration tests (Harbor + Rigging)

**Important**:
- These files ARE on GitHub (Harbor repo)
- Workspace config is tracked here
- Integration tests for both Harbor and Rigging go here

### ✅ Rigging Directory (GitHub Tracking)

**Location**: `/home/marc/Projects/rigging/`

**Accessed via symlink**: `/home/marc/Projects/harbor/rigging/` → `../rigging/`

**Tracked by**: Rigging git repo (pushed to GitHub)

**Examples**:
- `src/*.rs` - Rigging source code
- `examples/` - Rigging examples
- `CLAUDE.md` - Rigging documentation
- `tests/*.rs` - Rigging-specific tests

**Important**:
- These files ARE on GitHub (Rigging repo)
- Changes here should be committed to Rigging repo
- Can access via symlink or directly

## Decision Guide: Where Should This File Go?

### Harbor Files

**Create in Harbor if**:
- File is workspace configuration (Cargo.toml)
- File is Harbor source code
- File is Harbor-specific docs/examples/tests
- File is integration test spanning Harbor and Rigging
- File is Harbor development tooling

**Don't create in Harbor if**:
- File is Rigging source code (belongs in rigging/)

### Rigging Files

**Create in Rigging if**:
- File is Rigging source code
- File is Rigging-specific documentation
- File is Rigging example
- File is Rigging test

## Examples

### ✅ Correct Placements

```bash
# Workspace config (Harbor repo)
/home/marc/Projects/harbor/Cargo.toml

# Integration test (uses both Harbor and Rigging) - Harbor repo
/home/marc/Projects/harbor/tests/integration/harbor_rigging.rs

# Harbor source code (Harbor repo)
/home/marc/Projects/harbor/src/backend.rs

# Rigging source code (Rigging repo, accessed via symlink)
/home/marc/Projects/rigging/src/connector.rs
# OR via symlink:
/home/marc/Projects/harbor/rigging/src/connector.rs
```

### ❌ Incorrect Placements

```bash
# WRONG: Can't create files in symlink target from Harbor context
# When in Harbor, if you need to edit Rigging, navigate to actual Rigging repo
```

## When in Doubt

**Ask yourself**:
1. Is this file Harbor-specific? → `/home/marc/Projects/harbor/`
2. Is this file Rigging-specific? → `/home/marc/Projects/rigging/`
3. Is this file shared/integration? → `/home/marc/Projects/harbor/tests/`
4. Does this need workspace config? → `/home/marc/Projects/harbor/Cargo.toml`

## Preventing Mistakes

### Before Creating a File

```bash
# Check current directory
pwd

# Check which git repo you're in
git remote -v

# Harbor repo:
# origin  git@github.com:marctjones/harbor.git

# Rigging repo:
# origin  git@github.com:marctjones/rigging.git
```

### After Creating Files

```bash
# Verify file is tracked correctly
git status

# If in Harbor:
# - File should show in Harbor repo
# - rigging/ should NOT appear (it's ignored)

# If in Rigging:
# - File should show in Rigging repo
```

## Claude Instructions

When Claude needs to create a file:

1. **Determine correct location** based on file purpose
2. **Use absolute path** in tool call: `/home/marc/Projects/harbor/src/file.rs`
3. **Verify placement** matches these rules
4. **Inform user** which repo tracks the file

## User Instructions

When asking Claude to create files:

1. **Be explicit about location**:
   - ❌ "Create a test file"
   - ✅ "Create tests/backend_test.rs in Harbor"

2. **Specify if integration**:
   - "Create an integration test at tests/integration/test_name.rs"

3. **Ask for verification**:
   - "After creating the file, show me which git repo tracks it"

## Summary Table

| Location | Git Repo | On GitHub? | Purpose |
|----------|----------|------------|---------|
| `/home/marc/Projects/harbor/` | Harbor | ✅ Yes | Harbor code, workspace config, integration tests |
| `/home/marc/Projects/rigging/` | Rigging | ✅ Yes | Rigging source, docs, tests |
| `/home/marc/Projects/harbor/rigging/` | Rigging (symlink) | ✅ Yes | Access Rigging from Harbor workspace |

## Quick Reference

```bash
# Harbor files (workspace + Harbor code)
cd /home/marc/Projects/harbor
git status  # Shows Harbor files only

# Rigging files (via symlink)
cd /home/marc/Projects/harbor/rigging
git status  # Shows Rigging files only

# Rigging files (direct)
cd /home/marc/Projects/rigging
git status  # Shows Rigging files only
```
