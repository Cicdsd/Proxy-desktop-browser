# GitHub Copilot Agent Execution Guide
## Node.js → Bun Migration

> **Target Agent**: GitHub Copilot Workspace / Chat
> **Execution Mode**: Step-by-step with validation
> **Estimated Duration**: 2-4 hours

---

## 🎯 Mission Statement

Migrate the Virtual IP Browser UI from Node.js/npm to Bun runtime while maintaining 100% functionality and improving performance by 2-3x.

---

## 📋 Pre-Flight Checklist

Before starting, verify:
- [ ] Current code is committed to Git
- [ ] Working directory is clean (`git status`)
- [ ] You have reviewed `MIGRATION_PLAN_NODEJS_TO_BUN.md`
- [ ] Bun is installed or ready to install

---

## 🚀 PHASE 1: Environment Setup

### Step 1.1: Install Bun Runtime

**Copilot Prompt**:
```
@terminal Install Bun runtime on Windows using PowerShell.
Run: irm bun.sh/install.ps1 | iex
Then verify with: bun --version
```

**Expected Output**:
```
Bun v1.x.x
```

**Validation**:
```powershell
bun --version
bun --help
```

---

### Step 1.2: Create Baseline Backup

**Copilot Prompt**:
```
@terminal Create a git tag for the current Node.js baseline:
git tag v1.0.0-nodejs-baseline -m "Baseline before Bun migration"
git push origin v1.0.0-nodejs-baseline
```

**Validation**:
```powershell
git tag -l
```

---

## 🔧 PHASE 2: Configuration Files Update

### Step 2.1: Update package.json

**Copilot Prompt**:
```
@workspace Update ui-tauri/package.json:
1. Change all "npm run" to "bun run" in scripts
2. Use "bunx --bun vite" for dev and build scripts
3. Replace "vitest" and "@vitest/ui" with Bun's native test runner
4. Keep all other dependencies unchanged

Current scripts section:
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "tauri": "tauri dev",
    "tauri:build": "tauri build",
    "check": "svelte-check --tsconfig ./tsconfig.json",
    "test": "vitest",
    "test:ui": "vitest --ui",
    "format": "prettier --write .",
    "lint": "eslint ."
  }

Update to use Bun commands.
```

**Expected Changes**:
```json
{
  "scripts": {
    "dev": "bunx --bun vite",
    "build": "bunx --bun vite build",
    "preview": "bunx --bun vite preview",
    "tauri": "bun tauri dev",
    "tauri:build": "bun tauri build",
    "check": "svelte-check --tsconfig ./tsconfig.json",
    "test": "bun test",
    "test:watch": "bun test --watch",
    "format": "prettier --write .",
    "lint": "eslint ."
  }
}
```

**Manual Action Required**: Remove these from devDependencies:
- `"vitest": "^2.0.0"`
- `"@vitest/ui": "^2.0.0"`

---

### Step 2.2: Update tauri.conf.json

**Copilot Prompt**:
```
@workspace In ui-tauri/src-tauri/tauri.conf.json, update the build commands:
- Line 6: Change "beforeDevCommand": "npm run dev" to "bun run dev"
- Line 8: Change "beforeBuildCommand": "npm run build" to "bun run build"
Keep everything else unchanged.
```

**File**: `ui-tauri/src-tauri/tauri.conf.json`

**Expected Changes** (lines 5-10):
```json
"build": {
  "beforeDevCommand": "bun run dev",
  "devUrl": "http://localhost:5173",
  "beforeBuildCommand": "bun run build",
  "frontendDist": "../dist"
}
```

---

### Step 2.3: Update tsconfig.json

**Copilot Prompt**:
```
@workspace Update ui-tauri/tsconfig.json for Bun compatibility:
1. Add "bun-types" to compilerOptions.types array
2. Change lib to ["ES2022", "DOM"]
3. Add "module": "ESNext" and "moduleResolution": "bundler"
4. Clean up exclude array to remove node_modules reference from src
```

**Expected Result**:
```json
{
  "compilerOptions": {
    "types": ["svelte", "bun-types"],
    "baseUrl": ".",
    "paths": {
      "$lib/*": ["src/lib/*"]
    },
    "lib": ["ES2022", "DOM"],
    "module": "ESNext",
    "target": "ESNext",
    "moduleResolution": "bundler"
  },
  "include": ["src/**/*", "vite.config.ts"],
  "exclude": ["node_modules", "ui-tauri/src-tauri", "dist"]
}
```

---

### Step 2.4: Update .gitignore

**Copilot Prompt**:
```
@workspace Add Bun-specific entries to .gitignore:
- .bun/
- bun.lockb

Add these after the "# Node.js / npm" section.
```

**Expected Addition**:
```gitignore
# Bun
.bun/
bun.lockb

# Node.js / npm (keep node_modules)
node_modules/
```

---

### Step 2.5: Create Bun Configuration (Optional)

**Copilot Prompt**:
```
@workspace Create a new file ui-tauri/bunfig.toml with Bun runtime configuration for tests and installation.
```

**File**: `ui-tauri/bunfig.toml`

**Content**:
```toml
[test]
# Test configuration
preload = []
coverage = true

[install]
# Package installation config
cache = "~/.bun/install/cache"
lockfile = true
production = false

[run]
# Runtime configuration
bun = true
```

---

## 🧪 PHASE 3: Test Files Migration

### Step 3.1: Find All Test Files

**Copilot Prompt**:
```
@workspace Search for all test files in ui-tauri/src/ that use Vitest.
Look for files with .test.ts, .test.tsx, .spec.ts, or .spec.tsx extensions.
List all found files.
```

**Manual Command**:
```powershell
Get-ChildItem -Path ui-tauri/src -Recurse -Include *.test.ts,*.test.tsx,*.spec.ts,*.spec.tsx
```

---

### Step 3.2: Convert Test Files

**Copilot Prompt**:
```
@workspace For each test file found, convert from Vitest to Bun test:

1. Change imports:
   FROM: import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
   TO: import { describe, test, expect, mock, beforeEach, afterEach } from 'bun:test';

2. Replace all instances of it() with test()

3. Replace vi.fn() with mock()

4. Keep all test logic unchanged

5. Preserve all assertions and expectations

Example transformation:
BEFORE:
```typescript
import { describe, it, expect } from 'vitest';
describe('Component', () => {
  it('should work', () => {
    expect(true).toBe(true);
  });
});
```

AFTER:
```typescript
import { describe, test, expect } from 'bun:test';
describe('Component', () => {
  test('should work', () => {
    expect(true).toBe(true);
  });
});
```

Apply this pattern to all test files.
```

---

### Step 3.3: Update Test Setup Files

**Copilot Prompt**:
```
@workspace Search for test setup files like:
- setupTests.ts
- test-setup.ts
- vitest.config.ts

If found, update imports to use 'bun:test' instead of 'vitest'.
If vitest.config.ts exists, it can be deleted (Bun doesn't need it).
```

---

## 📦 PHASE 4: Dependency Migration

### Step 4.1: Clean Node.js Artifacts

**Copilot Prompt**:
```
@terminal Remove all Node.js/npm artifacts:
cd ui-tauri
Remove-Item -Recurse -Force node_modules -ErrorAction SilentlyContinue
Remove-Item -Force package-lock.json -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force .vitest -ErrorAction SilentlyContinue
```

**Validation**:
```powershell
# Verify removal
Test-Path ui-tauri/node_modules  # Should be False
Test-Path ui-tauri/package-lock.json  # Should be False
```

---

### Step 4.2: Install with Bun

**Copilot Prompt**:
```
@terminal Install all dependencies using Bun:
cd ui-tauri
bun install
```

**Expected Output**:
```
bun install v1.x.x
 + @tauri-apps/api@2.0.0
 + @tauri-apps/plugin-store@2.0.0
 ...
 [number] packages installed [time]
```

**Validation**:
```powershell
# Check for bun.lockb
Test-Path ui-tauri/bun.lockb  # Should be True

# List installed packages
bun pm ls
```

---

### Step 4.3: Add Bun Type Definitions

**Copilot Prompt**:
```
@terminal Add Bun's TypeScript definitions:
cd ui-tauri
bun add -d bun-types
```

---

## ✅ PHASE 5: Validation & Testing

### Step 5.1: Test Development Server

**Copilot Prompt**:
```
@terminal Start the Vite dev server with Bun:
cd ui-tauri
bun run dev
```

**Expected Output**:
```
VITE v5.x.x  ready in XXX ms
➜  Local:   http://localhost:5173/
```

**Success Criteria**:
- Server starts without errors
- Opens on port 5173
- Hot reload works

**Stop server**: Press `Ctrl+C`

---

### Step 5.2: Test Tauri Development

**Copilot Prompt**:
```
@terminal Launch Tauri in development mode:
cd ui-tauri
bun run tauri
```

**Expected Behavior**:
- Vite dev server starts
- Rust backend compiles
- Tauri window opens
- Application loads correctly

**Success Criteria**:
- No compilation errors
- UI renders properly
- All features work

**Stop app**: Close the window

---

### Step 5.3: Test Production Build

**Copilot Prompt**:
```
@terminal Build the UI for production:
cd ui-tauri
bun run build
```

**Expected Output**:
```
vite v5.x.x building for production...
✓ built in XXXms
```

**Validation**:
```powershell
# Check dist folder created
Test-Path ui-tauri/dist  # Should be True
Get-ChildItem ui-tauri/dist
```

---

### Step 5.4: Test Full Build

**Copilot Prompt**:
```
@terminal Build the complete Tauri application:
cd ui-tauri
bun run tauri:build
```

**Expected Output**:
- Vite builds successfully
- Rust compiles successfully
- Installer created in `ui-tauri/src-tauri/target/release/bundle/`

**Success Criteria**:
- No build errors
- Executable created
- Installer package created (MSI for Windows)

---

### Step 5.5: Run Tests

**Copilot Prompt**:
```
@terminal Run all tests with Bun:
cd ui-tauri
bun test
```

**Expected Output**:
```
bun test v1.x.x

test/example.test.ts:
✓ test name [time]

X tests passed (X)
```

**Success Criteria**:
- All tests pass
- No import errors
- No assertion failures

---

### Step 5.6: Test Type Checking

**Copilot Prompt**:
```
@terminal Run TypeScript type checking:
cd ui-tauri
bun run check
```

**Expected Output**:
```
svelte-check
========
No errors found!
```

---

### Step 5.7: Test Linting

**Copilot Prompt**:
```
@terminal Run ESLint:
cd ui-tauri
bun run lint
```

**Expected Output**:
- No errors (or existing errors unrelated to migration)

---

### Step 5.8: Test Formatting

**Copilot Prompt**:
```
@terminal Run Prettier:
cd ui-tauri
bun run format
```

**Expected Output**:
- Files formatted successfully

---

## 📊 PHASE 6: Performance Benchmarking

### Step 6.1: Benchmark Installation Speed

**Copilot Prompt**:
```
@terminal Benchmark Bun installation:
cd ui-tauri
Remove-Item -Recurse -Force node_modules
Measure-Command { bun install }
```

**Record**: Total milliseconds

---

### Step 6.2: Benchmark Build Speed

**Copilot Prompt**:
```
@terminal Benchmark build time:
cd ui-tauri
Remove-Item -Recurse -Force dist
Measure-Command { bun run build }
```

**Record**: Total milliseconds

---

### Step 6.3: Benchmark Test Speed

**Copilot Prompt**:
```
@terminal Benchmark test execution:
cd ui-tauri
Measure-Command { bun test }
```

**Record**: Total milliseconds

---

## 📝 PHASE 7: Documentation Updates

### Step 7.1: Update README.md

**Copilot Prompt**:
```
@workspace Update README.md to reflect Bun migration:

1. In Prerequisites section, replace Node.js with Bun
2. Update installation instructions to use "bun install"
3. Update all command examples from "npm run" to "bun run"
4. Add a note about Bun's performance benefits

Find sections mentioning:
- Node.js installation
- npm install
- npm run commands

Replace with Bun equivalents.
```

**Key Changes**:
```markdown
## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Bun](https://bun.sh/) v1.0+ (replaces Node.js)
- [Git](https://git-scm.com/)

## Installation

```bash
# Install dependencies with Bun
cd ui-tauri
bun install
```

## Development

```bash
# Start development server
bun run tauri

# Run tests
bun test

# Build for production
bun run tauri:build
```
```

---

### Step 7.2: Update GETTING_STARTED.md

**Copilot Prompt**:
```
@workspace Update GETTING_STARTED.md:
1. Replace Node.js references with Bun
2. Update installation steps
3. Update command examples
4. Add Bun-specific tips
```

---

### Step 7.3: Create Migration Documentation

**Copilot Prompt**:
```
@workspace Create a new file MIGRATION_COMPLETED.md documenting:
1. Migration date
2. Bun version used
3. What was changed
4. Performance improvements measured
5. Any issues encountered
6. Rollback instructions (if needed)
```

---

## 🎉 PHASE 8: Finalization

### Step 8.1: Verify All Changes

**Copilot Prompt**:
```
@terminal Review all changes:
git status
git diff
```

**Checklist**:
- [ ] package.json updated
- [ ] tauri.conf.json updated
- [ ] tsconfig.json updated
- [ ] .gitignore updated
- [ ] Test files converted
- [ ] Node.js artifacts removed
- [ ] bun.lockb created
- [ ] All tests pass
- [ ] Documentation updated

---

### Step 8.2: Commit Changes

**Copilot Prompt**:
```
@terminal Stage and commit all migration changes:
git add .
git commit -m "chore: migrate from Node.js/npm to Bun runtime

- Update package.json scripts for Bun commands
- Update tauri.conf.json build commands
- Configure TypeScript for Bun compatibility
- Convert Vitest tests to Bun test runner
- Update .gitignore for Bun artifacts
- Update documentation (README, GETTING_STARTED)
- Remove Node.js artifacts (node_modules, package-lock.json)

Performance improvements:
- Installation: ~60% faster
- Tests: ~2x faster
- Native TypeScript support

Breaking changes: None
Migration guide: See MIGRATION_PLAN_NODEJS_TO_BUN.md"
```

---

### Step 8.3: Create Migration Tag

**Copilot Prompt**:
```
@terminal Create version tag:
git tag v1.1.0-bun-migration -m "Migrated to Bun runtime"
```

---

### Step 8.4: Push to GitHub

**Copilot Prompt**:
```
@terminal Push changes and tags:
git push origin main
git push origin --tags
```

---

## 🔍 PHASE 9: Post-Migration Verification

### Step 9.1: Clean Install Test

**Copilot Prompt**:
```
@terminal Test clean installation in new directory:
cd ..
git clone <your-repo-url> test-migration
cd test-migration/ui-tauri
bun install
bun run dev
```

**Success Criteria**:
- Repository clones successfully
- Bun installs dependencies
- Dev server starts
- No errors

---

### Step 9.2: Full Build Test

**Copilot Prompt**:
```
@terminal Test full production build:
cd test-migration/ui-tauri
bun run tauri:build
```

**Success Criteria**:
- Build completes successfully
- Executable runs
- All features work

---

### Step 9.3: Clean Up Test Directory

**Copilot Prompt**:
```
@terminal Remove test directory:
cd ../..
Remove-Item -Recurse -Force test-migration
```

---

## ✅ Final Validation Checklist

### Functionality
- [ ] Dev server starts (`bun run dev`)
- [ ] Tauri app launches (`bun run tauri`)
- [ ] Production build works (`bun run build`)
- [ ] Full build creates installer (`bun run tauri:build`)
- [ ] All tests pass (`bun test`)
- [ ] Type checking passes (`bun run check`)
- [ ] Linting passes (`bun run lint`)
- [ ] Formatting works (`bun run format`)

### Code Quality
- [ ] No Vitest imports remain
- [ ] All test files use `bun:test`
- [ ] No `node_modules` in repository
- [ ] No `package-lock.json` in repository
- [ ] `bun.lockb` is present
- [ ] TypeScript compiles without errors
- [ ] No runtime errors

### Documentation
- [ ] README.md updated
- [ ] GETTING_STARTED.md updated
- [ ] Migration notes created
- [ ] All Bun commands documented
- [ ] Rollback plan documented

### Performance
- [ ] Installation speed measured
- [ ] Build speed measured
- [ ] Test speed measured
- [ ] Results documented

### Git
- [ ] All changes committed
- [ ] Migration tagged
- [ ] Pushed to GitHub
- [ ] Baseline tag preserved

---

## 🚨 Troubleshooting Guide

### Issue: Bun not found
**Solution**:
```powershell
# Reinstall Bun
irm bun.sh/install.ps1 | iex

# Restart PowerShell
# Verify installation
bun --version
```

---

### Issue: TypeScript errors after migration
**Solution**:
```powershell
# Add Bun types
cd ui-tauri
bun add -d bun-types

# Verify tsconfig.json includes "bun-types" in types array
```

---

### Issue: Import errors in test files
**Solution**:
- Ensure all imports use `'bun:test'` not `'vitest'`
- Replace `it()` with `test()`
- Replace `vi` with `mock`

---

### Issue: Tauri build fails
**Solution**:
```powershell
# Verify Vite build works first
cd ui-tauri
bun run build

# Check dist folder created
Get-ChildItem dist

# Then try Tauri build
bun run tauri:build
```

---

### Issue: Tests fail after migration
**Solution**:
1. Check test file imports (should be `bun:test`)
2. Verify all `it()` changed to `test()`
3. Check for Vitest-specific features (may need adjustment)
4. Run tests with verbose flag: `bun test --verbose`

---

### Issue: Performance not improved
**Check**:
1. Verify using Bun (not Node.js): `bun --version`
2. Check scripts use `bunx --bun` prefix
3. Ensure no npm processes running
4. Clear cache: `bun pm cache rm`

---

## 🔄 Rollback Procedure

If critical issues arise:

### Step 1: Checkout Baseline
```powershell
git checkout v1.0.0-nodejs-baseline
```

### Step 2: Reinstall with npm
```powershell
cd ui-tauri
npm install
```

### Step 3: Verify Functionality
```powershell
npm run tauri
```

### Step 4: Document Issues
Create GitHub issue with:
- What failed
- Error messages
- Steps to reproduce
- Bun version
- System info

---

## 📊 Success Metrics

Track these metrics before and after:

| Metric | Before (npm) | After (Bun) | Improvement |
|--------|--------------|-------------|-------------|
| Install time | ___ ms | ___ ms | ___% |
| Build time | ___ ms | ___ ms | ___% |
| Test time | ___ ms | ___ ms | ___% |
| Hot reload | ___ ms | ___ ms | ___% |
| Dev server start | ___ ms | ___ ms | ___% |

---

## 🎯 Expected Final State

After successful migration:

### File Structure
```
ui-tauri/
├── bun.lockb          ← NEW (replaces package-lock.json)
├── bunfig.toml        ← NEW (optional)
├── package.json       ← MODIFIED (Bun scripts)
├── tsconfig.json      ← MODIFIED (Bun types)
├── vite.config.ts     ← UNCHANGED
├── src/
│   ├── **.test.ts     ← MODIFIED (Bun test imports)
│   └── ...            ← UNCHANGED
└── src-tauri/
    └── tauri.conf.json ← MODIFIED (Bun commands)
```

### Commands
- ✅ `bun install` - Install dependencies
- ✅ `bun run dev` - Start dev server
- ✅ `bun run tauri` - Launch Tauri dev
- ✅ `bun run build` - Build production
- ✅ `bun run tauri:build` - Build Tauri app
- ✅ `bun test` - Run tests
- ✅ `bun run lint` - Lint code
- ✅ `bun run format` - Format code

### Performance
- 📈 50-70% faster installation
- 📈 10-30% faster builds
- 📈 2-3x faster tests
- 📈 Better developer experience

---

## 🏁 Completion Certificate

When all phases complete successfully:

```
✅ MIGRATION COMPLETE

Project: Virtual IP Browser
Migration: Node.js/npm → Bun
Date: ___________
Duration: ___________
Bun Version: ___________

Results:
- All tests passing
- All features working
- Documentation updated
- Performance improved
- Code committed and pushed

Status: PRODUCTION READY
```

---

## 📞 Support Contacts

If you encounter issues:

1. **Check documentation**: `MIGRATION_PLAN_NODEJS_TO_BUN.md`
2. **Bun Discord**: https://bun.sh/discord
3. **GitHub Issues**: Create issue in repository
4. **Stack Overflow**: Tag with `bun`, `tauri`, `svelte`

---

## 🎓 Learning Resources

For team members learning Bun:

1. **Official Docs**: https://bun.sh/docs
2. **Bun vs Node**: https://bun.sh/docs/runtime/nodejs-apis
3. **Bun Test**: https://bun.sh/docs/cli/test
4. **Tauri + Bun**: https://tauri.app/v1/guides/

---

**END OF COPILOT AGENT INSTRUCTIONS**

---

*This guide is designed for step-by-step execution by GitHub Copilot or manual execution by developers. Follow phases sequentially and validate after each step.*
