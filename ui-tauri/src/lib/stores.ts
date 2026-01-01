import { writable, derived, get, type Readable, type Writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/tauri';
import type { Tab, Country, ValidationResponse, WebviewTab } from './types';
import { createDebounce } from './utils';

// Request deduplication cache
const requestCache = new Map<string, Promise<unknown>>();
const CACHE_TTL = {
  tabs: 30000, // 30 seconds
  countries: 300000, // 5 minutes
  validation: 5000, // 5 seconds
};

// Enhanced store with request deduplication
function createDeduplicatedStore<T>(
  key: string,
  fetcher: () => Promise<T>,
  ttl: number = CACHE_TTL.tabs
): Writable<T[]> & { refresh: () => Promise<void>; loading: Readable<boolean> } {
  const store = writable<T[]>([]);
  const loading = writable(false);
  let lastFetch = 0;
  let currentRequest: Promise<T[]> | null = null;

  const refresh = async () => {
    const now = Date.now();
    
    // Return cached data if still valid
    if (now - lastFetch < ttl && get(store).length > 0) {
      return;
    }

    // Deduplicate concurrent requests
    const cacheKey = `fetch_${key}`;
    if (requestCache.has(cacheKey)) {
      await requestCache.get(cacheKey);
      return;
    }

    loading.set(true);
    
    try {
      const request = fetcher().then(data => {
        store.set(Array.isArray(data) ? data : [data]);
        lastFetch = now;
        requestCache.delete(cacheKey);
        return data;
      });

      requestCache.set(cacheKey, request);
      await request;
    } catch (error) {
      console.error(`Failed to fetch ${key}:`, error);
      throw error;
    } finally {
      loading.set(false);
    }
  };

  // Auto-refresh on first subscription
  let initialized = false;
  store.subscribe(() => {
    if (!initialized) {
      initialized = true;
      refresh();
    }
  });

  return {
    ...store,
    refresh,
    loading,
  };
}

// Tab store with optimized operations
export const tabsStore = createDeduplicatedStore(
  'tabs',
  () => invoke<Tab[]>('list_tabs'),
  CACHE_TTL.tabs
);

// Selected tab store with efficient updates
export const selectedTabStore = writable<Tab | null>(null);

// Derived store for selected tab index
export const selectedTabIndexStore = derived(
  [tabsStore, selectedTabStore],
  ([$tabs, $selectedTab]) => $tabs.findIndex(t => t.tab_id === $selectedTab?.tab_id)
);

// Countries store
export const countriesStore = createDeduplicatedStore(
  'countries',
  () => invoke<Country[]>('list_countries'),
  CACHE_TTL.countries
);

// Validation store with shorter TTL
export const validationStore = writable<ValidationResponse | null>(null);

// Optimized tab operations
export const tabOperations = {
  // Batch create tabs
  createBatch: async (countryCodes: string[]) => {
    const promises = countryCodes.map(code => invoke<Tab>('create_tab', { countryCode: code }));
    const newTabs = await Promise.all(promises);
    tabsStore.update(current => [...current, ...newTabs]);
    return newTabs;
  },

  // Efficient tab switching
  selectTab: (tabId: string) => {
    tabsStore.subscribe(tabs => {
      const tab = tabs.find(t => t.tab_id === tabId);
      if (tab) selectedTabStore.set(tab);
    })();
  },

  // Optimized tab removal
  removeTab: (tabId: string) => {
    tabsStore.update(tabs => {
      const filtered = tabs.filter(t => t.tab_id !== tabId);
      // Auto-select next tab if current was removed
      selectedTabStore.update(current => {
        if (current?.tab_id === tabId && filtered.length > 0) {
          return filtered[0];
        }
        return current;
      });
      return filtered;
    });
  },

  // Batch IP rotation
  rotateBatch: async (tabIds: string[]) => {
    const promises = tabIds.map(id => invoke<ValidationResponse>('rotate_ip', { tabId: id }));
    const results = await Promise.all(promises);
    
    // Refresh tabs to get updated IPs
    await tabsStore.refresh();
    
    return results;
  },
};

// Webview store with lazy initialization
class WebviewManager {
  private webviews: Map<string, WebviewTab> = new Map();
  private store = writable<WebviewTab[]>([]);
  
  get webviews$(): Readable<WebviewTab[]> {
    return this.store;
  }

  async createWebview(url?: string): Promise<WebviewTab> {
    const webview = await invoke<WebviewTab>('create_webview_tab', { url });
    this.webviews.set(webview.tab_id, webview);
    this.store.set(Array.from(this.webviews.values()));
    return webview;
  }

  async navigateWebview(tabId: string, url: string): Promise<void> {
    await invoke('navigate_webview_tab', { tabId, url });
    
    // Update local state optimistically
    const webview = this.webviews.get(tabId);
    if (webview) {
      webview.url = url;
      webview.is_loading = true;
      this.store.set(Array.from(this.webviews.values()));
    }
  }

  async closeWebview(tabId: string): Promise<void> {
    await invoke('close_webview_tab', { tabId });
    this.webviews.delete(tabId);
    this.store.set(Array.from(this.webviews.values()));
  }

  // Lazy cleanup of inactive webviews
  cleanupInactive(): void {
    const now = Date.now();
    const INACTIVE_THRESHOLD = 5 * 60 * 1000; // 5 minutes
    
    for (const [id, webview] of this.webviews) {
      const lastActivity = new Date(webview.created_at).getTime();
      if (now - lastActivity > INACTIVE_THRESHOLD) {
        this.closeWebview(id);
      }
    }
  }
}

export const webviewManager = new WebviewManager();

// Performance monitoring store
export const performanceStore = writable({
  apiCalls: 0,
  cacheHits: 0,
  averageResponseTime: 0,
});

// Enhanced API wrapper with performance tracking
const optimizedAPIImpl = {
  invoke: async <T>(command: string, args?: Record<string, unknown>): Promise<T> => {
    const start = performance.now();
    performanceStore.update(p => ({ ...p, apiCalls: p.apiCalls + 1 }));
    
    try {
      const result = await invoke<T>(command, args);
      const duration = performance.now() - start;
      
      performanceStore.update(p => ({
        ...p,
        averageResponseTime: (p.averageResponseTime + duration) / 2
      }));
      
      return result;
    } catch (error) {
      console.error(`API call failed: ${command}`, error);
      throw error;
    }
  },
};

export const optimizedAPI = {
  ...optimizedAPIImpl,
  
  // Batch multiple API calls
  batch: async <T>(calls: Array<{ command: string; args?: Record<string, unknown> }>): Promise<T[]> => {
    const promises = calls.map(call => optimizedAPIImpl.invoke<T>(call.command, call.args));
    return Promise.all(promises);
  },

  // Debounced API calls
  debounced: createDebounce(<T>(command: string, args?: Record<string, unknown>) => 
    optimizedAPIImpl.invoke<T>(command, args), 300),
};

// Memory usage monitoring
export const memoryStore = writable({
  webviewsCount: 0,
  tabsCount: 0,
  cacheSize: 0,
});

// Update memory stats periodically
setInterval(() => {
  webviewManager.webviews$.subscribe(webviews => {
    memoryStore.update(m => ({
      ...m,
      webviewsCount: webviews.length,
      cacheSize: requestCache.size,
    }));
  })();
  
  tabsStore.subscribe(tabs => {
    memoryStore.update(m => ({ ...m, tabsCount: tabs.length }));
  })();
}, 5000);

// Cleanup interval
setInterval(() => {
  // Clear expired cache entries
  const now = Date.now();
  for (const [key, promise] of requestCache) {
    // Clean up old cache entries
    if (key.startsWith('fetch_') && promise) {
      requestCache.delete(key);
    }
  }
  
  // Cleanup inactive webviews
  webviewManager.cleanupInactive();
}, 60000); // Every minute

// Legacy exports for backward compatibility
export const tabs = tabsStore;
export const countries = countriesStore;
export const selectedTab = selectedTabStore;
export const validationResult = validationStore;
export const loading = writable(false);
export const errorMessage = writable<string | null>(null);
