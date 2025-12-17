<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import type { WebviewTab, FreeProxy, PublicIpInfo, HistoryEntry, Bookmark, BrowserSettings } from '../../lib/types';
  
  interface BrowserTab extends WebviewTab {
    favicon?: string;
    isPinned?: boolean;
    isMuted?: boolean;
  }
  
  // Core state with Svelte 5 runes
  let tabs = $state<BrowserTab[]>([]);
  let activeTabId = $state<string | null>(null);
  let urlInput = $state('');
  let isLoading = $state(false);
  
  // Panel visibility
  let showProxyPanel = $state(false);
  let showBookmarksPanel = $state(false);
  let showHistoryPanel = $state(false);
  let showDownloadsPanel = $state(false);
  let showSettingsPanel = $state(false);
  let showTabSearch = $state(false);
  
  // Proxy state
  let currentProxy = $state<FreeProxy | null>(null);
  let publicIp = $state<PublicIpInfo | null>(null);
  let proxies = $state<FreeProxy[]>([]);
  let fetchingProxies = $state(false);
  let rotationStrategy = $state('round_robin');
  let proxyStats = $state<any>(null);
  let validatingProxy = $state(false);
  
  // Browser data
  let bookmarks = $state<Bookmark[]>([]);
  let history = $state<HistoryEntry[]>([]);
  let browserSettings = $state<BrowserSettings | null>(null);
  let recentlyClosedTabs = $state<BrowserTab[]>([]);
  
  // Search
  let tabSearchQuery = $state('');
  let historySearchQuery = $state('');
  
  // Derived state with Svelte 5 runes
  let activeTab = $derived(() => tabs.find(t => t.tab_id === activeTabId) || null);
  let filteredTabs = $derived(() => tabSearchQuery 
    ? tabs.filter(t => 
        t.title?.toLowerCase().includes(tabSearchQuery.toLowerCase()) ||
        t.url?.toLowerCase().includes(tabSearchQuery.toLowerCase())
      )
    : tabs);
  let filteredHistory = $derived(() => historySearchQuery
    ? history.filter(h =>
        h.title?.toLowerCase().includes(historySearchQuery.toLowerCase()) ||
        h.url?.toLowerCase().includes(historySearchQuery.toLowerCase())
      )
    : history);
  
  // Effect for updating URL when active tab changes
  $effect(() => {
    if (activeTab()) {
      urlInput = activeTab().url;
    }
  });
  
  // Effect for component lifecycle
  $effect(() => {
    // Component mounted
    loadTabs();
    loadProxies();
    detectIp();
    
    // Listen for navigation changes from backend
    const unlistenNav = listen('navigation_changed', (event: any) => {
      const { tab_id, url, title } = event.payload;
      updateTabState(tab_id, { url, title });
    });
    
    const unlistenTitle = listen('title_changed', (event: any) => {
      const { tab_id, title } = event.payload;
      updateTabState(tab_id, { title });
    });
    
    window.addEventListener('keydown', handleGlobalKeydown);
    
    // Cleanup function
    return () => {
      unlistenNav.then(fn => fn());
      unlistenTitle.then(fn => fn());
      window.removeEventListener('keydown', handleGlobalKeydown);
    };
  });
  
  function updateTabState(tabId: string, updates: Partial<BrowserTab>) {
    const index = tabs.findIndex(t => t.tab_id === tabId);
    if (index !== -1) {
      tabs[index] = { ...tabs[index], ...updates };
      tabs = [...tabs];
    }
  }
  
  async function loadTabs() {
    try {
      const result = await invoke<WebviewTab[]>('get_webview_tabs');
      tabs = result || [];
      if (tabs.length > 0 && !activeTabId) {
        activeTabId = tabs[0].tab_id;
      }
    } catch (e) {
      console.error('Failed to load tabs:', e);
    }
  }
  
  async function loadProxies() {
    try {
      proxies = await invoke<FreeProxy[]>('get_free_proxies');
      currentProxy = await invoke<FreeProxy | null>('get_active_proxy');
    } catch (e) {
      console.error('Failed to load proxies:', e);
    }
  }
  
  async function detectIp() {
    try {
      publicIp = await invoke<PublicIpInfo>('detect_public_ip');
    } catch (e) {
      console.error('Failed to detect IP:', e);
    }
  }
  
  async function fetchNewProxies() {
    fetchingProxies = true;
    try {
      await invoke('fetch_free_proxies');
      await loadProxies();
    } catch (e) {
      console.error('Failed to fetch proxies:', e);
    } finally {
      fetchingProxies = false;
    }
  }
  
  async function createNewTab(withProxy = true) {
    isLoading = true;
    try {
      let newTab: WebviewTab;
      if (withProxy && currentProxy) {
        const proxyUrl = `${currentProxy.protocol}://${currentProxy.ip}:${currentProxy.port}`;
        newTab = await invoke('create_webview_tab_with_proxy', { 
          url: 'https://www.google.com',
          proxyUrl
        });
      } else {
        newTab = await invoke('create_webview_tab', { url: 'https://www.google.com' });
      }
      tabs = [...tabs, newTab];
      activeTabId = newTab.tab_id;
    } catch (e) {
      console.error('Failed to create tab:', e);
    } finally {
      isLoading = false;
    }
  }
  
  async function closeTab(tabId: string, e?: MouseEvent) {
    e?.stopPropagation();
    try {
      await invoke('close_webview_tab', { tabId });
      tabs = tabs.filter(t => t.tab_id !== tabId);
      if (activeTabId === tabId) {
        activeTabId = tabs.length > 0 ? tabs[0].tab_id : null;
      }
    } catch (e) {
      console.error('Failed to close tab:', e);
    }
  }
  
  async function selectTab(tabId: string) {
    activeTabId = tabId;
    try {
      await invoke('focus_webview_tab', { tabId });
    } catch (e) {
      console.error('Failed to focus tab:', e);
    }
  }
  
  async function navigate() {
    if (!activeTab || !urlInput.trim()) return;
    
    let url = urlInput.trim();
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      if (url.includes('.') && !url.includes(' ')) {
        url = 'https://' + url;
      } else {
        url = `https://www.google.com/search?q=${encodeURIComponent(url)}`;
      }
    }
    
    isLoading = true;
    try {
      await invoke('navigate_webview_tab', { tabId: activeTab.tab_id, url });
      updateTabState(activeTab.tab_id, { url, is_loading: true });
      urlInput = url;
    } catch (e) {
      console.error('Failed to navigate:', e);
    } finally {
      isLoading = false;
    }
  }
  
  async function goBack() {
    if (!activeTab) return;
    try {
      await invoke('go_back', { tabId: activeTab.tab_id });
    } catch (e) {
      console.error('Go back failed:', e);
    }
  }
  
  async function goForward() {
    if (!activeTab) return;
    try {
      await invoke('go_forward', { tabId: activeTab.tab_id });
    } catch (e) {
      console.error('Go forward failed:', e);
    }
  }
  
  async function refresh() {
    if (!activeTab) return;
    try {
      await invoke('reload_page', { tabId: activeTab.tab_id });
    } catch (e) {
      console.error('Refresh failed:', e);
    }
  }
  
  async function setProxy(proxy: FreeProxy | null) {
    try {
      await invoke('set_active_proxy', { proxy });
      currentProxy = proxy;
      showProxyPanel = false;
      await detectIp();
    } catch (e) {
      console.error('Failed to set proxy:', e);
    }
  }
  
  async function rotateProxy() {
    if (!activeTabId) return;
    try {
      const newProxy = await invoke<FreeProxy | null>('rotate_proxy_for_tab', { tabId: activeTabId });
      if (newProxy) {
        currentProxy = newProxy;
        await detectIp();
        await loadProxyStats();
      }
    } catch (e) {
      console.error('Failed to rotate proxy:', e);
    }
  }
  
  async function setRotationStrategy(strategy: string, params?: any) {
    try {
      await invoke('update_rotation_strategy', { strategy, params });
      rotationStrategy = strategy;
    } catch (e) {
      console.error('Failed to update rotation strategy:', e);
    }
  }
  
  async function loadProxyStats() {
    if (!activeTabId) return;
    try {
      proxyStats = await invoke('get_proxy_session_stats', { tabId: activeTabId });
    } catch (e) {
      console.error('Failed to load proxy stats:', e);
    }
  }
  
  async function validateProxy(proxy: FreeProxy) {
    validatingProxy = proxy.ip;
    try {
      const result = await invoke('test_proxy', { proxy });
      // Update proxy status in the list
      const index = proxies.findIndex(p => p.ip === proxy.ip && p.port === proxy.port);
      if (index !== -1) {
        proxies[index].is_working = result.is_working;
        proxies[index].speed = result.response_time_ms;
        proxies = [...proxies];
      }
    } catch (e) {
      console.error('Failed to validate proxy:', e);
    } finally {
      validatingProxy = '';
    }
  }
  
  async function fetchProxiesFromProvider(provider: string) {
    fetchingProxies = true;
    try {
      const newProxies = await invoke<FreeProxy[]>('fetch_proxies_from_provider', { providerName: provider });
      // Merge with existing proxies, removing duplicates
      const existing = new Set(proxies.map(p => `${p.ip}:${p.port}`));
      const unique = newProxies.filter(p => !existing.has(`${p.ip}:${p.port}`));
      proxies = [...proxies, ...unique];
    } catch (e) {
      console.error(`Failed to fetch proxies from ${provider}:`, e);
    } finally {
      fetchingProxies = false;
    }
  }
  
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') navigate();
  }
  
  function getCountryFlag(code: string): string {
    if (!code || code.length !== 2) return '🌐';
    const codePoints = code
      .toUpperCase()
      .split('')
      .map(char => 127397 + char.charCodeAt(0));
    return String.fromCodePoint(...codePoints);
  }
  
  // Bookmark functions
  async function loadBookmarks() {
    try {
      bookmarks = await invoke<Bookmark[]>('get_bookmarks');
    } catch (e) {
      console.error('Failed to load bookmarks:', e);
    }
  }
  
  async function addBookmark() {
    if (!activeTab) return;
    try {
      await invoke('add_bookmark', { 
        url: activeTab.url, 
        title: activeTab.title || activeTab.url,
        folder: null
      });
      await loadBookmarks();
    } catch (e) {
      console.error('Failed to add bookmark:', e);
    }
  }
  
  async function deleteBookmark(id: number) {
    try {
      await invoke('delete_bookmark', { id });
      await loadBookmarks();
    } catch (e) {
      console.error('Failed to delete bookmark:', e);
    }
  }
  
  async function openBookmark(url: string) {
    if (activeTab) {
      urlInput = url;
      await navigate();
    } else {
      await createNewTab(false);
      urlInput = url;
      await navigate();
    }
    showBookmarksPanel = false;
  }
  
  // History functions
  async function loadHistory() {
    try {
      history = await invoke<HistoryEntry[]>('get_history', { limit: 100 });
    } catch (e) {
      console.error('Failed to load history:', e);
    }
  }
  
  async function clearHistory() {
    try {
      await invoke('clear_history');
      history = [];
    } catch (e) {
      console.error('Failed to clear history:', e);
    }
  }
  
  async function openHistoryEntry(url: string) {
    if (activeTab) {
      urlInput = url;
      await navigate();
    }
    showHistoryPanel = false;
  }
  
  // Settings functions
  async function loadSettings() {
    try {
      browserSettings = await invoke<BrowserSettings>('get_browser_settings');
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }
  
  async function saveSettings() {
    if (!browserSettings) return;
    try {
      await invoke('set_browser_settings', { settings: browserSettings });
    } catch (e) {
      console.error('Failed to save settings:', e);
    }
  }
  
  // Tab management
  function pinTab(tabId: string) {
    const index = tabs.findIndex(t => t.tab_id === tabId);
    if (index !== -1) {
      tabs[index].isPinned = !tabs[index].isPinned;
      // Move pinned tabs to the front
      tabs = [...tabs.filter(t => t.isPinned), ...tabs.filter(t => !t.isPinned)];
    }
  }
  
  function muteTab(tabId: string) {
    const index = tabs.findIndex(t => t.tab_id === tabId);
    if (index !== -1) {
      tabs[index].isMuted = !tabs[index].isMuted;
      tabs = [...tabs];
    }
  }
  
  async function duplicateTab(tabId: string) {
    const tab = tabs.find(t => t.tab_id === tabId);
    if (tab) {
      await createNewTab(!!currentProxy);
      if (activeTab) {
        urlInput = tab.url;
        await navigate();
      }
    }
  }
  
  async function reopenClosedTab() {
    if (recentlyClosedTabs.length > 0) {
      const tab = recentlyClosedTabs.pop();
      if (tab) {
        await createNewTab(!!currentProxy);
        urlInput = tab.url;
        await navigate();
      }
    }
  }
  
  // Close all panels
  function closeAllPanels() {
    showProxyPanel = false;
    showBookmarksPanel = false;
    showHistoryPanel = false;
    showDownloadsPanel = false;
    showSettingsPanel = false;
    showTabSearch = false;
  }
  
  function togglePanel(panel: string) {
    closeAllPanels();
    switch (panel) {
      case 'proxy': showProxyPanel = true; break;
      case 'bookmarks': showBookmarksPanel = true; loadBookmarks(); break;
      case 'history': showHistoryPanel = true; loadHistory(); break;
      case 'downloads': showDownloadsPanel = true; break;
      case 'settings': showSettingsPanel = true; loadSettings(); break;
      case 'tabSearch': showTabSearch = true; break;
    }
  }
  
  // Keyboard shortcuts
  function handleGlobalKeydown(e: KeyboardEvent) {
    // Ctrl+T: New tab
    if (e.ctrlKey && e.key === 't') {
      e.preventDefault();
      createNewTab();
    }
    // Ctrl+W: Close tab
    if (e.ctrlKey && e.key === 'w' && activeTab) {
      e.preventDefault();
      closeTab(activeTab.tab_id);
    }
    // Ctrl+Shift+T: Reopen closed tab
    if (e.ctrlKey && e.shiftKey && e.key === 'T') {
      e.preventDefault();
      reopenClosedTab();
    }
    // Ctrl+L: Focus URL bar
    if (e.ctrlKey && e.key === 'l') {
      e.preventDefault();
      document.querySelector<HTMLInputElement>('.url-input')?.focus();
    }
    // Ctrl+Shift+A: Tab search
    if (e.ctrlKey && e.shiftKey && e.key === 'A') {
      e.preventDefault();
      togglePanel('tabSearch');
    }
    // Ctrl+H: History
    if (e.ctrlKey && e.key === 'h') {
      e.preventDefault();
      togglePanel('history');
    }
    // Ctrl+B: Bookmarks
    if (e.ctrlKey && e.key === 'b') {
      e.preventDefault();
      togglePanel('bookmarks');
    }
    // Ctrl+D: Add bookmark
    if (e.ctrlKey && e.key === 'd') {
      e.preventDefault();
      addBookmark();
    }
    // Escape: Close panels
    if (e.key === 'Escape') {
      closeAllPanels();
    }
  }
  
    
  function formatTime(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  }
</script>

<div class="browser-shell">
  <!-- Tab Bar -->
  <div class="tab-bar">
    <div class="tabs-container">
      {#each tabs as tab (tab.tab_id)}
        <div 
          class="tab"
          class:active={tab.tab_id === activeTabId}
          onclick={() => selectTab(tab.tab_id)}
          onkeydown={(e) => e.key === 'Enter' && selectTab(tab.tab_id)}
          role="button"
          tabindex="0"
          title={tab.title || tab.url}
        >
          <span class="tab-icon">
            {#if tab.is_loading}
              <div class="spinner"></div>
            {:else}
              🌐
            {/if}
          </span>
          <span class="tab-title">{tab.title || 'New Tab'}</span>
          <button class="tab-close" onclick={(e) => closeTab(tab.tab_id, e)}>✕</button>
        </div>
      {/each}
      
      <button class="new-tab-btn" onclick={() => createNewTab()} title="New Tab (Ctrl+T)">
        <span>+</span>
      </button>
    </div>
    
    <div class="window-controls">
      <button class="win-btn minimize">─</button>
      <button class="win-btn maximize">□</button>
      <button class="win-btn close">✕</button>
    </div>
  </div>
  
  <!-- Navigation Bar -->
  <div class="nav-bar">
    <div class="nav-buttons">
      <button 
        class="nav-btn" 
        onclick={goBack}
        disabled={!activeTab()?.can_go_back}
        title="Go Back (Alt+Left)"
      >
        <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
          <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
        </svg>
      </button>
      <button 
        class="nav-btn" 
        onclick={goForward}
        disabled={!activeTab()?.can_go_forward}
        title="Go Forward (Alt+Right)"
      >
        <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
          <path d="M12 4l-1.41 1.41L16.17 11H4v2h12.17l-5.58 5.59L12 20l8-8z"/>
        </svg>
      </button>
      <button 
        class="nav-btn" 
        onclick={refresh}
        disabled={!activeTab()}
        title="Refresh (Ctrl+R)"
      >
        <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor" class:spinning={isLoading}>
          <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
        </svg>
      </button>
    </div>
    
    <div class="url-bar">
      <div class="security-indicator" class:secure={urlInput.startsWith('https://')}>
        {#if urlInput.startsWith('https://')}
          <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
            <path d="M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zm-6 9c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2zm3.1-9H8.9V6c0-1.71 1.39-3.1 3.1-3.1 1.71 0 3.1 1.39 3.1 3.1v2z"/>
          </svg>
        {:else}
          <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/>
          </svg>
        {/if}
      </div>
      <input
        type="text"
        bind:value={urlInput}
        onkeydown={handleKeydown}
        placeholder="Search or enter URL"
        class="url-input"
      />
    </div>
    
    <div class="nav-actions">
      <!-- Bookmark Button -->
      <button 
        class="action-btn icon-btn"
        onclick={addBookmark}
        title="Add Bookmark (Ctrl+D)"
      >
        <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
          <path d="M17 3H7c-1.1 0-1.99.9-1.99 2L5 21l7-3 7 3V5c0-1.1-.9-2-2-2z"/>
        </svg>
      </button>
      
      <!-- Proxy Status Button -->
      <button 
        class="action-btn proxy-btn"
        class:active={currentProxy !== null}
        onclick={() => togglePanel('proxy')}
        title="Proxy Settings"
      >
        {#if currentProxy}
          <span class="proxy-flag">{getCountryFlag(currentProxy.country_code)}</span>
          <span class="proxy-ip">{currentProxy.ip}</span>
        {:else}
          <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/>
          </svg>
          <span>Direct</span>
        {/if}
      </button>
      
      <!-- Menu Button -->
      <button 
        class="action-btn icon-btn menu-btn"
        onclick={() => togglePanel('settings')}
        title="Menu"
      >
        <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
          <path d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z"/>
        </svg>
      </button>
    </div>
  </div>
  
  <!-- Bookmarks Bar -->
  <div class="bookmarks-bar">
    <button class="bookmarks-toggle" onclick={() => togglePanel('bookmarks')} title="Bookmarks (Ctrl+B)">
      <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
        <path d="M17 3H7c-1.1 0-1.99.9-1.99 2L5 21l7-3 7 3V5c0-1.1-.9-2-2-2z"/>
      </svg>
      Bookmarks
    </button>
    <div class="bookmark-items">
      {#each bookmarks.slice(0, 8) as bookmark}
        <button class="bookmark-item" onclick={() => openBookmark(bookmark.url)} title={bookmark.url}>
          🌐 {bookmark.title.slice(0, 20)}{bookmark.title.length > 20 ? '...' : ''}
        </button>
      {/each}
    </div>
    <div class="toolbar-spacer"></div>
    <button class="toolbar-btn" onclick={() => togglePanel('history')} title="History (Ctrl+H)">
      <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
        <path d="M13 3c-4.97 0-9 4.03-9 9H1l3.89 3.89.07.14L9 12H6c0-3.87 3.13-7 7-7s7 3.13 7 7-3.13 7-7 7c-1.93 0-3.68-.79-4.94-2.06l-1.42 1.42C8.27 19.99 10.51 21 13 21c4.97 0 9-4.03 9-9s-4.03-9-9-9zm-1 5v5l4.28 2.54.72-1.21-3.5-2.08V8H12z"/>
      </svg>
    </button>
    <button class="toolbar-btn" onclick={() => togglePanel('downloads')} title="Downloads">
      <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
        <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
      </svg>
    </button>
  </div>
  
  <!-- Proxy Panel Dropdown -->
  {#if showProxyPanel}
    <div class="proxy-panel">
      <div class="proxy-panel-header">
        <h3>Proxy Configuration</h3>
        <button class="close-panel" onclick={() => showProxyPanel = false}>✕</button>
      </div>
      
      <div class="current-ip">
        <span class="label">Your IP:</span>
        <span class="value">{publicIp?.ip || 'Detecting...'}</span>
        {#if publicIp?.country_code}
          <span class="flag">{getCountryFlag(publicIp.country_code)}</span>
        {/if}
      </div>
      
      <!-- Rotation Strategy -->
      <div class="rotation-section">
        <h4>Rotation Strategy</h4>
        <select bind:value={rotationStrategy} onchange={(e) => setRotationStrategy(e.target.value)} class="strategy-select">
          <option value="round_robin">Round Robin</option>
          <option value="per_request">Per Request</option>
          <option value="per_duration">Per Duration</option>
          <option value="per_session">Per Session</option>
          <option value="random">Random</option>
          <option value="sticky">Sticky</option>
          <option value="geographic">Geographic</option>
          <option value="performance_based">Performance Based</option>
          <option value="domain_based">Domain Based</option>
          <option value="manual">Manual</option>
        </select>
        <button class="btn-rotate" onclick={rotateProxy} disabled={!activeTabId}>
          🔄 Rotate Now
        </button>
      </div>
      
      <!-- Proxy Stats -->
      {#if proxyStats}
        <div class="proxy-stats">
          <div class="stat-item">
            <span class="label">Current Proxy:</span>
            <span class="value">{proxyStats.current_proxy_ip}</span>
          </div>
          <div class="stat-item">
            <span class="label">Country:</span>
            <span class="value">{proxyStats.proxy_country}</span>
          </div>
          <div class="stat-item">
            <span class="label">Requests:</span>
            <span class="value">{proxyStats.request_count}</span>
          </div>
          <div class="stat-item">
            <span class="label">Duration:</span>
            <span class="value">{Math.floor(proxyStats.duration_seconds / 60)}m {proxyStats.duration_seconds % 60}s</span>
          </div>
        </div>
      {/if}
      
      <!-- Provider Fetching -->
      <div class="provider-section">
        <h4>Fetch from Provider</h4>
        <div class="provider-buttons">
          {#each ['ProxyScrape', 'GeoNode', 'PubProxy', 'FreeProxyList', 'ProxyNova', 'SpysOne'] as provider}
            <button 
              class="btn-provider"
              onclick={() => fetchProxiesFromProvider(provider)}
              disabled={fetchingProxies}
            >
              {provider}
            </button>
          {/each}
        </div>
      </div>
      
      <div class="proxy-actions">
        <button class="btn-primary" onclick={fetchNewProxies} disabled={fetchingProxies}>
          {fetchingProxies ? 'Fetching...' : 'Fetch All Proxies'}
        </button>
        <button class="btn-secondary" onclick={() => setProxy(null)}>
          Direct Connection
        </button>
      </div>
      
      <div class="proxy-list">
        <h4>Available Proxies ({proxies.length})</h4>
        <div class="proxy-filters">
          <button class="filter-btn" onclick={() => proxies = proxies.sort((a, b) => (a.speed || 9999) - (b.speed || 9999))}>
            Sort by Speed
          </button>
          <button class="filter-btn" onclick={() => proxies = proxies.filter(p => p.is_working)}>
            Working Only
          </button>
          <button class="filter-btn" onclick={() => loadProxies()}>
            Reset
          </button>
        </div>
        <div class="proxy-items">
          {#each proxies.slice(0, 20) as proxy}
            <div 
              class="proxy-item"
              class:active={currentProxy?.ip === proxy.ip}
              class:working={proxy.is_working}
              class:failed={!proxy.is_working}
              onclick={() => setProxy(proxy)}
              onkeydown={(e) => e.key === 'Enter' && setProxy(proxy)}
              role="button"
              tabindex="0"
            >
              <span class="proxy-flag">{getCountryFlag(proxy.country_code)}</span>
              <span class="proxy-info">
                <span class="proxy-addr">{proxy.ip}:{proxy.port}</span>
                <span class="proxy-meta">{proxy.protocol} • {proxy.country}</span>
              </span>
              <div class="proxy-actions-right">
                {#if proxy.speed > 0}
                  <span class="proxy-speed">{proxy.speed}ms</span>
                {/if}
                <button 
                  class="validate-btn"
                  onclick={(e) => { e.stopPropagation(); validateProxy(proxy); }}
                  disabled={validatingProxy === proxy.ip}
                >
                  {validatingProxy === proxy.ip ? '⏳' : '✓'}
                </button>
              </div>
            </div>
          {/each}
          {#if proxies.length === 0}
            <div class="no-proxies">No proxies available. Click "Fetch Free Proxies" to load.</div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
  
  <!-- Bookmarks Panel -->
  {#if showBookmarksPanel}
    <div class="side-panel">
      <div class="panel-header">
        <h3>Bookmarks</h3>
        <button class="close-panel" onclick={() => showBookmarksPanel = false}>✕</button>
      </div>
      <div class="panel-content">
        {#if bookmarks.length === 0}
          <div class="empty-panel">No bookmarks yet. Press Ctrl+D to add one.</div>
        {:else}
          {#each bookmarks as bookmark}
            <div class="panel-item" onclick={() => openBookmark(bookmark.url)} onkeydown={(e) => e.key === 'Enter' && openBookmark(bookmark.url)} role="button" tabindex="0">
              <span class="item-icon">🔖</span>
              <span class="item-info">
                <span class="item-title">{bookmark.title}</span>
                <span class="item-url">{bookmark.url}</span>
              </span>
              <button class="item-delete" onclick={(e) => { e.stopPropagation(); deleteBookmark(bookmark.id); }}>✕</button>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  {/if}
  
  <!-- History Panel -->
  {#if showHistoryPanel}
    <div class="side-panel">
      <div class="panel-header">
        <h3>History</h3>
        <button class="close-panel" onclick={() => showHistoryPanel = false}>✕</button>
      </div>
      <div class="panel-search">
        <input type="text" bind:value={historySearchQuery} placeholder="Search history..." />
      </div>
      <div class="panel-actions">
        <button class="btn-danger" onclick={clearHistory}>Clear All History</button>
      </div>
      <div class="panel-content">
        {#if filteredHistory.length === 0}
          <div class="empty-panel">No history yet.</div>
        {:else}
          {#each filteredHistory as entry}
            <button class="panel-item" onclick={() => openHistoryEntry(entry.url)}>
              <span class="item-icon">🕐</span>
              <span class="item-info">
                <span class="item-title">{entry.title || entry.url}</span>
                <span class="item-meta">{formatTime(entry.last_visit)} • {entry.visit_count} visits</span>
              </span>
            </button>
          {/each}
        {/if}
      </div>
    </div>
  {/if}
  
  <!-- Downloads Panel -->
  {#if showDownloadsPanel}
    <div class="side-panel">
      <div class="panel-header">
        <h3>Downloads</h3>
        <button class="close-panel" onclick={() => showDownloadsPanel = false}>✕</button>
      </div>
      <div class="panel-content">
        <div class="empty-panel">No downloads yet.</div>
      </div>
    </div>
  {/if}
  
  <!-- Settings Panel -->
  {#if showSettingsPanel}
    <div class="side-panel settings-panel">
      <div class="panel-header">
        <h3>Settings</h3>
        <button class="close-panel" onclick={() => showSettingsPanel = false}>✕</button>
      </div>
      <div class="panel-content">
        <div class="settings-section">
          <h4>Privacy & Security</h4>
          {#if browserSettings}
            <label class="setting-item">
              <span>Block Trackers</span>
              <input type="checkbox" bind:checked={browserSettings.block_trackers} onchange={saveSettings} />
            </label>
            <label class="setting-item">
              <span>Block Ads</span>
              <input type="checkbox" bind:checked={browserSettings.block_ads} onchange={saveSettings} />
            </label>
            <label class="setting-item">
              <span>DNS over HTTPS</span>
              <input type="checkbox" bind:checked={browserSettings.dns_over_https} onchange={saveSettings} />
            </label>
            <label class="setting-item">
              <span>JavaScript Enabled</span>
              <input type="checkbox" bind:checked={browserSettings.javascript_enabled} onchange={saveSettings} />
            </label>
            <label class="setting-item">
              <span>Cookies Enabled</span>
              <input type="checkbox" bind:checked={browserSettings.cookies_enabled} onchange={saveSettings} />
            </label>
          {/if}
        </div>
        <div class="settings-section">
          <h4>WebRTC Policy</h4>
          {#if browserSettings}
            <select bind:value={browserSettings.webrtc_policy} onchange={saveSettings} class="setting-select">
              <option value="default">Default</option>
              <option value="disable_non_proxied_udp">Disable Non-Proxied UDP</option>
              <option value="disabled">Disabled</option>
            </select>
          {/if}
        </div>
        <div class="settings-section">
          <h4>Keyboard Shortcuts</h4>
          <div class="shortcuts-list">
            <div class="shortcut"><kbd>Ctrl+T</kbd> New Tab</div>
            <div class="shortcut"><kbd>Ctrl+W</kbd> Close Tab</div>
            <div class="shortcut"><kbd>Ctrl+L</kbd> Focus URL</div>
            <div class="shortcut"><kbd>Ctrl+D</kbd> Add Bookmark</div>
            <div class="shortcut"><kbd>Ctrl+B</kbd> Bookmarks</div>
            <div class="shortcut"><kbd>Ctrl+H</kbd> History</div>
            <div class="shortcut"><kbd>Ctrl+Shift+T</kbd> Reopen Tab</div>
            <div class="shortcut"><kbd>Ctrl+Shift+A</kbd> Search Tabs</div>
          </div>
        </div>
      </div>
    </div>
  {/if}
  
  <!-- Tab Search Modal -->
  {#if showTabSearch}
    <div class="modal-overlay" onclick={() => showTabSearch = false}>
      <div class="tab-search-modal" onclick={(e) => e.stopPropagation()}>
        <input 
          type="text" 
          bind:value={tabSearchQuery} 
          placeholder="Search tabs..." 
          class="tab-search-input"
          autofocus
        />
        <div class="tab-search-results">
          {#each filteredTabs as tab}
            <button class="tab-search-item" onclick={() => { selectTab(tab.tab_id); showTabSearch = false; }}>
              <span class="tab-icon">🌐</span>
              <span class="tab-info">
                <span class="tab-title">{tab.title || 'New Tab'}</span>
                <span class="tab-url">{tab.url}</span>
              </span>
            </button>
          {/each}
          {#if filteredTabs.length === 0}
            <div class="no-results">No matching tabs</div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
  
  <!-- Browser Content Area -->
  <div class="browser-content">
    {#if tabs.length === 0}
      <div class="empty-state">
        <div class="empty-icon">🌐</div>
        <h2>Virtual IP Browser</h2>
        <p>Browse privately with virtual IP addresses</p>
        <div class="quick-actions">
          <button class="btn-large" onclick={() => createNewTab(true)}>
            <span class="btn-icon">🔒</span>
            New Tab with Proxy
          </button>
          <button class="btn-large secondary" onclick={() => createNewTab(false)}>
            <span class="btn-icon">🌐</span>
            New Tab (Direct)
          </button>
        </div>
        
        <div class="features">
          <div class="feature">
            <span class="feature-icon">🛡️</span>
            <span>IP Masking</span>
          </div>
          <div class="feature">
            <span class="feature-icon">🌍</span>
            <span>Multi-Country</span>
          </div>
          <div class="feature">
            <span class="feature-icon">🔄</span>
            <span>IP Rotation</span>
          </div>
          <div class="feature">
            <span class="feature-icon">🧬</span>
            <span>Fingerprint Protection</span>
          </div>
        </div>
      </div>
    {:else}
      <div class="webview-container">
        <div class="webview-placeholder">
          <p>Webview content for: <strong>{activeTab?.url}</strong></p>
          <p class="hint">Browser windows open in separate native windows with proxy support.</p>
        </div>
      </div>
    {/if}
  </div>
  
  <!-- Status Bar -->
  <div class="status-bar">
    <div class="status-left">
      {#if isLoading}
        <span class="status-loading">Loading...</span>
      {:else if activeTab}
        <span class="status-url">{activeTab.url}</span>
      {/if}
    </div>
    <div class="status-right">
      <span class="status-proxy">
        {#if currentProxy}
          🔒 {currentProxy.ip}:{currentProxy.port}
        {:else}
          🌐 Direct Connection
        {/if}
      </span>
      <span class="status-tabs">{tabs.length} tab{tabs.length !== 1 ? 's' : ''}</span>
    </div>
  </div>
</div>

<style>
  .browser-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #1a1a2e;
    color: #e0e0e0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  }
  
  /* Tab Bar */
  .tab-bar {
    display: flex;
    align-items: center;
    background: #0d0d1a;
    padding: 8px 8px 0;
    -webkit-app-region: drag;
  }
  
  .tabs-container {
    display: flex;
    align-items: center;
    flex: 1;
    gap: 2px;
    overflow-x: auto;
    -webkit-app-region: no-drag;
  }
  
  .tab {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: #1f1f35;
    border: none;
    border-radius: 8px 8px 0 0;
    color: #a0a0b0;
    cursor: pointer;
    min-width: 120px;
    max-width: 200px;
    transition: all 0.2s;
  }
  
  .tab:hover {
    background: #2a2a45;
    color: #ffffff;
  }
  
  .tab.active {
    background: #1a1a2e;
    color: #ffffff;
  }
  
  .tab-icon {
    font-size: 14px;
    flex-shrink: 0;
  }
  
  .tab-title {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
  }
  
  .tab-close {
    background: transparent;
    border: none;
    color: #606070;
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 4px;
    font-size: 10px;
    opacity: 0;
    transition: all 0.2s;
  }
  
  .tab:hover .tab-close {
    opacity: 1;
  }
  
  .tab-close:hover {
    background: #ff4757;
    color: white;
  }
  
  .new-tab-btn {
    width: 32px;
    height: 32px;
    background: transparent;
    border: 1px dashed #404060;
    border-radius: 6px;
    color: #808090;
    cursor: pointer;
    font-size: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-left: 4px;
    transition: all 0.2s;
  }
  
  .new-tab-btn:hover {
    background: #2a2a45;
    border-style: solid;
    color: #ffffff;
  }
  
  .window-controls {
    display: flex;
    gap: 4px;
    margin-left: 16px;
    -webkit-app-region: no-drag;
  }
  
  .win-btn {
    width: 28px;
    height: 28px;
    background: transparent;
    border: none;
    color: #808090;
    cursor: pointer;
    border-radius: 4px;
    font-size: 10px;
    transition: all 0.2s;
  }
  
  .win-btn:hover {
    background: #2a2a45;
  }
  
  .win-btn.close:hover {
    background: #ff4757;
    color: white;
  }
  
  /* Navigation Bar */
  .nav-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: #1a1a2e;
    border-bottom: 1px solid #2a2a45;
  }
  
  .nav-buttons {
    display: flex;
    gap: 4px;
  }
  
  .nav-btn {
    width: 32px;
    height: 32px;
    background: transparent;
    border: none;
    color: #a0a0b0;
    cursor: pointer;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }
  
  .nav-btn:hover:not(:disabled) {
    background: #2a2a45;
    color: #ffffff;
  }
  
  .nav-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  
  .spinning {
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
  
  .url-bar {
    flex: 1;
    display: flex;
    align-items: center;
    background: #0d0d1a;
    border: 1px solid #2a2a45;
    border-radius: 20px;
    padding: 4px 12px;
    transition: all 0.2s;
  }
  
  .url-bar:focus-within {
    border-color: #4f46e5;
    box-shadow: 0 0 0 2px rgba(79, 70, 229, 0.2);
  }
  
  .security-indicator {
    color: #606070;
    margin-right: 8px;
  }
  
  .security-indicator.secure {
    color: #10b981;
  }
  
  .url-input {
    flex: 1;
    background: transparent;
    border: none;
    color: #e0e0e0;
    font-size: 13px;
    outline: none;
  }
  
  .url-input::placeholder {
    color: #606070;
  }
  
  .nav-actions {
    display: flex;
    gap: 8px;
  }
  
  .action-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: transparent;
    border: 1px solid #2a2a45;
    border-radius: 6px;
    color: #a0a0b0;
    cursor: pointer;
    font-size: 12px;
    transition: all 0.2s;
  }
  
  .action-btn:hover {
    background: #2a2a45;
    color: #ffffff;
  }
  
  .action-btn.active {
    background: #10b981;
    border-color: #10b981;
    color: white;
  }
  
  .proxy-btn {
    min-width: 100px;
  }
  
  .proxy-flag {
    font-size: 14px;
  }
  
  .proxy-ip {
    font-family: monospace;
    font-size: 11px;
  }
  
  /* Proxy Panel */
  .proxy-panel {
    position: absolute;
    top: 100px;
    right: 12px;
    width: 360px;
    background: #1f1f35;
    border: 1px solid #2a2a45;
    border-radius: 12px;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5);
    z-index: 1000;
    max-height: 500px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  
  .proxy-panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    border-bottom: 1px solid #2a2a45;
  }
  
  .proxy-panel-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }
  
  .close-panel {
    background: transparent;
    border: none;
    color: #808090;
    cursor: pointer;
    font-size: 16px;
  }
  
  .current-ip {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    background: #0d0d1a;
  }
  
  .current-ip .label {
    color: #808090;
    font-size: 12px;
  }
  
  .current-ip .value {
    font-family: monospace;
    font-size: 13px;
    color: #10b981;
  }
  
  .current-ip .flag {
    font-size: 16px;
  }
  
  .proxy-actions {
    display: flex;
    gap: 8px;
    padding: 12px 16px;
  }
  
  .btn-primary, .btn-secondary {
    flex: 1;
    padding: 10px 16px;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
    transition: all 0.2s;
  }
  
  .btn-primary {
    background: #4f46e5;
    color: white;
  }
  
  .btn-primary:hover:not(:disabled) {
    background: #4338ca;
  }
  
  .btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  .btn-secondary {
    background: #2a2a45;
    color: #e0e0e0;
  }
  
  .btn-secondary:hover {
    background: #3a3a55;
  }
  
  .proxy-list {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  
  .proxy-list h4 {
    margin: 0;
    padding: 12px 16px 8px;
    font-size: 12px;
    color: #808090;
    font-weight: 500;
  }
  
  .proxy-items {
    flex: 1;
    overflow-y: auto;
    padding: 0 8px 8px;
  }
  
  .proxy-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 10px 12px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 8px;
    color: #e0e0e0;
    cursor: pointer;
    text-align: left;
    transition: all 0.2s;
    margin-bottom: 4px;
  }
  
  .proxy-item:hover {
    background: #2a2a45;
  }
  
  .proxy-item.active {
    background: #10b98120;
    border-color: #10b981;
  }
  
  .proxy-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  
  .proxy-addr {
    font-family: monospace;
    font-size: 12px;
  }
  
  .proxy-meta {
    font-size: 10px;
    color: #808090;
    text-transform: uppercase;
  }
  
  .proxy-speed {
    font-size: 11px;
    color: #10b981;
    font-family: monospace;
  }
  
  .no-proxies {
    text-align: center;
    padding: 20px;
    color: #808090;
    font-size: 13px;
  }
  
  /* Enhanced Proxy Panel Styles */
  .rotation-section {
    padding: 16px;
    border-bottom: 1px solid #2a2a45;
  }
  
  .rotation-section h4 {
    margin: 0 0 8px 0;
    font-size: 12px;
    font-weight: 600;
    color: #808090;
    text-transform: uppercase;
  }
  
  .strategy-select {
    width: 100%;
    padding: 8px 12px;
    background: #0d0d1a;
    border: 1px solid #2a2a45;
    border-radius: 6px;
    color: #e0e0e0;
    font-size: 13px;
    margin-bottom: 8px;
  }
  
  .btn-rotate {
    width: 100%;
    padding: 8px 16px;
    background: #4f46e5;
    border: none;
    border-radius: 6px;
    color: white;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .btn-rotate:hover:not(:disabled) {
    background: #6366f1;
  }
  
  .btn-rotate:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .proxy-stats {
    padding: 12px 16px;
    background: #0d0d1a;
    border-bottom: 1px solid #2a2a45;
  }
  
  .stat-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 4px 0;
    font-size: 12px;
  }
  
  .stat-item .label {
    color: #808090;
  }
  
  .stat-item .value {
    color: #e0e0e0;
    font-family: monospace;
  }
  
  .provider-section {
    padding: 16px;
    border-bottom: 1px solid #2a2a45;
  }
  
  .provider-section h4 {
    margin: 0 0 8px 0;
    font-size: 12px;
    font-weight: 600;
    color: #808090;
    text-transform: uppercase;
  }
  
  .provider-buttons {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
  }
  
  .btn-provider {
    padding: 6px 12px;
    background: #2a2a45;
    border: 1px solid #3a3a55;
    border-radius: 6px;
    color: #e0e0e0;
    font-size: 11px;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .btn-provider:hover:not(:disabled) {
    background: #3a3a55;
    border-color: #4a4a65;
  }
  
  .btn-provider:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .proxy-filters {
    display: flex;
    gap: 8px;
    margin-bottom: 12px;
    padding: 0 16px;
  }
  
  .filter-btn {
    padding: 6px 12px;
    background: #2a2a45;
    border: 1px solid #3a3a55;
    border-radius: 6px;
    color: #e0e0e0;
    font-size: 11px;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .filter-btn:hover {
    background: #3a3a55;
    border-color: #4a4a65;
  }
  
  .proxy-item.working {
    border-color: #10b981;
  }
  
  .proxy-item.failed {
    border-color: #ef4444;
    opacity: 0.7;
  }
  
  .proxy-actions-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  
  .validate-btn {
    background: transparent;
    border: 1px solid #3a3a55;
    border-radius: 4px;
    color: #808090;
    cursor: pointer;
    padding: 4px 8px;
    font-size: 12px;
    transition: all 0.2s;
  }
  
  .validate-btn:hover:not(:disabled) {
    border-color: #4a4a65;
    color: #e0e0e0;
  }
  
  .validate-btn:disabled {
    cursor: not-allowed;
  }
  
  /* Browser Content */
  .browser-content {
    flex: 1;
    overflow: hidden;
    position: relative;
  }
  
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    padding: 40px;
  }
  
  .empty-icon {
    font-size: 64px;
    margin-bottom: 20px;
  }
  
  .empty-state h2 {
    margin: 0 0 8px;
    font-size: 28px;
    font-weight: 600;
  }
  
  .empty-state p {
    margin: 0 0 32px;
    color: #808090;
    font-size: 14px;
  }
  
  .quick-actions {
    display: flex;
    gap: 16px;
    margin-bottom: 48px;
  }
  
  .btn-large {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px 32px;
    background: #4f46e5;
    border: none;
    border-radius: 12px;
    color: white;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    transition: all 0.2s;
  }
  
  .btn-large:hover {
    background: #4338ca;
    transform: translateY(-2px);
  }
  
  .btn-large.secondary {
    background: #2a2a45;
  }
  
  .btn-large.secondary:hover {
    background: #3a3a55;
  }
  
  .btn-icon {
    font-size: 20px;
  }
  
  .features {
    display: flex;
    gap: 32px;
  }
  
  .feature {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    color: #808090;
    font-size: 12px;
  }
  
  .feature-icon {
    font-size: 24px;
  }
  
  .webview-container {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #0d0d1a;
  }
  
  .webview-placeholder {
    text-align: center;
    color: #808090;
  }
  
  .webview-placeholder strong {
    color: #e0e0e0;
  }
  
  .hint {
    margin-top: 12px;
    font-size: 12px;
    opacity: 0.7;
  }
  
  /* Status Bar */
  .status-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 12px;
    background: #0d0d1a;
    border-top: 1px solid #2a2a45;
    font-size: 11px;
    color: #808090;
  }
  
  .status-left {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  
  .status-loading {
    color: #4f46e5;
  }
  
  .status-right {
    display: flex;
    gap: 16px;
  }
  
  .status-proxy {
    color: #10b981;
  }
  
  /* Spinner */
  .spinner {
    width: 12px;
    height: 12px;
    border: 2px solid #404060;
    border-top-color: #4f46e5;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  
  /* Scrollbar */
  ::-webkit-scrollbar {
    width: 6px;
    height: 6px;
  }
  
  ::-webkit-scrollbar-track {
    background: transparent;
  }
  
  ::-webkit-scrollbar-thumb {
    background: #2a2a45;
    border-radius: 3px;
  }
  
  ::-webkit-scrollbar-thumb:hover {
    background: #3a3a55;
  }
  
  /* Bookmarks Bar */
  .bookmarks-bar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 12px;
    background: #151525;
    border-bottom: 1px solid #2a2a45;
    font-size: 12px;
  }
  
  .bookmarks-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    background: transparent;
    border: none;
    color: #808090;
    cursor: pointer;
    border-radius: 4px;
    font-size: 12px;
  }
  
  .bookmarks-toggle:hover {
    background: #2a2a45;
    color: #e0e0e0;
  }
  
  .bookmark-items {
    display: flex;
    gap: 2px;
    flex: 1;
    overflow-x: auto;
  }
  
  .bookmark-item {
    padding: 4px 8px;
    background: transparent;
    border: none;
    color: #a0a0b0;
    cursor: pointer;
    border-radius: 4px;
    font-size: 11px;
    white-space: nowrap;
  }
  
  .bookmark-item:hover {
    background: #2a2a45;
    color: #e0e0e0;
  }
  
  .toolbar-spacer {
    flex: 1;
  }
  
  .toolbar-btn {
    width: 28px;
    height: 28px;
    background: transparent;
    border: none;
    color: #808090;
    cursor: pointer;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .toolbar-btn:hover {
    background: #2a2a45;
    color: #e0e0e0;
  }
  
  .icon-btn {
    padding: 6px;
    min-width: auto;
  }
  
  /* Side Panels */
  .side-panel {
    position: absolute;
    top: 100px;
    right: 12px;
    width: 380px;
    max-height: 70vh;
    background: #1f1f35;
    border: 1px solid #2a2a45;
    border-radius: 12px;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5);
    z-index: 1000;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  
  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    border-bottom: 1px solid #2a2a45;
  }
  
  .panel-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }
  
  .panel-search {
    padding: 12px;
    border-bottom: 1px solid #2a2a45;
  }
  
  .panel-search input {
    width: 100%;
    padding: 8px 12px;
    background: #0d0d1a;
    border: 1px solid #2a2a45;
    border-radius: 6px;
    color: #e0e0e0;
    font-size: 13px;
  }
  
  .panel-actions {
    padding: 8px 12px;
    border-bottom: 1px solid #2a2a45;
  }
  
  .btn-danger {
    padding: 8px 16px;
    background: #dc2626;
    border: none;
    border-radius: 6px;
    color: white;
    font-size: 12px;
    cursor: pointer;
  }
  
  .btn-danger:hover {
    background: #b91c1c;
  }
  
  .panel-content {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }
  
  .empty-panel {
    text-align: center;
    padding: 40px 20px;
    color: #606070;
    font-size: 13px;
  }
  
  .panel-item {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 10px 12px;
    background: transparent;
    border: none;
    border-radius: 8px;
    color: #e0e0e0;
    cursor: pointer;
    text-align: left;
    margin-bottom: 4px;
  }
  
  .panel-item:hover {
    background: #2a2a45;
  }
  
  .item-icon {
    font-size: 16px;
    flex-shrink: 0;
  }
  
  .item-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  
  .item-title {
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  
  .item-url, .item-meta {
    font-size: 11px;
    color: #808090;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  
  .item-delete {
    background: transparent;
    border: none;
    color: #606070;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    opacity: 0;
  }
  
  .panel-item:hover .item-delete {
    opacity: 1;
  }
  
  .item-delete:hover {
    background: #dc2626;
    color: white;
  }
  
  /* Settings Panel */
  .settings-panel {
    width: 400px;
  }
  
  .settings-section {
    padding: 16px;
    border-bottom: 1px solid #2a2a45;
  }
  
  .settings-section:last-child {
    border-bottom: none;
  }
  
  .settings-section h4 {
    margin: 0 0 12px 0;
    font-size: 12px;
    font-weight: 600;
    color: #808090;
    text-transform: uppercase;
  }
  
  .setting-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 0;
    font-size: 13px;
    cursor: pointer;
  }
  
  .setting-item input[type="checkbox"] {
    width: 18px;
    height: 18px;
    accent-color: #4f46e5;
  }
  
  .setting-select {
    width: 100%;
    padding: 8px 12px;
    background: #0d0d1a;
    border: 1px solid #2a2a45;
    border-radius: 6px;
    color: #e0e0e0;
    font-size: 13px;
  }
  
  .shortcuts-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  
  .shortcut {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 12px;
    color: #a0a0b0;
  }
  
  .shortcut kbd {
    padding: 4px 8px;
    background: #0d0d1a;
    border: 1px solid #2a2a45;
    border-radius: 4px;
    font-family: monospace;
    font-size: 11px;
    color: #e0e0e0;
  }
  
  /* Tab Search Modal */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 100px;
    z-index: 2000;
  }
  
  .tab-search-modal {
    width: 500px;
    max-height: 400px;
    background: #1f1f35;
    border: 1px solid #2a2a45;
    border-radius: 12px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  
  .tab-search-input {
    width: 100%;
    padding: 16px;
    background: transparent;
    border: none;
    border-bottom: 1px solid #2a2a45;
    color: #e0e0e0;
    font-size: 16px;
  }
  
  .tab-search-results {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }
  
  .tab-search-item {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 12px;
    background: transparent;
    border: none;
    border-radius: 8px;
    color: #e0e0e0;
    cursor: pointer;
    text-align: left;
  }
  
  .tab-search-item:hover {
    background: #2a2a45;
  }
  
  .tab-search-item .tab-icon {
    font-size: 18px;
  }
  
  .tab-search-item .tab-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  
  .tab-search-item .tab-title {
    font-size: 14px;
    font-weight: 500;
  }
  
  .tab-search-item .tab-url {
    font-size: 12px;
    color: #808090;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  
  .no-results {
    text-align: center;
    padding: 20px;
    color: #606070;
    font-size: 13px;
  }
</style>
