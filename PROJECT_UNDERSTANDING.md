# Proxy-Desktop-Browser Project Understanding

## Overview

**Proxy-Desktop-Browser** (also known as "Virtual IP Browser") is a privacy-focused desktop browser built with Rust and Tauri 2.0. It provides virtual IP routing and free proxy integration, allowing users to browse the web with enhanced privacy through tab isolation, proxy rotation, and fingerprint protection.

## Technology Stack

### Backend (Rust)
- **Framework**: Tauri 2.0 - For building cross-platform desktop applications
- **Async Runtime**: Tokio - High-performance async runtime
- **HTTP Client**: Reqwest with proxy support (SOCKS, HTTPS)
- **Database**: SQLx with SQLite - Async database operations
- **Web Scraping**: Scraper, Select - For parsing proxy lists
- **Browser Engine**: Chromiumoxide - Chromium browser automation
- **Web Framework**: Axum - For API server functionality

### Frontend (TypeScript/Svelte)
- **Framework**: Svelte with TypeScript
- **Build Tool**: Vite
- **Package Manager**: Bun (migrated from Node.js)

## Key Features

### 1. Core Browser Features
- Multi-tab browsing with complete isolation
- Navigation controls (back, forward, reload, home)
- Address bar with autocomplete
- Bookmark and history management
- Download manager and session save/restore

### 2. Privacy Features
- Tab Isolation: Complete cookie, storage, and cache isolation per tab
- Fingerprint Protection: Browser, Canvas, WebGL fingerprinting protection
- Virtual IP: Unique IP address per tab
- Proxy Rotation: 8+ rotation strategies
- WebRTC Leak Prevention and Timezone Spoofing

### 3. Proxy Features
- Integration with 8+ free proxy providers
- Automatic proxy validation and health monitoring
- Geographic-based rotation and domain-based sticky sessions
- SOCKS4, SOCKS5, HTTP, and HTTPS proxy support

### 4. Advanced Features (V1000)
- Memory Profiler, Error Recovery, Performance Optimizer
- Network Intelligence, Privacy Fortress
- Automation System with visual workflow builder
- Content Enhancement: Reader mode, media player, accessibility

### 5. Experimental Features
- Multi-engine system, Mesh proxy networking, Onion routing
- Zero-knowledge authentication, Post-quantum cryptography
- IPFS browser integration, AI/ML-powered features

## Project Structure
- crates/browser-core/ - Core browser functionality (25+ modules)
- crates/virtual-ip/ - Virtual IP management
- crates/api-server/ - REST API server
- ui-tauri/ - Svelte frontend with Tauri integration

## License: MIT OR Apache-2.0
