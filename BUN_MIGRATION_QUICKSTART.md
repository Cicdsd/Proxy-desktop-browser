# Bun Migration - Quick Start Guide

> **TL;DR**: Replace Node.js with Bun for 2-3x faster development workflow

---

## 🚀 Quick Migration (5 Steps)

### 1. Install Bun
```powershell
irm bun.sh/install.ps1 | iex
bun --version
```

### 2. Update Configuration Files

**`ui-tauri/package.json`** - Change scripts:
```json
"scripts": {
  "dev": "bunx --bun vite",
  "build": "bunx --bun vite build",
  "tauri": "bun tauri dev",
  "test": "bun test"
}
```

**`ui-tauri/src-tauri/tauri.conf.json`** - Change build commands:
```json
"beforeDevCommand": "bun run dev",
"beforeBuildCommand": "bun run build"
```

**`ui-tauri/tsconfig.json`** - Add Bun types:
```json
"types": ["svelte", "bun-types"]
```

### 3. Clean and Install
```powershell
cd ui-tauri
Remove-Item -Recurse -Force node_modules, package-lock.json
bun install
```

### 4. Convert Tests (if any)
```typescript
// Change FROM:
import { describe, it, expect } from 'vitest';

// Change TO:
import { describe, test, expect } from 'bun:test';

// Replace it() with test()
```

### 5. Verify
```powershell
bun run dev       # Should start dev server
bun run tauri     # Should launch app
bun test          # Should run tests
```

---

## 📊 Quick Reference

| Task | npm | Bun |
|------|-----|-----|
| Install all | `npm install` | `bun install` |
| Add package | `npm install pkg` | `bun add pkg` |
| Add dev package | `npm install -D pkg` | `bun add -d pkg` |
| Remove package | `npm uninstall pkg` | `bun remove pkg` |
| Run script | `npm run dev` | `bun run dev` |
| Execute binary | `npx cmd` | `bunx cmd` |
| Run file | `node file.js` | `bun file.js` |
| Test | `npm test` | `bun test` |

---

## ✅ Validation Checklist

- [ ] Bun installed (`bun --version`)
- [ ] `package.json` scripts updated
- [ ] `tauri.conf.json` commands updated
- [ ] `tsconfig.json` has bun-types
- [ ] `node_modules` removed
- [ ] `package-lock.json` removed
- [ ] `bun install` completed
- [ ] `bun.lockb` created
- [ ] Dev server works (`bun run dev`)
- [ ] Tauri works (`bun run tauri`)
- [ ] Tests pass (`bun test`)

---

## 🔄 Rollback

If issues occur:
```powershell
git checkout v1.0.0-nodejs-baseline
cd ui-tauri
npm install
npm run tauri
```

---

## 📚 Full Guides

- **Detailed Plan**: `MIGRATION_PLAN_NODEJS_TO_BUN.md`
- **Copilot Instructions**: `COPILOT_AGENT_INSTRUCTIONS.md`
- **Bun Docs**: https://bun.sh/docs

---

## 🎯 Expected Benefits

- ⚡ **60%** faster `npm install` → `bun install`
- 🚀 **2-3x** faster test execution
- 📦 **30%** smaller disk usage
- ✨ Native TypeScript support (no transpilation needed)
- 🔥 Faster hot reload in development

---

## ⚠️ Important Notes

**What Changes**:
- Package manager (npm → Bun)
- Test runner (Vitest → Bun test)
- Runtime (Node.js → Bun)

**What Stays Same**:
- All Svelte components
- All TypeScript code
- All application logic
- Vite bundler
- Tauri backend
- File structure

---

## 🆘 Common Issues

### "bun: command not found"
```powershell
# Restart PowerShell after installation
# Or add to PATH manually
```

### TypeScript errors
```powershell
bun add -d bun-types
# Verify tsconfig.json includes "bun-types"
```

### Tests fail
```typescript
// Ensure imports use 'bun:test' not 'vitest'
import { test, expect } from 'bun:test';
```

### Build fails
```powershell
# Test Vite build first
bun run build
# Then test Tauri
bun run tauri:build
```

---

## 🎉 Success Indicator

When you see this, migration is complete:
```
✅ bun.lockb exists
✅ node_modules reinstalled with Bun
✅ bun run dev works
✅ bun run tauri launches app
✅ bun test passes
✅ No errors in console
```

---

**Ready to migrate? Start with: `irm bun.sh/install.ps1 | iex`**
