# Proxy System Codebase Understanding

This document provides a comprehensive understanding of the proxy system architecture in the Proxy-Desktop-Browser project.

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Core Components](#core-components)
4. [Proxy Types](#proxy-types)
5. [Rotation Strategies](#rotation-strategies)
6. [Free IP Providers](#free-ip-providers)
7. [Data Flow](#data-flow)
8. [API Reference](#api-reference)
9. [Configuration](#configuration)
10. [Best Practices](#best-practices)

---

## Overview

The proxy system is a critical component of the Proxy-Desktop-Browser that provides:

- **Privacy Protection**: Route traffic through proxy servers to hide real IP
- **Tab Isolation**: Each browser tab can have its own proxy configuration
- **Automatic Rotation**: Multiple strategies for rotating proxies
- **Free Proxy Integration**: Support for 6+ free proxy providers
- **Proxy Validation**: Health checking and dead proxy removal

---

## Architecture

The proxy system follows a layered architecture:

1. **UI Layer**: Svelte components for proxy configuration
2. **Tauri Bridge**: Commands to interact with Rust backend
3. **Core Layer**: ProxyManager, ProxyRotationManager, FreeIpProviderManager
4. **Engine Layer**: Chromium Engine with CDP Protocol

---

## Core Components

### 1. ProxyManager (proxy.rs)

The central manager for proxy settings and operations.

**Key Structures:**

- `ProxyManager`: Main struct holding settings, free_proxies list, and active_proxy
- `ProxySettings`: Configuration including proxy_type, host, port, auth, DNS servers, bypass list
- `FreeProxy`: Proxy info including ip, port, protocol, country, anonymity, speed, uptime, provider

**Key Methods:**

| Method | Description |
|--------|-------------|
| get_settings() | Get current proxy settings |
| set_settings() | Update proxy settings |
| get_free_proxies() | Get list of available free proxies |
| add_free_proxies() | Add proxies to the pool |
| set_active_proxy() | Set the active proxy for use |
| get_active_proxy() | Get the currently active proxy |
| remove_dead_proxies() | Remove non-working proxies |
| fetch_proxies() | Fetch proxies from providers |
| get_effective_proxy_url() | Get the proxy URL to use |

---

### 2. ProxyRotationManager (proxy_rotation.rs)

Manages automatic proxy rotation with multiple strategies.

**Key Structures:**

- `ProxyRotationManager`: Manages provider_manager, active_proxies, strategy, performance_metrics
- `ProxySession`: Tracks proxy, assigned_at, last_used, request_count, tab_id, domain_proxy_map
- `ProxyMetrics`: Records response_time_ms, success_rate, last_success, failures, total_requests

**Key Methods:**

| Method | Description |
|--------|-------------|
| get_proxy_for_tab() | Get or rotate proxy for a specific tab |
| assign_proxy_to_tab() | Assign a new proxy to a tab |
| release_tab() | Release a tab proxy session |
| force_rotate() | Force immediate proxy rotation |
| update_metrics() | Update proxy performance metrics |
| get_session_stats() | Get statistics for a tab session |
| update_strategy() | Change the rotation strategy |

---

### 3. FreeIpProviderManager (free_ip_providers.rs)

Manages integration with free proxy providers.

**Supported Providers:**

| Provider | Type | Rate Limit | Description |
|----------|------|------------|-------------|
| ProxyScrape | API | 1s | Fast, reliable API |
| GeoNode | API | 2s | Geographic-focused |
| PubProxy | API | 1s | Simple REST API |
| FreeProxyList | Scrape | 5s | Web scraping |
| ProxyNova | Scrape | 5s | Web scraping |
| SpysOne | Scrape | 5s | Web scraping |

**Key Methods:**

| Method | Description |
|--------|-------------|
| fetch_from_provider() | Fetch proxies from specific provider |
| fetch_all() | Fetch from all providers |
| get_proxy_pool() | Get all fetched proxies |
| get_working_proxies() | Get only working proxies |
| get_random_working_proxy() | Get a random working proxy |
| validate_proxy() | Validate a single proxy |
| validate_all() | Validate all proxies in pool |

---

## Proxy Types

The system supports the following proxy protocols:

| Type | Description |
|------|-------------|
| Direct | No proxy (direct connection) |
| Http | HTTP proxy |
| Https | HTTPS proxy (HTTP CONNECT) |
| Socks4 | SOCKS4 proxy |
| Socks5 | SOCKS5 proxy (with auth support) |

**URL Format Examples:**

- HTTP: http://host:port
- HTTP with auth: http://user:pass@host:port
- SOCKS5: socks5://host:port
- SOCKS5 with auth: socks5://user:pass@host:port

---

## Rotation Strategies

The ProxyRotationStrategy enum defines how and when proxies are rotated:

| Strategy | Description | Auto-Rotate | Use Case |
|----------|-------------|-------------|----------|
| PerRequest(N) | Rotate after N requests | Yes | High anonymity |
| PerDuration(D) | Rotate after time period | Yes | Timed sessions |
| PerSession | Never rotate during session | No | Stable sessions |
| Random(p) | Random rotation (probability) | Yes | Unpredictable |
| Sticky(D) | Sticky with timeout | Yes | Session persistence |
| Geographic | By country codes | No | Geo-restricted |
| PerformanceBased | Use fastest proxies | Yes | Speed priority |
| RoundRobin | Sequential rotation | Yes | Fair distribution |
| DomainBased | Different proxy per domain | Per domain | Multi-site |
| Manual | User-triggered only | No | Full control |

---

## Free IP Providers

### Provider Integration Flow

1. Provider (API/Web) -> HTTP/Scrape Request -> Parse and Normalize
2. FreeProxy Object -> Validate Proxy -> Add to Pool

### ProxyFilter Options

| Filter | Description |
|--------|-------------|
| All | No filtering |
| ByCountry(list) | Filter by country |
| ByType(list) | Filter by protocol |
| WorkingOnly | Only validated proxies |

---

## Data Flow

### Request Flow with Proxy

1. User navigates to URL in Tab
2. Tab requests proxy from ProxyRotationManager
3. Check if rotation needed (based on strategy)
4. Return current session proxy OR get new proxy from FreeIpProviderManager
5. Configure Chromium with proxy settings
6. Make request through proxy
7. Update ProxyMetrics (success/failure)

### Proxy Validation Flow

1. Get proxy from pool
2. Make test request to IP check service
3. Check response (received? IP match? latency?)
4. Update proxy status (is_working, latency_ms, last_checked)
5. Remove from pool if failed validation

---

## API Reference

### Frontend API (api.ts)

- getProxyFromProvider(): Get proxy from configured provider
- testProxyProvider(): Test provider connection
- listProxyProviders(): List all configured providers
- saveProxyProvider(): Save provider configuration
- deleteProxyProvider(): Delete provider configuration

### Tauri Commands

- get_proxy_settings(): Get current proxy settings
- set_proxy_settings(): Set proxy settings
- fetch_free_proxies(): Fetch free proxies
- set_active_proxy(): Set active proxy
- test_proxy(): Test proxy connection

---

## Configuration

### Default Settings

- proxy_type: Direct (no proxy)
- dns_servers: 1.1.1.1, 8.8.8.8
- bypass_list: localhost, 127.0.0.1

### Provider Update Intervals

| Setting | Default | Description |
|---------|---------|-------------|
| update_interval | 300s (5 min) | How often to refresh proxy list |
| rate_limit (API) | 1-2s | Delay between API calls |
| rate_limit (Scrape) | 5s | Delay between scraping |
| validation_timeout | 10s | Timeout for proxy validation |

---

## Best Practices

### 1. Proxy Selection

- For anonymity: Use PerRequest(1) or Random strategy
- For speed: Use PerformanceBased strategy
- For geo-restrictions: Use Geographic with target countries
- For stable sessions: Use PerSession or Sticky strategy

### 2. Error Handling

- Always handle proxy failures gracefully
- Fall back to direct connection if no proxy available

### 3. Validation

- Validate proxies before adding to active rotation
- Remove dead proxies regularly with remove_dead_proxies()
- Monitor success rates in ProxyMetrics

### 4. Rate Limiting

- Respect provider rate limits to avoid bans
- Use exponential backoff on failures
- Spread requests across multiple providers

### 5. Security

- Use HTTPS proxies for sensitive data
- Prefer SOCKS5 for better protocol support
- Verify proxy anonymity level before use

---

## File Reference

| File | Purpose |
|------|---------|
| proxy.rs | Core proxy types and ProxyManager |
| proxy_rotation.rs | Rotation strategies and session management |
| proxy_validator.rs | Proxy health checking |
| free_ip_providers.rs | Free proxy provider integrations |
| local_proxy.rs | Local proxy server |
| pac_server.rs | PAC (Proxy Auto-Config) file server |
| scraper_util.rs | Web scraping utilities for proxy lists |
| http_client.rs | HTTP client with proxy support |

---

## Summary

The proxy system in Proxy-Desktop-Browser provides a comprehensive solution for:

1. **Flexible Configuration**: Support for multiple proxy types and authentication
2. **Automatic Management**: Smart rotation strategies based on use case
3. **Free Proxy Integration**: Built-in support for 6+ free proxy providers
4. **Performance Tracking**: Metrics collection for informed proxy selection
5. **Tab Isolation**: Per-tab proxy assignment for maximum privacy

This architecture enables users to browse privately and securely with minimal configuration while providing power users with fine-grained control over their proxy setup.
