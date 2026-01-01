import { invoke } from '@tauri-apps/api/tauri';
import { logDebug } from './logger';
import type { 
  Tab, WebviewTab, VirtualIPResponse, ValidationResponse, Country,
  ProxySettings, FreeProxy, ProxyTestResult, PublicIpInfo,
  BackupOptions, BackupInfo, BrowserState, BrowserSettings, EnterpriseUserData,
  HistoryEntry, Bookmark, User, ProxyProviderConfig
} from './types';


import { apiCache } from './utils';
import { invokeWithRetry, errorHandler } from './errorHandling';

// Auth API stubs
export async function loginUser(credentials: { username: string; password: string }) {
  logDebug('Stub: loginUser', credentials);
  return { id: 'demo', username: credentials.username, token: 'stub-token' };
}

export async function registerUser(userData: { username: string; email: string; password: string }) {
  logDebug('Stub: registerUser', userData);
  return { id: 'demo', username: userData.username, email: userData.email };
}

export async function createEnterpriseUser(userData: EnterpriseUserData) {
  logDebug('Stub: createEnterpriseUser', userData);
  return { id: 'demo', ...userData };
}

export async function promoteUserToAdmin(userId: string) {
  logDebug('Stub: promoteUserToAdmin', userId);
  return { success: true };
}

export async function fetchUsers(): Promise<User[]> {
  logDebug('Fetching users');
  // In production, this would call the actual API endpoint
  // For now, return mock data for development
  return [
    {
      id: '1',
      username: 'admin',
      email: 'admin@company.com',
      role: 'Admin',
      created_at: new Date().toISOString(),
      last_login: new Date().toISOString(),
      enterprise_id: null
    },
    {
      id: '2',
      username: 'john_doe',
      email: 'john@company.com',
      role: 'User',
      created_at: new Date().toISOString(),
      last_login: new Date().toISOString(),
      enterprise_id: 'enterprise-123'
    }
  ];
}


// Tab Management (Legacy - for IP management)
export async function fetchTabs(): Promise<Tab[]> {
  const cacheKey = 'tabs';
  let tabs = apiCache.get<Tab[]>(cacheKey);
  if (tabs) return tabs;
  
  tabs = await invokeWithRetry('list_tabs', undefined, {
    maxAttempts: 3,
    baseDelay: 1000,
    retryCondition: (error) => error.name === 'NetworkError' || error.status >= 500
  });
  apiCache.set(cacheKey, tabs, 30000); // Cache for 30 seconds
  return tabs;
}

export async function fetchCountries(): Promise<Country[]> {
  const cacheKey = 'countries';
  let countries = apiCache.get<Country[]>(cacheKey);
  if (countries) return countries;
  
  countries = await invoke('list_countries');
  apiCache.set(cacheKey, countries, 300000); // Cache for 5 minutes
  return countries;
}

export async function createTab(countryCode: string): Promise<Tab> {
  const tab = await invoke('create_tab', { countryCode });
  apiCache.delete('tabs'); // Invalidate cache
  return tab;
}

export async function createTabRandom(): Promise<Tab> {
  const tab = await invoke('create_tab_random');
  apiCache.delete('tabs'); // Invalidate cache
  return tab;
}

// Webview Tab Management (Native browser windows)
export async function fetchWebviewTabs(): Promise<WebviewTab[]> {
  return invoke('get_webview_tabs');
}

export async function createWebviewTab(url?: string): Promise<WebviewTab> {
  return invoke('create_webview_tab', { url });
}

export async function navigateWebviewTab(tabId: string, url: string): Promise<void> {
  return invoke('navigate_webview_tab', { tabId, url });
}

export async function closeWebviewTab(tabId: string): Promise<void> {
  return invoke('close_webview_tab', { tabId });
}

export async function focusWebviewTab(tabId: string): Promise<void> {
  return invoke('focus_webview_tab', { tabId });
}


export async function rotateIp(tabId: string, newCountry?: string): Promise<VirtualIPResponse> {
  return invoke('rotate_ip', { tabId, newCountry: newCountry ?? null });
}

export async function validateIp(tabId: string): Promise<ValidationResponse> {
  return invoke('validate_ip', { tabId });
}

// Proxy Management
export async function getProxySettings(): Promise<ProxySettings> {
  return invoke('get_proxy_settings');
}

export async function setProxySettings(settings: ProxySettings): Promise<void> {
  return invoke('set_proxy_settings', { settings });
}

export async function getActiveProxy(): Promise<FreeProxy | null> {
  return invoke('get_active_proxy');
}

export async function setActiveProxy(proxy: FreeProxy | null): Promise<void> {
  return invoke('set_active_proxy', { proxy });
}

// Public IP Detection
export async function detectPublicIp(): Promise<PublicIpInfo> {
  return invoke('detect_public_ip');
}

// Free IP Providers
export async function fetchFreeProxies(): Promise<FreeProxy[]> {
  return invoke('fetch_free_proxies');
}

export async function getFreeProxies(): Promise<FreeProxy[]> {
  return invoke('get_free_proxies');
}

export async function testProxy(proxy: FreeProxy): Promise<ProxyTestResult> {
  return invoke('test_proxy', { proxy });
}

export async function clearFreeProxies(): Promise<void> {
  return invoke('clear_free_proxies');
}

export async function removeDeadProxies(): Promise<void> {
  return invoke('remove_dead_proxies');
}

// Backup & Restore
export async function createBackup(options: BackupOptions): Promise<BackupInfo> {
  return invoke('create_backup', { options });
}

export async function listBackups(): Promise<BackupInfo[]> {
  return invoke('list_backups');
}

export async function restoreBackup(path: string, password?: string): Promise<void> {
  return invoke('restore_backup', { path, password: password ?? null });
}

export async function deleteBackup(id: string): Promise<void> {
  return invoke('delete_backup', { id });
}

// Tab Close
export async function closeTab(tabId: string): Promise<void> {
  await invoke('close_tab', { tabId });
  apiCache.delete('tabs'); // Invalidate cache
}

// Browser Controls
export async function navigate(tabId: string, url: string): Promise<BrowserState> {
  return invoke('navigate', { tabId, url });
}

export async function goBack(tabId: string): Promise<string | null> {
  return invoke('go_back', { tabId });
}

export async function goForward(tabId: string): Promise<string | null> {
  return invoke('go_forward', { tabId });
}

export async function reloadPage(tabId: string): Promise<string | null> {
  return invoke('reload_page', { tabId });
}

export async function getBrowserState(tabId: string): Promise<BrowserState | null> {
  return invoke('get_browser_state', { tabId });
}

export async function updatePageTitle(tabId: string, title: string): Promise<void> {
  return invoke('update_page_title', { tabId, title });
}

export async function getBrowserSettings(): Promise<BrowserSettings> {
  const cacheKey = 'browser_settings';
  let settings = apiCache.get<BrowserSettings>(cacheKey);
  if (settings) return settings;
  
  settings = await invoke('get_browser_settings');
  apiCache.set(cacheKey, settings, 600000); // Cache for 10 minutes
  return settings;
}

export async function setBrowserSettings(settings: BrowserSettings): Promise<void> {
  await invoke('set_browser_settings', { settings });
  apiCache.delete('browser_settings'); // Clear cache after update
}

// History
export async function getHistory(limit: number = 100): Promise<HistoryEntry[]> {
  return invoke('get_history', { limit });
}

export async function searchHistory(query: string): Promise<HistoryEntry[]> {
  return invoke('search_history', { query });
}

export async function clearHistory(): Promise<void> {
  return invoke('clear_history');
}

// Bookmarks
export async function addBookmark(url: string, title: string, folder?: string): Promise<number> {
  return invoke('add_bookmark', { url, title, folder: folder ?? null });
}

export async function getBookmarks(): Promise<Bookmark[]> {
  return invoke('get_bookmarks');
}

export async function deleteBookmark(id: number): Promise<void> {
  return invoke('delete_bookmark', { id });
}

// Proxy Provider API Functions
export async function getProxyFromProvider(
  providerId: string, 
  settings?: Record<string, unknown>
): Promise<FreeProxy> {
  logDebug('Getting proxy from provider', { providerId, settings });
  
  // Get provider config from localStorage
  const stored = localStorage.getItem('proxy_providers');
  if (!stored) {
    throw new Error('No proxy providers configured');
  }
  
  const providers = JSON.parse(stored);
  const provider = providers.find((p: { id: string }) => p.id === providerId);
  
  if (!provider) {
    throw new Error('Provider not found');
  }
  
  if (!provider.enabled) {
    throw new Error('Provider is disabled');
  }
  
  // Generate proxy based on provider type
  if (provider.provider === 'iproyal') {
    return generateIPRoyalProxy(provider, settings);
  }
  
  throw new Error(`Unsupported provider: ${provider.provider}`);
}

function generateIPRoyalProxy(
  provider: { api_token: string; settings: { country?: string; session_type?: string; session_duration?: number; protocol?: string } },
  overrideSettings?: Record<string, unknown>
): FreeProxy {
  const settings = { ...provider.settings, ...overrideSettings };
  const country = (settings.country as string) || 'us';
  const sessionType = (settings.session_type as string) || 'rotating';
  const protocol = (settings.protocol as string) || 'http';
  
  // Build IPRoyal username with options
  let username = provider.api_token;
  username += `_country-${country}`;
  
  if (sessionType === 'sticky') {
    const sessionId = Math.random().toString(36).substring(2, 10);
    username += `_session-${sessionId}`;
    if (settings.session_duration) {
      username += `_lifetime-${settings.session_duration}m`;
    }
  }
  
  return {
    ip: 'geo.iproyal.com',
    port: 12321,
    protocol: protocol as 'http' | 'https' | 'socks4' | 'socks5' | 'direct',
    country: country.toUpperCase(),
    country_code: country.toUpperCase(),
    anonymity: 'elite',
    speed: 100,
    uptime: 99.9,
    last_checked: new Date().toISOString(),
    provider: 'iproyal',
    // Additional auth info stored in the proxy object
    // The backend will need to use username:password auth
  };
}

export async function testProxyProvider(providerId: string): Promise<{ success: boolean; message: string }> {
  logDebug('Testing proxy provider', { providerId });
  
  const stored = localStorage.getItem('proxy_providers');
  if (!stored) {
    return { success: false, message: 'No proxy providers configured' };
  }
  
  const providers = JSON.parse(stored);
  const provider = providers.find((p: { id: string }) => p.id === providerId);
  
  if (!provider) {
    return { success: false, message: 'Provider not found' };
  }
  
  // Test connection based on provider
  if (provider.provider === 'iproyal') {
    try {
      // Generate a test proxy and verify it works
      const proxy = generateIPRoyalProxy(provider);
      const testResult = await testProxy(proxy);
      
      if (testResult.success) {
        return { success: true, message: `Connected successfully. Response time: ${testResult.response_time_ms}ms` };
      } else {
        return { success: false, message: testResult.error || 'Connection failed' };
      }
    } catch (e) {
      return { 
        success: false, 
        message: e instanceof Error ? e.message : 'Connection test failed' 
      };
    }
  }
  
  return { success: false, message: 'Unsupported provider' };
}

export async function listProxyProviders(): Promise<ProxyProviderConfig[]> {
  const stored = localStorage.getItem('proxy_providers');
  if (!stored) {
    return [];
  }
  return JSON.parse(stored);
}

export async function saveProxyProvider(provider: ProxyProviderConfig): Promise<void> {
  const stored = localStorage.getItem('proxy_providers');
  let providers: ProxyProviderConfig[] = stored ? JSON.parse(stored) : [];
  
  const index = providers.findIndex(p => p.id === provider.id);
  if (index >= 0) {
    providers[index] = provider;
  } else {
    providers.push(provider);
  }
  
  localStorage.setItem('proxy_providers', JSON.stringify(providers));
}

export async function deleteProxyProvider(providerId: string): Promise<void> {
  const stored = localStorage.getItem('proxy_providers');
  if (!stored) return;
  
  let providers: ProxyProviderConfig[] = JSON.parse(stored);
  providers = providers.filter(p => p.id !== providerId);
  localStorage.setItem('proxy_providers', JSON.stringify(providers));
}
