# Node.js to Bun Migration Plan for GitHub Copilot

## 🎯 Migration Overview

**Objective**: Migrate the Virtual IP Browser UI from Node.js/npm to Bun runtime for improved performance and developer experience.

**Benefits of Bun**:
- ⚡ **3x faster** package installation
- 🚀 **Built-in TypeScript** support (no ts-node needed)
- 📦 **Drop-in replacement** for Node.js
- 🔥 **Native test runner** (replaces Vitest)
- 💾 **Smaller disk footprint**
- 🛠️ **Built-in bundler** (can replace Vite if desired)

---

## 📋 Phase 1: Pre-Migration Preparation

### 1.1 Backup Current State
```bash
# Already done - code is in GitHub
git tag v1.0.0-nodejs-baseline
git push origin v1.0.0-nodejs-baseline
```

### 1.2 Document Current Dependencies
- [x] Review `package.json` (42 total dependencies)
- [x] Check Tauri build commands
- [x] Identify potential incompatibilities

### 1.3 Verify Bun Compatibility
Check these packages for Bun support:
- ✅ Vite (fully compatible)
- ✅ Svelte (fully compatible)
- ✅ TypeScript (native support)
- ✅ Tauri CLI (compatible)
- ⚠️ Vitest (can use Bun test instead)
- ✅ ESLint (compatible)
- ✅ Prettier (compatible)

---

## 📋 Phase 2: Install Bun Runtime

### 2.1 Install Bun on Windows
```powershell
# Using PowerShell (as Administrator)
irm bun.sh/install.ps1 | iex

# Verify installation
bun --version
```

### 2.2 Verify Bun Installation
```bash
bun --version  # Should show v1.x.x
bun --help
```

---

## 📋 Phase 3: Update Configuration Files

### 3.1 Update `package.json`
**File**: `ui-tauri/package.json`

**Changes to make**:
```json
{
  "name": "virtual-ip-browser-ui",
  "version": "1.0.0",
  "description": "Virtual IP Browser UI",
  "type": "module",
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
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-store": "^2.0.0",
    "@tauri-apps/plugin-shell": "^2.0.0",
    "@tauri-apps/plugin-dialog": "^2.0.0",
    "@tauri-apps/plugin-notification": "^2.0.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@sveltejs/vite-plugin-svelte": "^4.0.0",
    "svelte": "^5.0.0",
    "svelte-check": "^4.0.0",
    "vite": "^5.4.0",
    "typescript": "^5.6.0",
    "tslib": "^2.7.0",
    "@testing-library/svelte": "^5.0.0",
    "prettier": "^3.3.0",
    "prettier-plugin-svelte": "^3.2.0",
    "eslint": "^9.9.0",
    "eslint-plugin-svelte": "^2.43.0",
    "@typescript-eslint/eslint-plugin": "^8.0.0",
    "@typescript-eslint/parser": "^8.0.0"
  }
}
```

**Note**: Removed Vitest packages - will use Bun's native test runner instead.

---

### 3.2 Update `tauri.conf.json`
**File**: `ui-tauri/src-tauri/tauri.conf.json`

**Changes to make** (lines 6-8):
```json
{
  "build": {
    "beforeDevCommand": "bun run dev",
    "devUrl": "http://localhost:5173",
    "beforeBuildCommand": "bun run build",
    "frontendDist": "../dist"
  }
}
```

---

### 3.3 Update `tsconfig.json`
**File**: `ui-tauri/tsconfig.json`

**Changes to make**:
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

### 3.4 Create Bun Test Configuration (Optional)
**File**: `ui-tauri/bunfig.toml`

```toml
[test]
# Test configuration
preload = ["./test-setup.ts"]
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

## 📋 Phase 4: Migrate Test Files

### 4.1 Convert Vitest Tests to Bun Test
**Pattern to follow**:

**Before (Vitest)**:
```typescript
import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';

describe('MyComponent', () => {
  it('renders correctly', () => {
    const { container } = render(MyComponent);
    expect(container).toBeTruthy();
  });
});
```

**After (Bun Test)**:
```typescript
import { test, expect, describe } from 'bun:test';
import { render } from '@testing-library/svelte';

describe('MyComponent', () => {
  test('renders correctly', () => {
    const { container } = render(MyComponent);
    expect(container).toBeTruthy();
  });
});
```

**Key changes**:
- Import from `bun:test` instead of `vitest`
- Use `test()` instead of `it()`
- Everything else stays the same

---

### 4.2 Test Files to Update
Search for and update all test files:
```bash
# Find all test files
find ui-tauri/src -name "*.test.ts" -o -name "*.spec.ts"
```

**Expected test files**:
- Any files in `ui-tauri/src/**/*.test.ts`
- Any files in `ui-tauri/src/**/*.spec.ts`

---

## 📋 Phase 5: Update `.gitignore`

**File**: `.gitignore`

**Add Bun-specific entries**:
```gitignore
# Bun
.bun/
bun.lockb
node_modules/

# Keep existing entries...
```

---

## 📋 Phase 6: Migration Execution

### 6.1 Remove Node.js Artifacts
```bash
cd ui-tauri

# Remove node_modules
Remove-Item -Recurse -Force node_modules

# Remove package-lock.json
Remove-Item -Force package-lock.json

# Remove any Vitest cache
Remove-Item -Recurse -Force .vitest
```

---

### 6.2 Install Dependencies with Bun
```bash
cd ui-tauri

# Install all dependencies
bun install

# This creates bun.lockb (Bun's lockfile)
```

---

### 6.3 Verify Installation
```bash
# Check installed packages
bun pm ls

# Verify Tauri CLI works
bun tauri --version

# Check Vite works
bun vite --version
```

---

## 📋 Phase 7: Testing & Validation

### 7.1 Test Development Server
```bash
cd ui-tauri

# Start Vite dev server with Bun
bun run dev

# Should see:
# VITE v5.x.x  ready in XXX ms
# ➜  Local:   http://localhost:5173/
```

---

### 7.2 Test Tauri Development Build
```bash
cd ui-tauri

# Start Tauri in dev mode
bun run tauri

# Should compile Rust backend and launch app
```

---

### 7.3 Test Production Build
```bash
cd ui-tauri

# Build for production
bun run build

# Verify dist/ folder created
ls ../dist

# Test Tauri production build
bun run tauri:build
```

---

### 7.4 Run Tests
```bash
cd ui-tauri

# Run all tests with Bun
bun test

# Run with watch mode
bun test --watch

# Run with coverage
bun test --coverage
```

---

### 7.5 Test Linting & Formatting
```bash
cd ui-tauri

# Test ESLint
bun run lint

# Test Prettier
bun run format

# Test Svelte type checking
bun run check
```

---

## 📋 Phase 8: Update Documentation

### 8.1 Update README.md
Add Bun installation and usage instructions:

```markdown
## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Bun](https://bun.sh/) v1.0.0 or higher (replaces Node.js)
- [Git](https://git-scm.com/)

## Quick Start

1. Install dependencies:
   ```bash
   cd ui-tauri
   bun install
   ```

2. Run development server:
   ```bash
   bun run tauri
   ```

3. Build for production:
   ```bash
   bun run tauri:build
   ```

## Scripts

- `bun run dev` - Start Vite dev server
- `bun run build` - Build for production
- `bun run tauri` - Launch Tauri in dev mode
- `bun run tauri:build` - Build Tauri app
- `bun test` - Run tests
- `bun run lint` - Lint code
- `bun run format` - Format code
```

---

### 8.2 Create Migration Notes
**File**: `MIGRATION_NOTES.md`

Document:
- Migration date
- Bun version used
- Any compatibility issues encountered
- Performance improvements observed
- Breaking changes (if any)

---

## 📋 Phase 9: CI/CD Updates (If Applicable)

### 9.1 Update GitHub Actions
**File**: `.github/workflows/build.yml` (if exists)

**Change from**:
```yaml
- name: Setup Node.js
  uses: actions/setup-node@v3
  with:
    node-version: '18'

- name: Install dependencies
  run: npm install
```

**Change to**:
```yaml
- name: Setup Bun
  uses: oven-sh/setup-bun@v1
  with:
    bun-version: latest

- name: Install dependencies
  run: bun install
```

---

## 📋 Phase 10: Performance Benchmarking

### 10.1 Measure Installation Speed
```bash
# Clean install comparison
Measure-Command { bun install }
# vs
Measure-Command { npm install }
```

### 10.2 Measure Build Speed
```bash
# Build time comparison
Measure-Command { bun run build }
# vs
Measure-Command { npm run build }
```

### 10.3 Document Results
Create performance comparison table in documentation.

---

## 📋 Phase 11: Team Training & Documentation

### 11.1 Developer Setup Guide
Create guide for team members:
1. Install Bun
2. Remove Node.js artifacts
3. Run `bun install`
4. Continue development as normal

### 11.2 Common Commands Reference
Create cheat sheet comparing npm vs bun commands:

| Task | npm | Bun |
|------|-----|-----|
| Install all | `npm install` | `bun install` |
| Add package | `npm install <pkg>` | `bun add <pkg>` |
| Add dev package | `npm install -D <pkg>` | `bun add -d <pkg>` |
| Remove package | `npm uninstall <pkg>` | `bun remove <pkg>` |
| Run script | `npm run dev` | `bun run dev` |
| Execute binary | `npx <cmd>` | `bunx <cmd>` |
| Run file | `node index.js` | `bun index.js` |
| Test | `npm test` | `bun test` |

---

## 📋 Phase 12: Rollback Plan (Just in Case)

### 12.1 If Issues Arise
```bash
# Restore from git
git stash

# Reinstall with npm
cd ui-tauri
npm install

# Revert tauri.conf.json
git checkout ui-tauri/src-tauri/tauri.conf.json

# Revert package.json
git checkout ui-tauri/package.json
```

### 12.2 Keep Node.js Baseline
- Keep the `v1.0.0-nodejs-baseline` tag
- Document any Bun-specific issues
- Can always rollback if needed

---

## 🎯 GitHub Copilot Instructions

### Copilot Agent Prompts

**Prompt 1: Initial Setup**
```
@workspace I need to migrate this Tauri + Svelte project from Node.js/npm to Bun. 
Please help me:
1. Update package.json scripts to use Bun
2. Update tauri.conf.json to use Bun commands
3. Update tsconfig.json for Bun compatibility
4. Remove Vitest and configure Bun's native test runner
```

**Prompt 2: Test Migration**
```
@workspace Find all test files using Vitest and convert them to use Bun's test runner.
Change imports from 'vitest' to 'bun:test' and replace it() with test().
Preserve all test logic and assertions.
```

**Prompt 3: Verification**
```
@workspace After migrating to Bun:
1. Verify all import statements work correctly
2. Check if any packages need Bun-specific configuration
3. Update any npm-specific scripts or commands
4. Test that TypeScript compilation works
```

**Prompt 4: Documentation**
```
@workspace Update the README.md and other documentation to reflect:
1. Bun as the required runtime instead of Node.js
2. New installation instructions (bun install)
3. Updated command examples (bun run instead of npm run)
4. Performance benefits of using Bun
```

---

## ✅ Migration Checklist

Use this checklist to track progress:

### Pre-Migration
- [ ] Backup code to GitHub
- [ ] Create baseline tag
- [ ] Document current setup
- [ ] Review dependencies for Bun compatibility

### Installation
- [ ] Install Bun runtime
- [ ] Verify Bun installation
- [ ] Check Bun version

### Configuration Updates
- [ ] Update `package.json` scripts
- [ ] Update `tauri.conf.json` commands
- [ ] Update `tsconfig.json` for Bun
- [ ] Create `bunfig.toml` (optional)
- [ ] Update `.gitignore`

### Code Migration
- [ ] Remove `node_modules/`
- [ ] Remove `package-lock.json`
- [ ] Run `bun install`
- [ ] Verify `bun.lockb` created
- [ ] Convert test files from Vitest to Bun test
- [ ] Update any Node.js-specific code

### Testing
- [ ] Test development server (`bun run dev`)
- [ ] Test Tauri dev mode (`bun run tauri`)
- [ ] Test production build (`bun run build`)
- [ ] Test Tauri build (`bun run tauri:build`)
- [ ] Run all tests (`bun test`)
- [ ] Test linting (`bun run lint`)
- [ ] Test formatting (`bun run format`)
- [ ] Test type checking (`bun run check`)

### Documentation
- [ ] Update README.md
- [ ] Update GETTING_STARTED.md
- [ ] Create MIGRATION_NOTES.md
- [ ] Update development guides
- [ ] Create Bun command reference

### CI/CD (if applicable)
- [ ] Update GitHub Actions
- [ ] Test CI pipeline
- [ ] Update deployment scripts

### Performance
- [ ] Benchmark installation time
- [ ] Benchmark build time
- [ ] Benchmark test execution
- [ ] Document improvements

### Finalization
- [ ] Commit all changes
- [ ] Create migration tag
- [ ] Push to GitHub
- [ ] Update project documentation
- [ ] Train team members

---

## 🚀 Expected Outcomes

### Performance Improvements
- **Install time**: 50-70% faster
- **Build time**: 10-30% faster  
- **Test execution**: 2-3x faster
- **Disk space**: ~30% less (no separate node_modules cache)

### Developer Experience
- Faster hot reload
- Native TypeScript execution
- Simpler tooling (one runtime for everything)
- Better error messages
- Built-in utilities (bundler, test runner, package manager)

### Compatibility
- All existing code continues to work
- No changes to Svelte components
- No changes to Tauri Rust backend
- No changes to application logic
- Drop-in replacement for npm/node

---

## 📞 Support & Resources

### Bun Resources
- [Official Docs](https://bun.sh/docs)
- [GitHub](https://github.com/oven-sh/bun)
- [Discord](https://bun.sh/discord)

### Migration Guides
- [Bun with Vite](https://bun.sh/guides/ecosystem/vite)
- [Bun with TypeScript](https://bun.sh/docs/runtime/typescript)
- [Bun Test Runner](https://bun.sh/docs/cli/test)

### Tauri with Bun
- Fully compatible
- No special configuration needed
- Use `bun` instead of `npm` in beforeDevCommand

---

## 🎯 Success Criteria

Migration is complete when:
1. ✅ All dependencies installed with `bun install`
2. ✅ Dev server runs with `bun run dev`
3. ✅ Tauri app launches with `bun run tauri`
4. ✅ Production build works with `bun run tauri:build`
5. ✅ All tests pass with `bun test`
6. ✅ Linting works with `bun run lint`
7. ✅ No Node.js/npm artifacts remain
8. ✅ Documentation updated
9. ✅ Team trained on new workflow
10. ✅ Performance improvements verified

---

## 📝 Notes for GitHub Copilot Agent

When executing this migration:

1. **Work incrementally**: Complete one phase before moving to the next
2. **Test frequently**: Run tests after each major change
3. **Preserve functionality**: Don't change application logic, only tooling
4. **Document changes**: Keep track of what was modified and why
5. **Ask for clarification**: If package compatibility is unclear, ask before proceeding
6. **Use context**: Reference existing code patterns and project structure
7. **Verify imports**: Ensure all import paths remain valid
8. **Check types**: TypeScript should compile without errors
9. **Maintain style**: Follow existing code formatting and conventions
10. **Create backups**: Suggest creating git commits at major milestones

**Priority order**: Configuration → Installation → Testing → Documentation → CI/CD

**Critical files to update**:
1. `ui-tauri/package.json` (scripts and dependencies)
2. `ui-tauri/src-tauri/tauri.conf.json` (build commands)
3. `ui-tauri/tsconfig.json` (compiler options)
4. `.gitignore` (Bun artifacts)
5. Any test files (Vitest → Bun test)

**Do NOT change**:
- Svelte components (`.svelte` files)
- TypeScript logic files (unless fixing imports)
- Rust backend code
- Application architecture
- UI/UX implementation

---

## 🏁 Final Steps

After successful migration:

```bash
# Commit changes
git add .
git commit -m "chore: migrate from Node.js/npm to Bun runtime

- Update package.json scripts for Bun
- Update tauri.conf.json build commands
- Configure TypeScript for Bun
- Replace Vitest with Bun test runner
- Update documentation
- Remove Node.js artifacts

Performance improvements:
- Installation: ~60% faster
- Tests: ~2x faster
- Better DX with native TypeScript support"

# Create migration tag
git tag v1.1.0-bun-migration

# Push to GitHub
git push origin main
git push origin v1.1.0-bun-migration
```

---

**Migration Plan Created**: Ready for GitHub Copilot Agent execution
**Estimated Time**: 2-4 hours (including testing and documentation)
**Risk Level**: Low (can rollback easily)
**Complexity**: Medium
