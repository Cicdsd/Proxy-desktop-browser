# GitHub Copilot Coding Agent - Deep Execution Plan
## Virtual IP Browser Project

---

## 📋 Table of Contents

1. [Project Overview](#project-overview)
2. [Architecture Understanding](#architecture-understanding)
3. [Development Phases](#development-phases)
4. [Agent Execution Patterns](#agent-execution-patterns)
5. [Code Generation Guidelines](#code-generation-guidelines)
6. [Testing Strategy](#testing-strategy)
7. [Security & Quality Standards](#security--quality-standards)
8. [Common Tasks & Solutions](#common-tasks--solutions)
9. [Troubleshooting Guide](#troubleshooting-guide)
10. [Reference Documentation](#reference-documentation)

---

## 🎯 Project Overview

### What We're Building
A **Desktop Browser Application** with advanced proxy and virtual IP management capabilities:
- Multi-tab browser with Chromium engine integration
- Per-tab proxy configuration and rotation
- Virtual IP generation and management
- Browser fingerprint protection
- Cookie and storage isolation per tab
- Free proxy provider integration
- Ad verification capabilities
- Comprehensive backup/restore system

### Tech Stack
- **Backend**: Rust + Tauri 2.0
- **Frontend**: Svelte + TypeScript + Vite
- **Database**: SQLite (via sqlx)
- **Browser Engine**: Chromium (via chromiumoxide)
- **HTTP Client**: reqwest with middleware
- **Async Runtime**: Tokio
- **Build Tool**: Bun (migrating from Node.js)

### Project Structure
```
proxy-desktop-browser/
├── crates/
│   ├── browser-core/      # Core browser logic (Rust)
│   ├── virtual-ip/        # IP generation & rotation
│   ├── api-server/        # Optional API server
├── ui-tauri/              # Tauri desktop app
│   ├── src-tauri/         # Rust backend
│   └── src/               # Svelte frontend
├── docs/                  # Comprehensive documentation
└── tests/                 # Integration tests
```

---

## 🏗️ Architecture Understanding

### Core Components

#### 1. Browser Core (`crates/browser-core/`)
**Purpose**: Central business logic for browser operations

**Key Modules**:
- `chromium_engine.rs` - Browser engine integration
- `browser_tab_manager.rs` - Tab lifecycle management
- `proxy.rs` - Proxy connection handling
- `proxy_rotation.rs` - Automatic proxy switching
- `proxy_validator.rs` - Proxy health checking
- `fingerprint.rs` - Browser fingerprint generation
- `tab_isolation.rs` - Per-tab network/storage isolation
- `storage.rs` - Cookie, history, bookmark management
- `database.rs` - SQLite operations
- `security.rs` - Input validation and sanitization
- `backup.rs` - Backup/restore functionality
- `free_ip_providers.rs` - Free proxy API integration
- `ad_verification.rs` - Ad impression verification

#### 2. Virtual IP (`crates/virtual-ip/`)
**Purpose**: IP address generation and rotation logic

**Key Modules**:
- `generator.rs` - Virtual IP generation algorithms
- `rotation.rs` - IP rotation strategies
- `validator.rs` - IP validation logic
- `models.rs` - Data structures

#### 3. Tauri Backend (`ui-tauri/src-tauri/`)
**Purpose**: Desktop app bridge between Rust and UI

**Key Files**:
- `main.rs` - Tauri commands and app initialization
- `auth.rs` - User authentication
- `webview_manager.rs` - WebView lifecycle

#### 4. Svelte Frontend (`ui-tauri/src/`)
**Purpose**: User interface

**Component Categories**:
- `components/browser/` - Browser UI (tabs, navigation)
- `components/config/` - Settings and configuration
- `components/auth/` - Authentication UI
- `components/layout/` - Layout components
- `components/ui/` - Reusable UI utilities

---

## 📅 Development Phases

### Current Implementation Status

#### ✅ **COMPLETED**
- Basic Rust workspace with 4 crates
- Tauri 2.0 desktop app foundation
- Svelte UI framework setup
- Database schema (SQLite with migrations)
- Virtual IP data models
- Proxy data structures
- Tab management framework
- UI component structure (auth, browser, config, layout)
- Error handling utilities
- Security manager skeleton
- Backup/restore system foundation

#### 🚧 **IN PROGRESS / PARTIAL**
- Chromium engine integration (structure exists, needs implementation)
- Proxy connection logic (models ready, connection logic partial)
- Browser tab lifecycle (manager exists, WebView integration needed)
- Free proxy providers (structure ready, API integrations needed)
- Ad verification system (models defined, verification logic needed)

#### ❌ **NOT STARTED / CRITICAL GAPS**
- **Network Traffic Interception**: Core feature - routing per-tab traffic through proxies
- **Actual Browser Rendering**: Chromium engine needs full integration
- **WebView Isolation**: Per-tab WebView instances with separate contexts
- **Proxy Rotation Engine**: Automatic switching based on health/strategy
- **Cookie/Storage Isolation**: Per-tab isolated storage implementation
- **PAC (Proxy Auto-Config)**: Dynamic proxy assignment
- **Local Proxy Server**: SOCKS5/HTTP proxy for routing
- **Free Proxy Scraping**: Active scraping from provider APIs
- **Fingerprint Randomization**: Active user agent/canvas/WebGL spoofing
- **End-to-End Testing**: Browser automation tests

---

### Phase-by-Phase Implementation Guide

#### **PHASE 1: Core Browser Engine** (PRIORITY: CRITICAL)
**Goal**: Get a working browser that can display web pages

**Tasks**:
1. **Complete Chromium Engine Integration**
   - File: `crates/browser-core/src/chromium_engine.rs`
   - Implement `ChromiumEngine::launch()` using chromiumoxide
   - Create browser instance with custom launch options
   - Implement `ChromiumTab` wrapper for tab operations
   - Add navigation methods: `goto()`, `reload()`, `back()`, `forward()`
   - Test: Open browser, navigate to URL, verify page loads

2. **WebView Manager Enhancement**
   - File: `ui-tauri/src-tauri/src/webview_manager.rs`
   - Remove TODO comment on line 43
   - Implement per-webview proxy configuration using Tauri WebView API
   - Create isolated WebView instances per tab
   - Link WebView lifecycle to tab lifecycle
   - Test: Create multiple tabs, verify isolation

3. **Tab Lifecycle Management**
   - File: `crates/browser-core/src/browser_tab_manager.rs`
   - Complete `create_tab()` implementation
   - Integrate with `ChromiumEngine` to spawn actual browser tabs
   - Implement `close_tab()`, `suspend_tab()`, `resume_tab()`
   - Add cleanup logic (resolve TODO on line 110 in tab_manager.rs)
   - Test: Create/close tabs, verify memory cleanup

4. **Tauri Commands for Browser Control**
   - File: `ui-tauri/src-tauri/src/main.rs`
   - Add commands: `navigate_tab`, `reload_tab`, `go_back`, `go_forward`
   - Expose tab lifecycle: `suspend_tab`, `resume_tab`
   - Add event emitters for tab state changes
   - Test: Call commands from frontend, verify browser responds

**Success Criteria**:
- ✅ Browser window opens with Chromium engine
- ✅ Can navigate to any URL
- ✅ Multiple tabs work independently
- ✅ Back/forward navigation works
- ✅ Tabs can be created/closed without crashes

---

#### **PHASE 2: Proxy & Virtual IP Integration** (PRIORITY: CRITICAL)
**Goal**: Route browser traffic through proxies on a per-tab basis

**Tasks**:
1. **Local Proxy Server**
   - File: `crates/browser-core/src/local_proxy.rs`
   - Implement SOCKS5 proxy server using tokio
   - Accept connections from browser tabs
   - Route traffic through configured upstream proxy
   - Add connection tracking per tab
   - Test: Connect browser through local proxy, verify traffic flows

2. **Proxy Connection Implementation**
   - File: `crates/browser-core/src/proxy.rs`
   - Complete `ProxyManager::connect()` method
   - Implement SOCKS5, HTTP, HTTPS proxy connections
   - Add authentication support (username/password)
   - Handle connection errors with retry logic
   - Test: Connect to known good proxies, verify IP changes

3. **PAC Server for Proxy Auto-Config**
   - File: `crates/browser-core/src/pac_server.rs`
   - Implement HTTP server serving PAC files
   - Generate PAC file dynamically per tab
   - Configure WebView to use PAC URL
   - Test: Verify browser uses correct proxy via PAC

4. **Per-Tab Proxy Assignment**
   - File: `crates/browser-core/src/browser_tab_manager.rs`
   - Integrate proxy assignment in `create_tab()`
   - Configure WebView proxy settings before navigation
   - Store proxy-tab mapping in database
   - Test: Each tab shows different IP address

5. **Virtual IP Generator Integration**
   - File: `crates/virtual-ip/src/generator.rs`
   - Implement IP generation algorithms (realistic ranges)
   - Integrate with proxy system for IP rotation
   - Test: Generated IPs are valid and diverse

**Success Criteria**:
- ✅ Browser traffic routes through proxy
- ✅ Each tab can use different proxy
- ✅ IP address verification shows proxy IP
- ✅ Proxy failures are handled gracefully
- ✅ No DNS leaks or WebRTC leaks

---

#### **PHASE 3: Proxy Rotation & Health Monitoring** (PRIORITY: HIGH)
**Goal**: Automatic proxy switching based on health and strategy

**Tasks**:
1. **Proxy Health Checker**
   - File: `crates/browser-core/src/proxy_validator.rs`
   - Implement `ProxyHealthChecker` background service
   - Test proxy connectivity, latency, anonymity level
   - Update proxy health metrics in database
   - Run health checks on interval (e.g., every 5 minutes)
   - Test: Dead proxies are detected and marked

2. **Proxy Rotation Engine**
   - File: `crates/browser-core/src/proxy_rotation.rs`
   - Implement rotation strategies: Round-robin, Random, Latency-based
   - Auto-switch proxies based on configurable triggers:
     - Time-based (every N minutes)
     - Request-based (every N requests)
     - Health-based (when proxy fails)
   - Add rotation to tab manager
   - Test: Proxies rotate automatically, tabs maintain connections

3. **Proxy Pool Management**
   - File: `crates/browser-core/src/proxy.rs`
   - Implement proxy pool with health-based ranking
   - Add/remove proxies dynamically
   - Blacklist failed proxies temporarily
   - Persist proxy metrics to database
   - Test: Pool maintains healthy proxies only

4. **Tauri Commands for Proxy Control**
   - File: `ui-tauri/src-tauri/src/main.rs`
   - Add commands: `rotate_proxy`, `get_proxy_health`, `add_proxy`, `remove_proxy`
   - Expose proxy pool status
   - Test: UI can control proxy rotation

**Success Criteria**:
- ✅ Proxies rotate automatically based on strategy
- ✅ Failed proxies are detected and skipped
- ✅ Health metrics are visible in UI
- ✅ Manual proxy rotation works
- ✅ Proxy pool is persisted across restarts

---

#### **PHASE 4: Free Proxy Provider Integration** (PRIORITY: HIGH)
**Goal**: Automatically fetch and integrate free proxies from providers

**Tasks**:
1. **Free Proxy Provider APIs**
   - File: `crates/browser-core/src/free_ip_providers.rs`
   - Implement scrapers for free proxy services:
     - ProxyScrape API
     - GeoNode API
     - ProxyList API
     - PubProxy API
   - Parse proxy lists (IP, port, protocol, country)
   - Add rate limiting for API requests
   - Test: Fetch proxies from each provider successfully

2. **Proxy Scraper Enhancement**
   - File: `crates/browser-core/src/scraper_util.rs`
   - Add HTML scraping for proxy websites
   - Parse proxy tables with scraper crate
   - Extract proxy details with regex
   - Test: Scrape proxies from public proxy sites

3. **Auto-Import & Validation Pipeline**
   - Create new service: `ProxyImportService`
   - Fetch proxies from providers on schedule
   - Validate new proxies before adding to pool
   - Remove duplicates and invalid entries
   - Store validated proxies in database
   - Test: Proxy pool grows automatically

4. **Tauri Commands for Provider Management**
   - File: `ui-tauri/src-tauri/src/main.rs`
   - Add commands: `fetch_free_proxies`, `get_provider_status`, `enable_provider`
   - Test: UI can trigger proxy fetching

**Success Criteria**:
- ✅ Free proxies are fetched automatically
- ✅ Invalid proxies are filtered out
- ✅ Proxy pool is always populated
- ✅ Multiple providers are supported
- ✅ Provider status is visible in UI

---

#### **PHASE 5: UI/UX Implementation** (PRIORITY: MEDIUM)
**Goal**: Complete and polish the user interface

**Tasks**:
1. **Address Bar Component**
   - File: `ui-tauri/src/components/browser/NavigationBar.svelte`
   - Add URL input with autocomplete
   - Add navigation buttons (back, forward, reload, home)
   - Show loading state and progress bar
   - Add SSL certificate indicator
   - Test: Navigate via address bar, verify all controls work

2. **Enhanced Tab List**
   - File: `ui-tauri/src/components/browser/EnhancedTabList.svelte`
   - Add tab switching, closing, dragging
   - Show tab favicons and titles
   - Display active proxy per tab
   - Add "new tab" button
   - Test: Manage multiple tabs via UI

3. **Settings Panel**
   - File: `ui-tauri/src/components/config/BrowserSettingsPanel.svelte`
   - Add proxy rotation strategy selector
   - Configure rotation intervals
   - Set default proxy settings
   - Add fingerprint protection options
   - Test: Settings persist and apply correctly

4. **Proxy Configuration UI**
   - File: `ui-tauri/src/components/config/ProxyConfiguration.svelte`
   - Add manual proxy entry form
   - Display proxy pool with health status
   - Enable/disable proxies
   - Test proxies from UI
   - Test: Manage proxy pool via UI

5. **Free IP Providers Panel**
   - File: `ui-tauri/src/components/config/FreeIpProvidersPanel.svelte`
   - List available providers with enable/disable toggles
   - Show last fetch time and proxy count
   - Trigger manual fetch
   - Test: Control free proxy providers

6. **Status Bar**
   - Create: `ui-tauri/src/components/browser/StatusBar.svelte`
   - Show current IP address
   - Display proxy status (connected/disconnected)
   - Show network activity indicator
   - Test: Status updates in real-time

**Success Criteria**:
- ✅ Complete browser UI with all controls
- ✅ Settings are configurable and persistent
- ✅ Proxy management is intuitive
- ✅ Real-time status updates work
- ✅ UI is responsive and polished

---

#### **PHASE 6: Advanced Features** (PRIORITY: MEDIUM)
**Goal**: Add browser fingerprint protection, storage isolation, and advanced features

**Tasks**:
1. **Cookie & Storage Isolation**
   - File: `crates/browser-core/src/storage.rs`
   - Implement per-tab cookie jar
   - Isolate localStorage, sessionStorage, IndexedDB
   - Clear storage on tab close (optional)
   - Test: Cookies don't leak between tabs

2. **Fingerprint Protection**
   - File: `crates/browser-core/src/fingerprint.rs`
   - Randomize User-Agent per tab
   - Spoof Canvas fingerprint
   - Spoof WebGL fingerprint
   - Randomize screen resolution, timezone, language
   - Test: Each tab has unique fingerprint

3. **Download Manager**
   - Create: `crates/browser-core/src/download_manager.rs`
   - Intercept download requests
   - Show download progress
   - Save downloads to user-selected location
   - Test: File downloads work correctly

4. **Bookmark & History Manager**
   - File: `crates/browser-core/src/storage.rs`
   - Implement bookmark CRUD operations
   - Store browsing history per tab
   - Add search functionality
   - Test: Bookmarks and history persist

5. **Session Management**
   - Create: `crates/browser-core/src/session_manager.rs`
   - Save open tabs on close
   - Restore session on startup
   - Export/import sessions
   - Test: Session restore works correctly

**Success Criteria**:
- ✅ Each tab has isolated storage
- ✅ Fingerprints are randomized and unique
- ✅ Downloads work seamlessly
- ✅ Bookmarks and history are managed
- ✅ Sessions can be saved and restored

---

#### **PHASE 7: Testing & Security** (PRIORITY: HIGH)
**Goal**: Comprehensive testing and security hardening

**Tasks**:
1. **Unit Tests**
   - Add tests for all core modules
   - Test proxy connection logic
   - Test IP generation and validation
   - Test tab isolation
   - Target: 80%+ code coverage

2. **Integration Tests**
   - File: `crates/browser-core/tests/`
   - Test end-to-end browser workflows
   - Test proxy rotation scenarios
   - Test tab lifecycle with real WebViews
   - Test database operations

3. **Security Auditing**
   - Fix TODO in `ui-tauri/src-tauri/src/auth.rs` (line 127): Implement proper password verification
   - Audit all input validation
   - Prevent SQL injection (verify sqlx parameterization)
   - Prevent XSS in WebView
   - Secure credential storage (use keyring)
   - Test: Run security scanners

4. **Performance Testing**
   - Test with 50+ simultaneous tabs
   - Measure memory usage and cleanup
   - Profile CPU usage during proxy rotation
   - Optimize bottlenecks

5. **Error Recovery Testing**
   - Test network failure scenarios
   - Test proxy failure handling
   - Test database corruption recovery
   - Ensure graceful degradation

**Success Criteria**:
- ✅ 80%+ test coverage
- ✅ All security TODOs resolved
- ✅ No critical vulnerabilities
- ✅ Handles 50+ tabs without crashes
- ✅ Error recovery is robust

---

#### **PHASE 8: Deployment & Distribution** (PRIORITY: LOW)
**Goal**: Build, package, and distribute the application

**Tasks**:
1. **Build Configuration**
   - Optimize release builds (already configured in Cargo.toml)
   - Configure Tauri bundler for all platforms
   - Add application icons
   - Test: Build succeeds on Windows, macOS, Linux

2. **Installer Creation**
   - Create MSI installer (Windows)
   - Create DMG installer (macOS)
   - Create DEB/AppImage (Linux)
   - Test: Installation works on fresh systems

3. **Auto-Update System**
   - Integrate Tauri updater
   - Set up update server
   - Test: App updates itself automatically

4. **Documentation**
   - Write user manual
   - Create developer documentation
   - Add troubleshooting guide
   - Test: Documentation is clear and complete

**Success Criteria**:
- ✅ App builds for all platforms
- ✅ Installers work correctly
- ✅ Auto-updates function
- ✅ Documentation is comprehensive

---

## 🤖 Agent Execution Patterns

### Pattern 1: Single File Implementation
**When to use**: Adding a new feature to an existing file

**Steps**:
1. Read the file to understand current implementation
2. Identify insertion point (follow existing patterns)
3. Implement the feature with proper error handling
4. Add necessary imports and dependencies
5. Write tests for the new functionality
6. Verify compilation with `cargo check`

**Example**: Adding a new Tauri command
```rust
// Step 1: Add to ui-tauri/src-tauri/src/main.rs
#[tauri::command]
async fn my_new_command(param: String) -> Result<String, String> {
    // Implementation
    Ok(result)
}

// Step 2: Register in main()
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            existing_commands,
            my_new_command  // Add here
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

### Pattern 2: Multi-File Feature Implementation
**When to use**: Adding a feature that spans multiple modules

**Steps**:
1. **Plan**: Identify all files that need changes
2. **Backend First**: Implement core logic in `crates/browser-core/`
3. **Tauri Bridge**: Add Tauri commands in `ui-tauri/src-tauri/src/main.rs`
4. **Type Definitions**: Update TypeScript types in `ui-tauri/src/lib/types.ts`
5. **API Layer**: Add API functions in `ui-tauri/src/lib/api.ts`
6. **UI Component**: Create/update Svelte components
7. **Integration**: Wire everything together
8. **Testing**: Test from UI to backend

**Example**: Adding Proxy Rotation Feature
```
Files to modify:
1. crates/browser-core/src/proxy_rotation.rs (core logic)
2. ui-tauri/src-tauri/src/main.rs (Tauri commands)
3. ui-tauri/src/lib/types.ts (TypeScript types)
4. ui-tauri/src/lib/api.ts (API wrapper)
5. ui-tauri/src/components/config/ProxyConfiguration.svelte (UI)
```

---

### Pattern 3: New Module Creation
**When to use**: Adding a completely new subsystem

**Steps**:
1. **Create Module File**: `crates/browser-core/src/my_module.rs`
2. **Define Public API**: Structs, enums, traits, functions
3. **Implement Core Logic**: Business logic with error handling
4. **Add to lib.rs**: Export public items
5. **Database Schema**: Add migrations if needed (`migrations/`))
6. **Add Tests**: Create `crates/browser-core/tests/my_module.rs`
7. **Integrate**: Connect to existing systems
8. **Document**: Add inline docs and README updates

**Example**: Creating Download Manager
```rust
// crates/browser-core/src/download_manager.rs
use std::path::PathBuf;
use anyhow::Result;

pub struct DownloadManager {
    downloads_dir: PathBuf,
}

impl DownloadManager {
    pub fn new(downloads_dir: PathBuf) -> Self {
        Self { downloads_dir }
    }

    pub async fn start_download(&self, url: &str) -> Result<String> {
        // Implementation
        Ok(download_id)
    }
}

// crates/browser-core/src/lib.rs
pub mod download_manager;
pub use download_manager::DownloadManager;
```

---

### Pattern 4: Database Operations
**When to use**: Adding database tables or queries

**Steps**:
1. **Create Migration**: `crates/browser-core/migrations/XXX_description.sql`
2. **Write SQL**: CREATE TABLE, indexes, constraints
3. **Update Database Module**: Add query functions to `database.rs`
4. **Use sqlx Macros**: Use `sqlx::query!` for type safety
5. **Test**: Verify migrations apply cleanly
6. **Integrate**: Use in business logic

**Example**: Adding Proxy Sessions Table
```sql
-- migrations/005_proxy_sessions.sql
CREATE TABLE IF NOT EXISTS proxy_sessions (
    id TEXT PRIMARY KEY,
    tab_id TEXT NOT NULL,
    proxy_id TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    ended_at INTEGER,
    requests_count INTEGER DEFAULT 0,
    FOREIGN KEY (tab_id) REFERENCES tabs(id),
    FOREIGN KEY (proxy_id) REFERENCES proxies(id)
);
```

```rust
// crates/browser-core/src/database.rs
impl Database {
    pub async fn create_proxy_session(
        &self,
        session: &ProxySession
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "INSERT INTO proxy_sessions (id, tab_id, proxy_id, started_at) 
             VALUES (?, ?, ?, ?)",
            session.id,
            session.tab_id,
            session.proxy_id,
            session.started_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
```

---

### Pattern 5: Svelte Component Creation
**When to use**: Building new UI components

**Steps**:
1. **Create Component File**: `ui-tauri/src/components/category/MyComponent.svelte`
2. **Import Types**: From `src/lib/types.ts`
3. **Import API Functions**: From `src/lib/api.ts`
4. **Define Props**: Using TypeScript
5. **Add State Management**: Use stores if needed
6. **Implement UI**: HTML + Tailwind CSS
7. **Add Event Handlers**: Call API functions
8. **Export**: Make available to parent components

**Example**: Creating Status Indicator Component
```svelte
<script lang="ts">
import { invoke } from '@tauri-apps/api/core';
import type { ProxyStatus } from '/types';
import { onMount, onDestroy } from 'svelte';

export let tabId: string;

let status: ProxyStatus | null = null;
let interval: number;

onMount(async () => {
    await updateStatus();
    interval = setInterval(updateStatus, 5000);
});

onDestroy(() => {
    if (interval) clearInterval(interval);
});

async function updateStatus() {
    try {
        status = await invoke<ProxyStatus>('get_proxy_status', { tabId });
    } catch (err) {
        console.error('Failed to get status:', err);
    }
}
</script>

<div class="status-indicator">
    {#if status}
        <span class="status-dot" class:connected={status.connected}></span>
        <span>{status.ip_address}</span>
    {:else}
        <span>Loading...</span>
    {/if}
</div>

<style>
.status-dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: red;
}
.status-dot.connected {
    background-color: green;
}
</style>
```

---

## 📝 Code Generation Guidelines

### Rust Backend Guidelines

#### Error Handling
**Always use proper error handling**:
```rust
// ❌ AVOID: Unwrapping that can panic
let result = some_operation().unwrap();

// ✅ PREFER: Returning Result
pub async fn my_function() -> Result<Data, MyError> {
    let result = some_operation()
        .map_err(|e| MyError::OperationFailed(e.to_string()))?;
    Ok(result)
}

// ✅ PREFER: Using anyhow for applications
pub async fn my_function() -> anyhow::Result<Data> {
    let result = some_operation()
        .context("Failed to perform operation")?;
    Ok(result)
}
```

#### Async/Await
**Use tokio runtime properly**:
```rust
// ✅ Async function with proper error handling
pub async fn fetch_proxies() -> Result<Vec<Proxy>, reqwest::Error> {
    let response = reqwest::get("https://api.example.com/proxies")
        .await?
        .json::<Vec<Proxy>>()
        .await?;
    Ok(response)
}

// ✅ Spawning background tasks
tokio::spawn(async move {
    loop {
        if let Err(e) = health_check().await {
            eprintln!("Health check failed: {}", e);
        }
        tokio::time::sleep(Duration::from_secs(300)).await;
    }
});
```

#### Database Operations
**Use sqlx with type safety**:
```rust
// ✅ Type-safe queries with compile-time verification
pub async fn get_proxy(&self, id: &str) -> Result<Option<Proxy>, DatabaseError> {
    let proxy = sqlx::query_as!(
        Proxy,
        "SELECT id, host, port, protocol, username, password FROM proxies WHERE id = ?",
        id
    )
    .fetch_optional(&self.pool)
    .await?;
    Ok(proxy)
}

// ✅ Transactions for multiple operations
pub async fn rotate_proxy(&self, tab_id: &str, new_proxy_id: &str) -> Result<(), DatabaseError> {
    let mut tx = self.pool.begin().await?;
    
    // End current session
    sqlx::query!("UPDATE proxy_sessions SET ended_at = ? WHERE tab_id = ? AND ended_at IS NULL", 
        chrono::Utc::now().timestamp(), tab_id)
        .execute(&mut *tx)
        .await?;
    
    // Start new session
    sqlx::query!("INSERT INTO proxy_sessions (id, tab_id, proxy_id, started_at) VALUES (?, ?, ?, ?)",
        Uuid::new_v4().to_string(), tab_id, new_proxy_id, chrono::Utc::now().timestamp())
        .execute(&mut *tx)
        .await?;
    
    tx.commit().await?;
    Ok(())
}
```

#### Logging
**Use tracing for structured logging**:
```rust
use tracing::{info, warn, error, debug, instrument};

#[instrument(skip(self), fields(tab_id = %tab_id))]
pub async fn create_tab(&self, tab_id: &str) -> Result<Tab, TabError> {
    info!("Creating new tab");
    
    let tab = Tab::new(tab_id);
    
    debug!("Assigning proxy to tab");
    if let Err(e) = self.assign_proxy(&tab).await {
        warn!("Failed to assign proxy: {}", e);
        // Fallback to direct connection
    }
    
    info!("Tab created successfully");
    Ok(tab)
}
```

---

### TypeScript/Svelte Guidelines

#### Type Safety
**Always define types explicitly**:
```typescript
// ui-tauri/src/lib/types.ts

export interface Tab {
    id: string;
    url: string;
    title: string;
    favicon?: string;
    proxyId?: string;
    createdAt: number;
}

export interface ProxyStatus {
    connected: boolean;
    ipAddress?: string;
    latency?: number;
    lastChecked: number;
}

export type ProxyRotationStrategy = 'round-robin' | 'random' | 'latency-based';
```

#### API Wrapper Functions
**Create typed API wrappers**:
```typescript
// ui-tauri/src/lib/api.ts
import { invoke } from '@tauri-apps/api/core';
import type { Tab, ProxyStatus, Proxy } from './types';

export async function createTab(url: string): Promise<Tab> {
    return await invoke<Tab>('create_tab', { url });
}

export async function getProxyStatus(tabId: string): Promise<ProxyStatus> {
    return await invoke<ProxyStatus>('get_proxy_status', { tabId });
}

export async function rotateProxy(tabId: string): Promise<Proxy> {
    return await invoke<Proxy>('rotate_proxy', { tabId });
}
```

#### Svelte Stores
**Use stores for global state**:
```typescript
// ui-tauri/src/lib/stores.ts
import { writable, derived } from 'svelte/store';
import type { Tab, Proxy } from './types';

export const tabs = writable<Tab[]>([]);
export const proxies = writable<Proxy[]>([]);
export const activeTabId = writable<string | null>(null);

export const activeTab = derived(
    [tabs, activeTabId],
    ([\, \]) => \.find(t => t.id === \)
);
```

---

### Security Best Practices

#### Input Validation
**Always validate and sanitize user input**:
```rust
use validator::Validate;
use ammonia::clean;

#[derive(Validate)]
pub struct ProxyInput {
    #[validate(length(min = 7, max = 255))]
    host: String,
    
    #[validate(range(min = 1, max = 65535))]
    port: u16,
}

pub fn sanitize_url(url: &str) -> String {
    clean(url)
}
```

#### SQL Injection Prevention
**Use parameterized queries (sqlx does this automatically)**:
```rust
// ✅ SAFE: Parameterized query
sqlx::query!("SELECT * FROM tabs WHERE id = ?", tab_id)
    .fetch_one(&pool)
    .await?;

// ❌ DANGEROUS: String concatenation
// NEVER DO THIS:
// let query = format!("SELECT * FROM tabs WHERE id = '{}'", tab_id);
```

#### Credential Storage
**Use system keyring for sensitive data**:
```rust
use keyring::Entry;

pub fn store_proxy_credentials(proxy_id: &str, username: &str, password: &str) -> Result<()> {
    let entry = Entry::new("virtual-ip-browser", proxy_id)?;
    entry.set_password(&format!("{}:{}", username, password))?;
    Ok(())
}

pub fn get_proxy_credentials(proxy_id: &str) -> Result<(String, String)> {
    let entry = Entry::new("virtual-ip-browser", proxy_id)?;
    let creds = entry.get_password()?;
    let parts: Vec<&str> = creds.split(':').collect();
    Ok((parts[0].to_string(), parts[1].to_string()))
}
```

---

## 🧪 Testing Strategy

### Unit Testing Pattern
**Test individual functions and modules**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_proxy_validation() {
        let validator = ProxyValidator::new();
        let proxy = Proxy {
            host: "127.0.0.1".to_string(),
            port: 8080,
            protocol: ProxyProtocol::Http,
            username: None,
            password: None,
        };
        
        let result = validator.validate(&proxy).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_ip_generation() {
        let generator = IpGenerator::new();
        let ip = generator.generate();
        assert!(ip.octets()[0] > 0);
    }
}
```

### Integration Testing Pattern
**Test multi-module interactions**:
```rust
// crates/browser-core/tests/tab_lifecycle.rs
use browser_core::{BrowserTabManager, ProxyManager, Database};

#[tokio::test]
async fn test_tab_with_proxy() {
    let db = Database::new(":memory:").await.unwrap();
    let proxy_manager = ProxyManager::new(db.clone());
    let tab_manager = BrowserTabManager::new(db.clone(), proxy_manager);

    // Create tab
    let tab = tab_manager.create_tab("https://example.com").await.unwrap();
    assert!(tab.proxy_id.is_some());

    // Verify proxy assigned
    let proxy = proxy_manager.get_proxy(&tab.proxy_id.unwrap()).await.unwrap();
    assert!(proxy.is_some());

    // Close tab
    tab_manager.close_tab(&tab.id).await.unwrap();
}
```

### End-to-End Testing
**Test complete workflows from UI to backend**:
```typescript
// Using Playwright or similar
import { test, expect } from '@playwright/test';

test('create tab and navigate', async ({ page }) => {
    await page.goto('http://localhost:1420');
    
    // Create new tab
    await page.click('[data-testid="new-tab-button"]');
    await expect(page.locator('[data-testid="tab-list"]')).toContainText('New Tab');
    
    // Navigate to URL
    await page.fill('[data-testid="address-bar"]', 'https://example.com');
    await page.press('[data-testid="address-bar"]', 'Enter');
    
    // Verify navigation
    await expect(page.locator('[data-testid="active-tab"]')).toContainText('Example Domain');
});
```

---


## 🛠️ Common Tasks & Solutions

### Task 1: Adding a New Tauri Command
**Scenario**: Expose backend functionality to the frontend

**Steps**:
1. **Implement Rust function** in appropriate module
2. **Create Tauri command** in `ui-tauri/src-tauri/src/main.rs`
3. **Register command** in handler
4. **Add TypeScript wrapper** in `ui-tauri/src/lib/api.ts`
5. **Update types** in `ui-tauri/src/lib/types.ts`
6. **Call from UI** component

**Example**:
```rust
// ui-tauri/src-tauri/src/main.rs
#[tauri::command]
async fn get_active_proxy(tab_id: String, state: tauri::State<'_, AppState>) -> Result<Option<Proxy>, String> {
    state.proxy_manager
        .get_active_proxy(&tab_id)
        .await
        .map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_active_proxy,  // Register here
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

```typescript
// ui-tauri/src/lib/api.ts
export async function getActiveProxy(tabId: string): Promise<Proxy | null> {
    return await invoke<Proxy | null>('get_active_proxy', { tabId });
}
```

---

### Task 2: Adding Database Migration
**Scenario**: Need to add new table or modify schema

**Steps**:
1. **Create migration file**: `crates/browser-core/migrations/XXX_name.sql`
2. **Write SQL**: Use SQLite syntax
3. **Test migration**: Run with `sqlx migrate run`
4. **Update database module**: Add query functions
5. **Update models**: Add/modify structs

**Example**:
```sql
-- migrations/006_proxy_metrics.sql
CREATE TABLE IF NOT EXISTS proxy_metrics (
    id TEXT PRIMARY KEY,
    proxy_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    latency_ms INTEGER,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    FOREIGN KEY (proxy_id) REFERENCES proxies(id)
);

CREATE INDEX idx_proxy_metrics_proxy_id ON proxy_metrics(proxy_id);
CREATE INDEX idx_proxy_metrics_timestamp ON proxy_metrics(timestamp);
```

```rust
// crates/browser-core/src/database.rs
impl Database {
    pub async fn log_proxy_metric(&self, metric: &ProxyMetric) -> Result<()> {
        sqlx::query!(
            "INSERT INTO proxy_metrics (id, proxy_id, timestamp, latency_ms, success, error_message) 
             VALUES (?, ?, ?, ?, ?, ?)",
            metric.id,
            metric.proxy_id,
            metric.timestamp,
            metric.latency_ms,
            metric.success,
            metric.error_message
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
```

---

### Task 3: Implementing Proxy Rotation
**Scenario**: Automatically switch proxies based on strategy

**Key Files**:
- `crates/browser-core/src/proxy_rotation.rs`
- `crates/browser-core/src/browser_tab_manager.rs`

**Implementation**:
```rust
// proxy_rotation.rs
pub struct ProxyRotator {
    strategy: RotationStrategy,
    proxy_manager: Arc<ProxyManager>,
}

impl ProxyRotator {
    pub async fn rotate(&self, tab_id: &str) -> Result<Proxy> {
        let current_proxy = self.proxy_manager.get_active_proxy(tab_id).await?;
        
        let next_proxy = match self.strategy {
            RotationStrategy::RoundRobin => {
                self.get_next_round_robin(current_proxy.as_ref()).await?
            }
            RotationStrategy::Random => {
                self.get_random_proxy().await?
            }
            RotationStrategy::LatencyBased => {
                self.get_fastest_proxy().await?
            }
        };
        
        // Update tab's proxy
        self.proxy_manager.assign_proxy(tab_id, &next_proxy.id).await?;
        
        Ok(next_proxy)
    }
}
```

---

### Task 4: Adding UI Component
**Scenario**: Create new Svelte component for feature

**Steps**:
1. **Create component file** in appropriate category
2. **Define props and state**
3. **Import API functions**
4. **Implement UI logic**
5. **Add styling**
6. **Import in parent component**

**Example** (Proxy Rotation Control):
```svelte
<!-- ui-tauri/src/components/config/ProxyRotationControl.svelte -->
<script lang="ts">
import { invoke } from '@tauri-apps/api/core';
import type { RotationStrategy } from '$lib/types';

export let tabId: string;

let strategy: RotationStrategy = 'round-robin';
let intervalMinutes: number = 5;
let enabled: boolean = false;

async function updateRotationConfig() {
    try {
        await invoke('set_rotation_config', {
            tabId,
            strategy,
            intervalMinutes,
            enabled
        });
    } catch (err) {
        console.error('Failed to update rotation config:', err);
    }
}

async function rotateNow() {
    try {
        await invoke('rotate_proxy', { tabId });
        alert('Proxy rotated successfully');
    } catch (err) {
        alert('Failed to rotate proxy: ' + err);
    }
}
</script>

<div class="rotation-control">
    <h3>Proxy Rotation</h3>
    
    <label>
        <input type="checkbox" bind:checked={enabled} on:change={updateRotationConfig} />
        Enable Auto-Rotation
    </label>
    
    {#if enabled}
        <div class="config-section">
            <label>
                Strategy:
                <select bind:value={strategy} on:change={updateRotationConfig}>
                    <option value="round-robin">Round Robin</option>
                    <option value="random">Random</option>
                    <option value="latency-based">Latency Based</option>
                </select>
            </label>
            
            <label>
                Interval (minutes):
                <input type="number" bind:value={intervalMinutes} 
                       min="1" max="60" on:change={updateRotationConfig} />
            </label>
        </div>
    {/if}
    
    <button on:click={rotateNow}>Rotate Now</button>
</div>

<style>
.rotation-control {
    padding: 1rem;
    border: 1px solid #ccc;
    border-radius: 4px;
}

.config-section {
    margin-top: 1rem;
    padding: 0.5rem;
    background-color: #f5f5f5;
}

label {
    display: block;
    margin-bottom: 0.5rem;
}
</style>
```

---

### Task 5: Integrating Free Proxy Provider
**Scenario**: Add support for new free proxy API

**Steps**:
1. **Research API** documentation and endpoints
2. **Update `free_ip_providers.rs`** with new provider
3. **Implement fetch function** with reqwest
4. **Parse response** to Proxy objects
5. **Add to provider list**
6. **Test with real API**

**Example**:
```rust
// crates/browser-core/src/free_ip_providers.rs

pub async fn fetch_from_proxyscrape() -> Result<Vec<Proxy>> {
    let url = "https://api.proxyscrape.com/v2/?request=displayproxies&protocol=http&timeout=10000&country=all&ssl=all&anonymity=all";
    
    let response = reqwest::get(url).await?;
    let text = response.text().await?;
    
    let proxies: Vec<Proxy> = text
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 2 {
                Some(Proxy {
                    id: Uuid::new_v4().to_string(),
                    host: parts[0].to_string(),
                    port: parts[1].parse().ok()?,
                    protocol: ProxyProtocol::Http,
                    username: None,
                    password: None,
                    country: None,
                    health_score: 0.5,
                    last_checked: None,
                })
            } else {
                None
            }
        })
        .collect();
    
    Ok(proxies)
}

pub async fn fetch_all_providers() -> Vec<Proxy> {
    let mut all_proxies = Vec::new();
    
    // Fetch from multiple providers concurrently
    let results = tokio::join!(
        fetch_from_proxyscrape(),
        fetch_from_geonode(),
        fetch_from_proxylist(),
    );
    
    if let Ok(proxies) = results.0 {
        all_proxies.extend(proxies);
    }
    if let Ok(proxies) = results.1 {
        all_proxies.extend(proxies);
    }
    if let Ok(proxies) = results.2 {
        all_proxies.extend(proxies);
    }
    
    all_proxies
}
```

---

## 🔍 Troubleshooting Guide

### Issue: Cargo Build Fails

**Symptoms**: Compilation errors, dependency conflicts

**Solutions**:
1. **Clean and rebuild**:
   ```bash
   cargo clean
   cargo build
   ```

2. **Update dependencies**:
   ```bash
   cargo update
   ```

3. **Check Rust version** (needs 1.75+):
   ```bash
   rustc --version
   rustup update
   ```

4. **Verify sqlx migrations**:
   ```bash
   cd crates/browser-core
   sqlx migrate run --database-url sqlite:test.db
   ```

---

### Issue: Tauri App Won't Start

**Symptoms**: Window doesn't appear, crashes on startup

**Solutions**:
1. **Check console logs** in terminal
2. **Verify database initialization**:
   ```rust
   // Ensure database is created before use
   let db = Database::new("./app.db").await?;
   db.run_migrations().await?;
   ```

3. **Check Tauri config**:
   - Verify `ui-tauri/src-tauri/tauri.conf.json`
   - Check window configuration
   - Verify build settings

4. **Test in dev mode**:
   ```bash
   cd ui-tauri
   bun run tauri dev
   ```

---

### Issue: Proxy Connection Fails

**Symptoms**: Tabs can't connect through proxy, timeout errors

**Solutions**:
1. **Validate proxy manually**:
   ```bash
   curl -x http://proxy_host:proxy_port https://api.ipify.org
   ```

2. **Check proxy credentials**:
   - Verify username/password if required
   - Check authentication method (Basic, Digest)

3. **Test local proxy server**:
   ```rust
   // Add debug logging
   tracing::info!("Connecting to proxy: {}:{}", proxy.host, proxy.port);
   ```

4. **Verify firewall settings**:
   - Ensure app has network permissions
   - Check if proxy ports are accessible

5. **Check DNS resolution**:
   - Proxy host must be resolvable
   - Consider using IP address instead

---

## 📚 Reference Documentation

### Important Files to Know

#### Configuration Files
- `Cargo.toml` - Root workspace configuration
- `ui-tauri/src-tauri/Cargo.toml` - Tauri app dependencies
- `ui-tauri/package.json` - Frontend dependencies
- `ui-tauri/src-tauri/tauri.conf.json` - Tauri configuration
- `bunfig.toml` - Bun configuration

#### Database
- `crates/browser-core/migrations/` - SQL migrations
- `crates/browser-core/src/database.rs` - Database operations

#### Core Logic
- `crates/browser-core/src/lib.rs` - Public API exports
- `crates/browser-core/src/browser_tab_manager.rs` - Tab management
- `crates/browser-core/src/proxy.rs` - Proxy management
- `crates/browser-core/src/proxy_rotation.rs` - Rotation logic

#### UI Entry Points
- `ui-tauri/src/main.ts` - Frontend entry
- `ui-tauri/src/App.svelte` - Root component
- `ui-tauri/src-tauri/src/main.rs` - Backend entry

---

### Key TODOs to Address

Based on code analysis, here are critical TODOs found in the codebase:

1. **`ui-tauri/src-tauri/src/webview_manager.rs:43`**
   - TODO: Implement per-webview proxy configuration
   - Priority: CRITICAL
   - Required for: Phase 2 - Proxy integration

2. **`ui-tauri/src-tauri/src/auth.rs:127`**
   - TODO: Implement proper password verification with hashing
   - Priority: HIGH (Security)
   - Required for: Phase 7 - Security hardening

3. **`crates/browser-core/src/tab_manager.rs:110`**
   - TODO: Add cleanup logic for closed tabs
   - Priority: HIGH
   - Required for: Phase 1 - Tab lifecycle

4. **`crates/browser-core/src/proxy_rotation.rs`**
   - Multiple TODOs for rotation strategies
   - Priority: HIGH
   - Required for: Phase 3 - Proxy rotation

---

### External Resources

#### Rust Crates Documentation
- **Tauri**: https://tauri.app/v2/
- **sqlx**: https://docs.rs/sqlx/
- **reqwest**: https://docs.rs/reqwest/
- **tokio**: https://tokio.rs/
- **chromiumoxide**: https://docs.rs/chromiumoxide/

#### Frontend Documentation
- **Svelte**: https://svelte.dev/docs
- **TypeScript**: https://www.typescriptlang.org/docs/
- **Vite**: https://vitejs.dev/guide/
- **Bun**: https://bun.sh/docs

#### Proxy & Networking
- **SOCKS5 Protocol**: RFC 1928
- **PAC Files**: https://developer.mozilla.org/en-US/docs/Web/HTTP/Proxy_servers_and_tunneling/Proxy_Auto-Configuration_PAC_file
- **Free Proxy APIs**:
  - ProxyScrape: https://proxyscrape.com/api
  - GeoNode: https://geonode.com/free-proxy-list
  - ProxyList: https://www.proxy-list.download/api

---

### Development Commands Cheat Sheet

```bash
# Build entire workspace
cargo build

# Build release
cargo build --release

# Run tests
cargo test

# Run specific test
cargo test test_name

# Check code without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy

# Run Tauri dev mode
cd ui-tauri && bun run tauri dev

# Build Tauri app
cd ui-tauri && bun run tauri build

# Run database migrations
cd crates/browser-core && sqlx migrate run

# Create new migration
sqlx migrate add migration_name

# Update frontend dependencies
cd ui-tauri && bun update

# Install new Rust dependency
cargo add dependency_name
```

---

## 🎯 Quick Start for Agent

### When Starting New Task:
1. **Read this document** to understand project structure
2. **Identify which phase** the task belongs to
3. **Review relevant patterns** from "Agent Execution Patterns"
4. **Check for TODOs** in related files
5. **Follow code generation guidelines**
6. **Write tests** for new functionality
7. **Update this document** if adding new patterns

### Priority Order for Implementation:
1. **Phase 1: Core Browser Engine** - Get basic browser working
2. **Phase 2: Proxy Integration** - Enable proxy routing
3. **Phase 3: Proxy Rotation** - Add auto-switching
4. **Phase 4: Free Proxy Providers** - Populate proxy pool
5. **Phase 5: UI/UX** - Polish user interface
6. **Phase 6: Advanced Features** - Add fingerprinting, isolation
7. **Phase 7: Testing & Security** - Harden and test
8. **Phase 8: Deployment** - Package and distribute

### Before Committing Code:
- ✅ Code compiles without warnings
- ✅ Tests pass (`cargo test`)
- ✅ Code is formatted (`cargo fmt`)
- ✅ No clippy warnings (`cargo clippy`)
- ✅ Changes are documented
- ✅ TODOs are addressed or documented

---

## 🎓 Learning Resources

### For Understanding the Codebase:
1. Start with `README.md` and `PROJECT_SUMMARY.md`
2. Read `COMPREHENSIVE_DEVELOPMENT_PLAN.md` for detailed architecture
3. Study `TECH_STACK_FINAL_DECISION.md` for technology choices
4. Review `IMPLEMENTATION_CHECKLIST.md` for task breakdown

### For Learning Technologies:
1. **Rust**: The Rust Programming Language book
2. **Tauri**: Official Tauri v2 documentation
3. **Svelte**: Svelte tutorial and docs
4. **SQLite**: SQLite documentation
5. **Proxy Protocols**: RFCs for SOCKS5, HTTP CONNECT

---

## 🤝 Best Practices Summary

### Code Quality
- Write type-safe code (Rust's type system, TypeScript types)
- Handle all errors explicitly (no unwraps in production code)
- Use async/await properly with tokio
- Add tracing/logging for debugging
- Write unit tests for all business logic

### Architecture
- Keep business logic in `browser-core` crate
- Use Tauri commands as thin wrappers
- Separate concerns (UI, business logic, data access)
- Use database for persistent state
- Use stores for UI state

### Security
- Validate all user input
- Use parameterized SQL queries
- Store credentials securely (keyring)
- Prevent XSS in WebViews
- Regular security audits

### Performance
- Use connection pooling for database
- Spawn background tasks for long operations
- Clean up resources properly
- Profile and optimize hot paths
- Test with realistic data volumes

---

**Last Updated**: 2024-01-20
**Agent Version**: 1.0.0
**Project Status**: In Development - Phase 1 & 2 Critical

---

## 🤖 Agent Execution Patterns

### Pattern 1: Single File Implementation
**When to use**: Adding a new feature to an existing file

**Steps**:
1. Read the file to understand current implementation
2. Identify insertion point (follow existing patterns)
3. Implement the feature with proper error handling
4. Add necessary imports and dependencies
5. Write tests for the new functionality
6. Verify compilation with `cargo check`

**Example**: Adding a new Tauri command
```rust
// Step 1: Add to ui-tauri/src-tauri/src/main.rs
#[tauri::command]
async fn my_new_command(param: String) -> Result<String, String> {
    // Implementation
    Ok(result)
}

// Step 2: Register in main()
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            existing_commands,
            my_new_command  // Add here
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

### Pattern 2: Multi-File Feature Implementation
**When to use**: Adding a feature that spans multiple modules

**Steps**:
1. **Plan**: Identify all files that need changes
2. **Backend First**: Implement core logic in `crates/browser-core/`
3. **Tauri Bridge**: Add Tauri commands in `ui-tauri/src-tauri/src/main.rs`
4. **Type Definitions**: Update TypeScript types in `ui-tauri/src/lib/types.ts`
5. **API Layer**: Add API functions in `ui-tauri/src/lib/api.ts`
6. **UI Component**: Create/update Svelte components
7. **Integration**: Wire everything together
8. **Testing**: Test from UI to backend

**Example**: Adding Proxy Rotation Feature
```
Files to modify:
1. crates/browser-core/src/proxy_rotation.rs (core logic)
2. ui-tauri/src-tauri/src/main.rs (Tauri commands)
3. ui-tauri/src/lib/types.ts (TypeScript types)
4. ui-tauri/src/lib/api.ts (API wrapper)
5. ui-tauri/src/components/config/ProxyConfiguration.svelte (UI)
```

---

### Pattern 3: New Module Creation
**When to use**: Adding a completely new subsystem

**Steps**:
1. **Create Module File**: `crates/browser-core/src/my_module.rs`
2. **Define Public API**: Structs, enums, traits, functions
3. **Implement Core Logic**: Business logic with error handling
4. **Add to lib.rs**: Export public items
5. **Database Schema**: Add migrations if needed (`migrations/`)
6. **Add Tests**: Create `crates/browser-core/tests/my_module.rs`
7. **Integrate**: Connect to existing systems
8. **Document**: Add inline docs and README updates

**Example**: Creating Download Manager
```rust
// crates/browser-core/src/download_manager.rs
use std::path::PathBuf;
use anyhow::Result;

pub struct DownloadManager {
    downloads_dir: PathBuf,
}

impl DownloadManager {
    pub fn new(downloads_dir: PathBuf) -> Self {
        Self { downloads_dir }
    }

    pub async fn start_download(&self, url: &str) -> Result<String> {
        // Implementation
        Ok(download_id)
    }
}

// crates/browser-core/src/lib.rs
pub mod download_manager;
pub use download_manager::DownloadManager;
```

---

### Pattern 4: Database Operations
**When to use**: Adding database tables or queries

**Steps**:
1. **Create Migration**: `crates/browser-core/migrations/XXX_description.sql`
2. **Write SQL**: CREATE TABLE, indexes, constraints
3. **Update Database Module**: Add query functions to `database.rs`
4. **Use sqlx Macros**: Use `sqlx::query!` for type safety
5. **Test**: Verify migrations apply cleanly
6. **Integrate**: Use in business logic

**Example**: Adding Proxy Sessions Table
```sql
-- migrations/005_proxy_sessions.sql
CREATE TABLE IF NOT EXISTS proxy_sessions (
    id TEXT PRIMARY KEY,
    tab_id TEXT NOT NULL,
    proxy_id TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    ended_at INTEGER,
    requests_count INTEGER DEFAULT 0,
    FOREIGN KEY (tab_id) REFERENCES tabs(id),
    FOREIGN KEY (proxy_id) REFERENCES proxies(id)
);
```

```rust
// crates/browser-core/src/database.rs
impl Database {
    pub async fn create_proxy_session(
        &self,
        session: &ProxySession
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "INSERT INTO proxy_sessions (id, tab_id, proxy_id, started_at) 
             VALUES (?, ?, ?, ?)",
            session.id,
            session.tab_id,
            session.proxy_id,
            session.started_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
```

---

### Pattern 5: Svelte Component Creation
**When to use**: Building new UI components

**Steps**:
1. **Create Component File**: `ui-tauri/src/components/category/MyComponent.svelte`
2. **Import Types**: From `src/lib/types.ts`
3. **Import API Functions**: From `src/lib/api.ts`
4. **Define Props**: Using TypeScript
5. **Add State Management**: Use stores if needed
6. **Implement UI**: HTML + Tailwind CSS
7. **Add Event Handlers**: Call API functions
8. **Export**: Make available to parent components

**Example**: Creating Status Indicator Component
```svelte
<script lang="ts">
import { invoke } from '@tauri-apps/api/core';
import type { ProxyStatus } from '$lib/types';
import { onMount, onDestroy } from 'svelte';

export let tabId: string;

let status: ProxyStatus | null = null;
let interval: number;

onMount(async () => {
    await updateStatus();
    interval = setInterval(updateStatus, 5000);
});

onDestroy(() => {
    if (interval) clearInterval(interval);
});

async function updateStatus() {
    try {
        status = await invoke<ProxyStatus>('get_proxy_status', { tabId });
    } catch (err) {
        console.error('Failed to get status:', err);
    }
}
</script>

<div class="status-indicator">
    {#if status}
        <span class="status-dot" class:connected={status.connected}></span>
        <span>{status.ip_address}</span>
    {:else}
        <span>Loading...</span>
    {/if}
</div>

<style>
.status-dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: red;
}
.status-dot.connected {
    background-color: green;
}
</style>
```

---

## 📝 Code Generation Guidelines

### Rust Backend Guidelines

#### Error Handling
**Always use proper error handling**:
```rust
// ❌ AVOID: Unwrapping that can panic
let result = some_operation().unwrap();

// ✅ PREFER: Returning Result
pub async fn my_function() -> Result<Data, MyError> {
    let result = some_operation()
        .map_err(|e| MyError::OperationFailed(e.to_string()))?;
    Ok(result)
}

// ✅ PREFER: Using anyhow for applications
pub async fn my_function() -> anyhow::Result<Data> {
    let result = some_operation()
        .context("Failed to perform operation")?;
    Ok(result)
}
```

#### Async/Await
**Use tokio runtime properly**:
```rust
// ✅ Async function with proper error handling
pub async fn fetch_proxies() -> Result<Vec<Proxy>, reqwest::Error> {
    let response = reqwest::get("https://api.example.com/proxies")
        .await?
        .json::<Vec<Proxy>>()
        .await?;
    Ok(response)
}

// ✅ Spawning background tasks
tokio::spawn(async move {
    loop {
        if let Err(e) = health_check().await {
            eprintln!("Health check failed: {}", e);
        }
        tokio::time::sleep(Duration::from_secs(300)).await;
    }
});
```

#### Database Operations
**Use sqlx with type safety**:
```rust
// ✅ Type-safe queries with compile-time verification
pub async fn get_proxy(&self, id: &str) -> Result<Option<Proxy>, DatabaseError> {
    let proxy = sqlx::query_as!(
        Proxy,
        "SELECT id, host, port, protocol, username, password FROM proxies WHERE id = ?",
        id
    )
    .fetch_optional(&self.pool)
    .await?;
    Ok(proxy)
}

// ✅ Transactions for multiple operations
pub async fn rotate_proxy(&self, tab_id: &str, new_proxy_id: &str) -> Result<(), DatabaseError> {
    let mut tx = self.pool.begin().await?;
    
    // End current session
    sqlx::query!("UPDATE proxy_sessions SET ended_at = ? WHERE tab_id = ? AND ended_at IS NULL", 
        chrono::Utc::now().timestamp(), tab_id)
        .execute(&mut *tx)
        .await?;
    
    // Start new session
    sqlx::query!("INSERT INTO proxy_sessions (id, tab_id, proxy_id, started_at) VALUES (?, ?, ?, ?)",
        Uuid::new_v4().to_string(), tab_id, new_proxy_id, chrono::Utc::now().timestamp())
        .execute(&mut *tx)
        .await?;
    
    tx.commit().await?;
    Ok(())
}
```

#### Logging
**Use tracing for structured logging**:
```rust
use tracing::{info, warn, error, debug, instrument};

#[instrument(skip(self), fields(tab_id = %tab_id))]
pub async fn create_tab(&self, tab_id: &str) -> Result<Tab, TabError> {
    info!("Creating new tab");
    
    let tab = Tab::new(tab_id);
    
    debug!("Assigning proxy to tab");
    if let Err(e) = self.assign_proxy(&tab).await {
        warn!("Failed to assign proxy: {}", e);
        // Fallback to direct connection
    }
    
    info!("Tab created successfully");
    Ok(tab)
}
```

---

### TypeScript/Svelte Guidelines

#### Type Safety
**Always define types explicitly**:
```typescript
// ui-tauri/src/lib/types.ts

export interface Tab {
    id: string;
    url: string;
    title: string;
    favicon?: string;
    proxyId?: string;
    createdAt: number;
}

export interface ProxyStatus {
    connected: boolean;
    ipAddress?: string;
    latency?: number;
    lastChecked: number;
}

export type ProxyRotationStrategy = 'round-robin' | 'random' | 'latency-based';
```

#### API Wrapper Functions
**Create typed API wrappers**:
```typescript
// ui-tauri/src/lib/api.ts
import { invoke } from '@tauri-apps/api/core';
import type { Tab, ProxyStatus, Proxy } from './types';

export async function createTab(url: string): Promise<Tab> {
    return await invoke<Tab>('create_tab', { url });
}

export async function getProxyStatus(tabId: string): Promise<ProxyStatus> {
    return await invoke<ProxyStatus>('get_proxy_status', { tabId });
}

export async function rotateProxy(tabId: string): Promise<Proxy> {
    return await invoke<Proxy>('rotate_proxy', { tabId });
}
```

#### Svelte Stores
**Use stores for global state**:
```typescript
// ui-tauri/src/lib/stores.ts
import { writable, derived } from 'svelte/store';
import type { Tab, Proxy } from './types';

export const tabs = writable<Tab[]>([]);
export const proxies = writable<Proxy[]>([]);
export const activeTabId = writable<string | null>(null);

export const activeTab = derived(
    [tabs, activeTabId],
    ([$tabs, $activeTabId]) => $tabs.find(t => t.id === $activeTabId)
);
```

---

### Security Best Practices

#### Input Validation
**Always validate and sanitize user input**:
```rust
use validator::Validate;
use ammonia::clean;

#[derive(Validate)]
pub struct ProxyInput {
    #[validate(length(min = 7, max = 255))]
    host: String,
    
    #[validate(range(min = 1, max = 65535))]
    port: u16,
}

pub fn sanitize_url(url: &str) -> String {
    clean(url)
}
```

#### SQL Injection Prevention
**Use parameterized queries (sqlx does this automatically)**:
```rust
// ✅ SAFE: Parameterized query
sqlx::query!("SELECT * FROM tabs WHERE id = ?", tab_id)
    .fetch_one(&pool)
    .await?;

// ❌ DANGEROUS: String concatenation
// NEVER DO THIS:
// let query = format!("SELECT * FROM tabs WHERE id = '{}'", tab_id);
```

#### Credential Storage
**Use system keyring for sensitive data**:
```rust
use keyring::Entry;

pub fn store_proxy_credentials(proxy_id: &str, username: &str, password: &str) -> Result<()> {
    let entry = Entry::new("virtual-ip-browser", proxy_id)?;
    entry.set_password(&format!("{}:{}", username, password))?;
    Ok(())
}

pub fn get_proxy_credentials(proxy_id: &str) -> Result<(String, String)> {
    let entry = Entry::new("virtual-ip-browser", proxy_id)?;
    let creds = entry.get_password()?;
    let parts: Vec<&str> = creds.split(':').collect();
    Ok((parts[0].to_string(), parts[1].to_string()))
}
```

---

## 🧪 Testing Strategy

### Unit Testing Pattern
**Test individual functions and modules**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_proxy_validation() {
        let validator = ProxyValidator::new();
        let proxy = Proxy {
            host: "127.0.0.1".to_string(),
            port: 8080,
            protocol: ProxyProtocol::Http,
            username: None,
            password: None,
        };
        
        let result = validator.validate(&proxy).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_ip_generation() {
        let generator = IpGenerator::new();
        let ip = generator.generate();
        assert!(ip.octets()[0] > 0);
    }
}
```

### Integration Testing Pattern
**Test multi-module interactions**:
```rust
// crates/browser-core/tests/tab_lifecycle.rs
use browser_core::{BrowserTabManager, ProxyManager, Database};

#[tokio::test]
async fn test_tab_with_proxy() {
    let db = Database::new(":memory:").await.unwrap();
    let proxy_manager = ProxyManager::new(db.clone());
    let tab_manager = BrowserTabManager::new(db.clone(), proxy_manager);

    // Create tab
    let tab = tab_manager.create_tab("https://example.com").await.unwrap();
    assert!(tab.proxy_id.is_some());

    // Verify proxy assigned
    let proxy = proxy_manager.get_proxy(&tab.proxy_id.unwrap()).await.unwrap();
    assert!(proxy.is_some());

    // Close tab
    tab_manager.close_tab(&tab.id).await.unwrap();
}
```

### End-to-End Testing
**Test complete workflows from UI to backend**:
```typescript
// Using Playwright or similar
import { test, expect } from '@playwright/test';

test('create tab and navigate', async ({ page }) => {
    await page.goto('http://localhost:1420');
    
    // Create new tab
    await page.click('[data-testid="new-tab-button"]');
    await expect(page.locator('[data-testid="tab-list"]')).toContainText('New Tab');
    
    // Navigate to URL
    await page.fill('[data-testid="address-bar"]', 'https://example.com');
    await page.press('[data-testid="address-bar"]', 'Enter');
    
    // Verify navigation
    await expect(page.locator('[data-testid="active-tab"]')).toContainText('Example Domain');
});
```

---
