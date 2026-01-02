# Developer Quick Reference

A quick reference guide for common development tasks in the Proxy-Desktop-Browser project.

## 🚀 Quick Commands

### Build & Run

| Task | Command |
|------|---------|
| Build project | `cargo build` |
| Build release | `cargo build --release` |
| Run dev mode | `cargo tauri dev` |
| Run tests | `cargo test` |
| Format code | `cargo fmt` |
| Lint code | `cargo clippy` |

### Frontend (ui-tauri)

| Task | Command |
|------|---------|
| Install deps | `cd ui-tauri && npm install` |
| Run dev | `npm run dev` |
| Build | `npm run build` |
| Lint | `npm run lint` |

## 📁 Key File Locations

### Rust Backend (crates/browser-core/src/)

| File | Purpose |
|------|---------|
| `lib.rs` | Library exports |
| `prelude.rs` | Common imports & utilities |
| `webview_manager.rs` | Tab/webview management |
| `proxy.rs` | Proxy configuration |
| `proxy_rotation.rs` | Proxy rotation strategies |
| `free_ip_providers.rs` | Free proxy providers |
| `chromium_engine.rs` | Browser engine integration |
| `storage.rs` | Data persistence |
| `privacy_fortress.rs` | Privacy protection |
| `config_manager.rs` | Configuration management |

### TypeScript Frontend (ui-tauri/src/)

| File | Purpose |
|------|---------|
| `lib/api.ts` | API functions |
| `lib/types.ts` | Type definitions |
| `lib/logger.ts` | Logging utility |
| `lib/stores.ts` | State management |
| `lib/utils.ts` | Utility functions |

## 🔧 Common Patterns

### Error Handling (Rust)

Use the prelude module and context for meaningful errors:
- Import with `use crate::prelude::*;`
- Use `.context("message")?` for error context
- Use `?` operator for propagation

### Logging

**Rust**: Use tracing macros (`info!`, `warn!`, `error!`, `debug!`)

**TypeScript**: Use logger functions (`logInfo`, `logError`, `logDebug`, `logWarn`)

## 📊 Proxy Types

| Type | URL Format |
|------|------------|
| HTTP | `http://host:port` |
| HTTPS | `https://host:port` |
| SOCKS4 | `socks4://host:port` |
| SOCKS5 | `socks5://host:port` |
| With Auth | `http://user:pass@host:port` |

## 🔄 Rotation Strategies

| Strategy | Use Case |
|----------|----------|
| PerRequest | Maximum anonymity |
| PerDuration | Time-based sessions |
| PerSession | Stable sessions |
| Random | Unpredictable rotation |
| Geographic | Geo-restricted content |
| PerformanceBased | Speed optimization |
| RoundRobin | Fair distribution |

## 🛠️ Debugging

- Enable debug logging: `RUST_LOG=debug cargo run`
- Enable trace logging: `RUST_LOG=browser_core=trace cargo run`
- Open DevTools: `cargo tauri dev --devtools`

## 📚 Documentation Links

- [Codebase Understanding](CODEBASE_UNDERSTANDING.md)
- [Coding Efficiency Guide](CODING_EFFICIENCY.md)
- [Proxy System Docs](PROXY_CODEBASE_UNDERSTANDING.md)
- [Implementation Checklist](IMPLEMENTATION_CHECKLIST.md)
- [Architecture Diagrams](ARCHITECTURE_DIAGRAMS.md)
- [Getting Started](GETTING_STARTED.md)

---

*Quick reference for Proxy-Desktop-Browser development*
