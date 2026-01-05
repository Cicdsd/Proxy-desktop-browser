/**
 * Proxy Configuration Module
 * 
 * Provides functionality for exporting and importing proxy configurations including:
 * - Proxy list export/import
 * - Configuration backup
 * - Multiple format support (JSON, CSV)
 */

import { writable, get } from 'svelte/store';
import { persist } from './persist';

/**
 * Proxy configuration entry
 */
export interface ProxyConfig {
  id: string;
  name?: string;
  ip: string;
  port: number;
  protocol: 'http' | 'https' | 'socks4' | 'socks5';
  username?: string;
  password?: string;
  country?: string;
  countryCode?: string;
  city?: string;
  isActive: boolean;
  isWorking?: boolean;
  lastChecked?: string;
  responseTime?: number;
  tags?: string[];
  notes?: string;
}

/**
 * Proxy configuration group
 */
export interface ProxyGroup {
  id: string;
  name: string;
  description?: string;
  proxies: ProxyConfig[];
  createdAt: string;
  updatedAt: string;
}

/**
 * Export format options
 */
export type ExportFormat = 'json' | 'csv' | 'txt';

/**
 * Proxy configuration state
 */
interface ProxyConfigState {
  proxies: ProxyConfig[];
  groups: ProxyGroup[];
  activeProxyId?: string;
  rotationEnabled: boolean;
  rotationInterval: number; // in seconds
}

const defaultState: ProxyConfigState = {
  proxies: [],
  groups: [],
  activeProxyId: undefined,
  rotationEnabled: false,
  rotationInterval: 300,
};

// Persistent proxy config store
export const proxyConfigStore = persist(
  writable<ProxyConfigState>(defaultState),
  'virtual-ip-browser-proxy-config'
);

/**
 * Generate a unique proxy ID
 */
function generateProxyId(): string {
  return `proxy_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * Add a new proxy configuration
 */
export function addProxy(proxy: Omit<ProxyConfig, 'id'>): ProxyConfig {
  const newProxy: ProxyConfig = {
    ...proxy,
    id: generateProxyId(),
  };
  
  proxyConfigStore.update(state => ({
    ...state,
    proxies: [...state.proxies, newProxy],
  }));
  
  return newProxy;
}

/**
 * Update a proxy configuration
 */
export function updateProxy(proxyId: string, updates: Partial<ProxyConfig>): void {
  proxyConfigStore.update(state => ({
    ...state,
    proxies: state.proxies.map(p =>
      p.id === proxyId ? { ...p, ...updates } : p
    ),
  }));
}

/**
 * Delete a proxy configuration
 */
export function deleteProxy(proxyId: string): void {
  proxyConfigStore.update(state => ({
    ...state,
    proxies: state.proxies.filter(p => p.id !== proxyId),
    activeProxyId: state.activeProxyId === proxyId ? undefined : state.activeProxyId,
  }));
}

/**
 * Set the active proxy
 */
export function setActiveProxy(proxyId: string | undefined): void {
  proxyConfigStore.update(state => ({
    ...state,
    activeProxyId: proxyId,
  }));
}

/**
 * Get all proxies
 */
export function getProxies(): ProxyConfig[] {
  return get(proxyConfigStore).proxies;
}

/**
 * Get active proxy
 */
export function getActiveProxy(): ProxyConfig | undefined {
  const state = get(proxyConfigStore);
  return state.proxies.find(p => p.id === state.activeProxyId);
}

/**
 * Export proxies to JSON format
 */
export function exportProxiesToJson(proxies?: ProxyConfig[]): string {
  const state = get(proxyConfigStore);
  const exportData = {
    version: '1.0',
    exportedAt: new Date().toISOString(),
    proxies: proxies || state.proxies,
    groups: state.groups,
    settings: {
      rotationEnabled: state.rotationEnabled,
      rotationInterval: state.rotationInterval,
    },
  };
  return JSON.stringify(exportData, null, 2);
}

/**
 * Export proxies to CSV format
 */
export function exportProxiesToCsv(proxies?: ProxyConfig[]): string {
  const state = get(proxyConfigStore);
  const proxyList = proxies || state.proxies;
  
  const headers = ['ip', 'port', 'protocol', 'username', 'password', 'country', 'countryCode', 'city', 'tags', 'notes'];
  const csvLines = [headers.join(',')];
  
  for (const proxy of proxyList) {
    const values = [
      proxy.ip,
      proxy.port.toString(),
      proxy.protocol,
      proxy.username || '',
      proxy.password || '',
      proxy.country || '',
      proxy.countryCode || '',
      proxy.city || '',
      (proxy.tags || []).join(';'),
      (proxy.notes || '').replace(/,/g, ';'),
    ];
    csvLines.push(values.map(v => `"${v}"`).join(','));
  }
  
  return csvLines.join('\n');
}

/**
 * Export proxies to plain text format (ip:port)
 */
export function exportProxiesToTxt(proxies?: ProxyConfig[]): string {
  const state = get(proxyConfigStore);
  const proxyList = proxies || state.proxies;
  
  return proxyList.map(p => {
    if (p.username && p.password) {
      return `${p.protocol}://${p.username}:${p.password}@${p.ip}:${p.port}`;
    }
    return `${p.protocol}://${p.ip}:${p.port}`;
  }).join('\n');
}

/**
 * Export proxies to specified format
 */
export function exportProxies(format: ExportFormat, proxies?: ProxyConfig[]): string {
  switch (format) {
    case 'json':
      return exportProxiesToJson(proxies);
    case 'csv':
      return exportProxiesToCsv(proxies);
    case 'txt':
      return exportProxiesToTxt(proxies);
    default:
      return exportProxiesToJson(proxies);
  }
}

/**
 * Import proxies from JSON format
 */
export function importProxiesFromJson(json: string): ProxyConfig[] {
  const data = JSON.parse(json);
  const importedProxies: ProxyConfig[] = [];
  
  if (data.proxies && Array.isArray(data.proxies)) {
    for (const proxy of data.proxies) {
      const newProxy: ProxyConfig = {
        id: generateProxyId(),
        ip: proxy.ip,
        port: parseInt(proxy.port, 10),
        protocol: proxy.protocol || 'http',
        username: proxy.username,
        password: proxy.password,
        country: proxy.country,
        countryCode: proxy.countryCode,
        city: proxy.city,
        isActive: false,
        tags: proxy.tags,
        notes: proxy.notes,
      };
      importedProxies.push(newProxy);
    }
    
    proxyConfigStore.update(state => ({
      ...state,
      proxies: [...state.proxies, ...importedProxies],
    }));
  }
  
  return importedProxies;
}

/**
 * Import proxies from CSV format
 */
export function importProxiesFromCsv(csv: string): ProxyConfig[] {
  const lines = csv.trim().split('\n');
  if (lines.length < 2) return [];
  
  const headers = lines[0].split(',').map(h => h.trim().replace(/"/g, ''));
  const importedProxies: ProxyConfig[] = [];
  
  for (let i = 1; i < lines.length; i++) {
    const values = lines[i].split(',').map(v => v.trim().replace(/"/g, ''));
    const proxy: ProxyConfig = {
      id: generateProxyId(),
      ip: values[headers.indexOf('ip')] || '',
      port: parseInt(values[headers.indexOf('port')] || '0', 10),
      protocol: (values[headers.indexOf('protocol')] as ProxyConfig['protocol']) || 'http',
      username: values[headers.indexOf('username')] || undefined,
      password: values[headers.indexOf('password')] || undefined,
      country: values[headers.indexOf('country')] || undefined,
      countryCode: values[headers.indexOf('countryCode')] || undefined,
      city: values[headers.indexOf('city')] || undefined,
      isActive: false,
      tags: values[headers.indexOf('tags')]?.split(';').filter(Boolean) || [],
      notes: values[headers.indexOf('notes')] || undefined,
    };
    
    if (proxy.ip && proxy.port) {
      importedProxies.push(proxy);
    }
  }
  
  proxyConfigStore.update(state => ({
    ...state,
    proxies: [...state.proxies, ...importedProxies],
  }));
  
  return importedProxies;
}

/**
 * Import proxies from plain text format (ip:port per line)
 */
export function importProxiesFromTxt(txt: string): ProxyConfig[] {
  const lines = txt.trim().split('\n');
  const importedProxies: ProxyConfig[] = [];
  
  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    
    // Parse various formats: ip:port, protocol://ip:port, protocol://user:pass@ip:port
    let protocol: ProxyConfig['protocol'] = 'http';
    let ip = '';
    let port = 0;
    let username: string | undefined;
    let password: string | undefined;
    
    let remaining = trimmed;
    
    // Extract protocol
    const protocolMatch = remaining.match(/^(https?|socks[45]):\/\//i);
    if (protocolMatch) {
      protocol = protocolMatch[1].toLowerCase() as ProxyConfig['protocol'];
      remaining = remaining.slice(protocolMatch[0].length);
    }
    
    // Extract auth if present
    const authMatch = remaining.match(/^([^:]+):([^@]+)@/);
    if (authMatch) {
      username = authMatch[1];
      password = authMatch[2];
      remaining = remaining.slice(authMatch[0].length);
    }
    
    // Extract ip:port
    const hostMatch = remaining.match(/^([^:]+):(\d+)/);
    if (hostMatch) {
      ip = hostMatch[1];
      port = parseInt(hostMatch[2], 10);
    }
    
    if (ip && port) {
      importedProxies.push({
        id: generateProxyId(),
        ip,
        port,
        protocol,
        username,
        password,
        isActive: false,
      });
    }
  }
  
  proxyConfigStore.update(state => ({
    ...state,
    proxies: [...state.proxies, ...importedProxies],
  }));
  
  return importedProxies;
}

/**
 * Import proxies from specified format
 */
export function importProxies(data: string, format: ExportFormat): ProxyConfig[] {
  switch (format) {
    case 'json':
      return importProxiesFromJson(data);
    case 'csv':
      return importProxiesFromCsv(data);
    case 'txt':
      return importProxiesFromTxt(data);
    default:
      return importProxiesFromJson(data);
  }
}

/**
 * Create a proxy group
 */
export function createProxyGroup(name: string, description?: string, proxyIds?: string[]): ProxyGroup {
  const state = get(proxyConfigStore);
  const proxies = proxyIds 
    ? state.proxies.filter(p => proxyIds.includes(p.id))
    : [];
  
  const now = new Date().toISOString();
  const group: ProxyGroup = {
    id: `group_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
    name,
    description,
    proxies,
    createdAt: now,
    updatedAt: now,
  };
  
  proxyConfigStore.update(state => ({
    ...state,
    groups: [...state.groups, group],
  }));
  
  return group;
}

/**
 * Delete a proxy group
 */
export function deleteProxyGroup(groupId: string): void {
  proxyConfigStore.update(state => ({
    ...state,
    groups: state.groups.filter(g => g.id !== groupId),
  }));
}

/**
 * Clear all proxy configurations
 */
export function clearAllProxies(): void {
  proxyConfigStore.set(defaultState);
}

/**
 * Set rotation settings
 */
export function setRotationSettings(enabled: boolean, interval?: number): void {
  proxyConfigStore.update(state => ({
    ...state,
    rotationEnabled: enabled,
    rotationInterval: interval ?? state.rotationInterval,
  }));
}

/**
 * Get rotation settings
 */
export function getRotationSettings(): { enabled: boolean; interval: number } {
  const state = get(proxyConfigStore);
  return {
    enabled: state.rotationEnabled,
    interval: state.rotationInterval,
  };
}
