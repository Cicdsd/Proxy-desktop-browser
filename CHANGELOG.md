# Changelog

All notable changes to the Proxy-Desktop-Browser project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Proxy Codebase Understanding Documentation** (`PROXY_CODEBASE_UNDERSTANDING.md`)
  - Comprehensive proxy system architecture documentation
  - Core components reference (ProxyManager, ProxyRotationManager, FreeIpProviderManager)
  - Proxy types and rotation strategies explained
  - Data flow diagrams for request and validation flows
  - API reference for frontend and Tauri commands
  - Best practices for proxy selection, error handling, and security
  - Complete file reference for all proxy-related modules
- **Code Quality Improvements**
  - Fixed long lines in `lib.rs` by splitting `pub use` statements
  - Fixed long function signature in `automation.rs` (record_action)
  - Reduced complexity in `chromium_engine.rs` (apply_fingerprint_spoofing: 25 → <10)
  - Reduced complexity in `storage.rs` (import_with_options: 21 → <10)
  - No functions with complexity > 20 remaining
  - All modified Rust files pass `rustfmt --check` validation
  - Updated code quality metrics in IMPROVEMENTS.md

- **Import/Export Features for In-Memory Storage**
  - `export_all()` - Export all storage data to StorageExport struct
  - `export_to_file()` - Export storage to JSON file
  - `export_to_json()` - Export storage to JSON string
  - `import_all()` - Import all storage data from StorageExport struct
  - `import_from_file()` - Import storage from JSON file
  - `import_from_json()` - Import storage from JSON string
  - Selective export/import for cookies, history, bookmarks, local storage
  - Merge or replace options for imports
  - `ImportOptions` and `ExportOptions` structs for fine-grained control
  - `ImportExportStats` for tracking import/export results
  - `get_stats()` - Get current storage statistics
  - `clear_all()` - Clear all storage data
  - Comprehensive unit tests for import/export functionality

### Removed
- Cleaned up all commented database-related code (sqlx, sqlite, rusqlite references)
- Removed legacy database module comments from source files

### Added
- **API Settings Panel** for proxy provider configuration
  - Support for IPRoyal, Bright Data, Oxylabs, Smartproxy, Webshare, and custom providers
  - Default IPRoyal configuration with pre-configured API token
  - Proxy generation with country, session type, and protocol options
  - Connection testing and status monitoring
  - "Use Proxy" button to activate generated proxy in the browser
  - "Copy Proxy" button to copy proxy URL to clipboard
  - Provider enable/disable toggle
  - Persistent storage of provider configurations

- **Proxy Provider Types** in types.ts
  - `ProxyProviderType` - Supported provider types
  - `ProxyProviderConfig` - Provider configuration schema
  - `ProxyProviderSettings` - Proxy generation settings
  - `IPRoyalProxyResponse` - IPRoyal proxy response type
  - `ProxyProviderStatus` - Provider connection status

- **Proxy Provider API Functions** in api.ts
  - `getProxyFromProvider()` - Generate proxy from configured provider
  - `testProxyProvider()` - Test provider connection
  - `listProxyProviders()` - List all configured providers
  - `saveProxyProvider()` - Save provider configuration
  - `deleteProxyProvider()` - Delete provider configuration

- **Database Removed** - Switched to in-memory storage
  - Removed SQLite/sqlx dependency completely
  - TabIPManager now uses in-memory HashMap storage
  - StorageEngine converted to in-memory storage for cookies, history, bookmarks
  - BrowserTabManager simplified without database requirements
  - Faster startup and no database file management needed

- **Makefile** for project build automation


  - `make build` - Build entire project
  - `make clean` - Clean all build artifacts
  - `make test` - Run all tests
  - `make lint` - Run linters
  - `make format` - Format code
  - `make help` - Show available targets

- **Tauri Event Types** in types.ts
  - `NavigationChangedPayload` for navigation events
  - `TitleChangedPayload` for title change events
  - `TauriEvent<T>` generic event wrapper

- **fetchUsers API function** for user management

### Changed
- **Complete TypeScript Type Safety Overhaul**
  - Eliminated ALL `any` type usages (15 → 0)
  - Added Svelte 5 generics to VirtualList and AsyncWrapper components
  - Improved error handling with `unknown` type and proper type guards

### Fixed
- **Password Verification** in auth.rs
  - Added `password_hashes` storage to AuthManager
  - Implemented proper Argon2 password verification
  - Fixed enterprise user creation to store password hashes

- **Network Throttling** in chromium_engine.rs
  - Implemented `apply_network_throttling` with proper bandwidth conversion
  - Added logging for throttling configuration

- **TODO Comments Addressed** (6 → 1 remaining)
  - Implemented fetchUsers API endpoint
  - Implemented password verification
  - Improved documentation for webview proxy limitations
  - Added proper tab cleanup logging

## [1.0.0] - 2025-12-21

### Added

- **Structured Logger Utility** (`ui-tauri/src/lib/logger.ts`)
  - Proper logging system with log levels (DEBUG, INFO, WARN, ERROR)
  - Timestamps, context support, and log history
  - Auto-adjusts log level based on development/production mode

- **Type Declarations** (`ui-tauri/src/lib/tauri.d.ts`)
  - Proper Window interface extension for Tauri metadata
  - Eliminates need for `window as any` casts

- **New TypeScript Types** (`ui-tauri/src/lib/types.ts`)
  - `EnterpriseUserData`: Proper type for enterprise user creation
  - `AuthResponse`: Standardized auth response type
  - `SuccessResponse`: Generic success response type

- **Prelude Module** (`crates/browser-core/src/prelude.rs`)
  - Common imports for consistent error handling
  - `OptionExt` trait for better Option-to-Result conversion
  - `ResultExt` trait for adding context to errors
  - `unix_timestamp()` and `unix_timestamp_ms()` helper functions

- **V1000 Experimental Features**
  - Automation modules for advanced browser control
  - Content enhancement modules
  - V1000 upgrade modules with experimental features

- **Comprehensive Documentation**
  - Architecture diagrams
  - Development phases documentation
  - Implementation checklist
  - Quick start guide
  - Tech stack analysis and recommendations

### Changed
- **Migrated to Tauri v2.0** from Tauri v1.x
  - Updated all Tauri dependencies to 2.0
  - Added new Tauri plugins: store, shell, dialog, notification, updater

- **Chromium Engine Enhanced to v1000**
  - Advanced features and bug fixes
  - Better tab management with proper validation
  - Fixed CDP methods with proper tab_id validation

- **Logging Improvements**
  - Replaced `console.log`, `console.error` with structured logger
  - Replaced `println!`/`eprintln!` with tracing macros in Rust
  - Files updated: `errorHandling.ts`, `api.ts`, `connectionPool.ts`, `MainApp.svelte`, `main.rs`

- **TypeScript Type Safety Improvements**
  - Changed `error: any` to `error: unknown` (safer error handling)
  - Changed `args?: any` to `args?: Record<string, unknown>`
  - Changed `userData: any` to `userData: EnterpriseUserData`
  - Changed Promise callbacks to use `unknown` type

- **Node.js to Bun Migration**
  - Migrated package management from npm to Bun
  - Updated build scripts to use `bunx --bun`
  - Added `bunfig.toml` configuration

### Fixed
- **Critical Bug Fixes**
  - Fixed tab_id bug in CDP methods with proper validation
  - Fixed division by zero issues
  - Fixed canvas RGBA overflow
  - Fixed geolocation handling
  - Fixed compilation errors: added Hash/Eq derives
  - Fixed borrow checker issues

- **Security Issues**
  - Addressed security issues from PR #9 review
  - Refined imports to be more specific per code review feedback

- **Code Quality**
  - Removed unnecessary `unwrap()` calls in `free_ip_providers.rs`
  - Improved code formatting in `BrowserShell.svelte`
  - Fixed typos: 'devloper' to 'developer'

### Removed
- Deleted obsolete documentation files:
  - `docs_formatted.md`
  - `docs_raw.txt`
  - `docs_with_breaks.txt`
  - `docs.md`

### Security
- Added security-focused dependencies:
  - `keyring` for secure credential storage
  - `validator` for input validation
  - `ammonia` for HTML sanitization
  - `governor` for rate limiting
  - `aes-gcm` for encryption
  - `argon2` for password hashing

## [0.1.0] - 2025-12-14

### Added
- **Initial Project Setup**
  - Virtual IP Browser project with Rust backend
  - Tauri UI framework integration
  - Workspace structure with multiple crates:
    - `virtual-ip`: Virtual IP management
    - `browser-core`: Core browser functionality
    - `api-server`: API server implementation

- **Core Dependencies**
  - Async runtime with Tokio
  - HTTP client with reqwest (socks proxy, rustls-tls, cookies support)
  - SQLx for async database operations
  - Web scraping with scraper and select

- **UI Framework**
  - Svelte 5.0 with TypeScript
  - Vite 5.4 for build tooling
  - Tauri API integration

- **Documentation**
  - Comprehensive migration plans (Node.js to Bun)
  - Refactoring index
  - Agent documentation and instructions

---

## Version History Summary

| Version | Date       | Highlights                                    |
|---------|------------|-----------------------------------------------|
| 1.0.0   | 2025-12-21 | Tauri v2.0, V1000 features, security updates |
| 0.1.0   | 2025-12-14 | Initial release with Rust/Tauri architecture |

## Code Quality Metrics (v1.0.0)

| Metric              | Before | After | Improvement    |
|---------------------|--------|-------|----------------|
| console.log usage   | 10     | 0     | ✅ 100% fixed  |
| println!/eprintln!  | 2      | 0     | ✅ 100% fixed  |
| `any` type usage    | 26     | 15    | ✅ 42% reduced |
| unwrap() usage      | 101    | 100   | ✅ 1 fixed     |

## Contributors

- Virtual IP Browser Team

## Links

- [Repository](https://github.com/Cicdsd/Proxy-desktop-browser)
- [Issues](https://github.com/Cicdsd/Proxy-desktop-browser/issues)
- [Pull Requests](https://github.com/Cicdsd/Proxy-desktop-browser/pulls)
