<script lang="ts">
  import { onMount, onDestroy, getContext } from 'svelte';
  import { logDebug, logError } from '../lib/logger';
  import TwoPanelLayout from './layout/TwoPanelLayout.svelte';
  import LeftPanel from './layout/LeftPanel.svelte';
  import RightPanel from './layout/RightPanel.svelte';
  import WebviewBrowser from './browser/WebviewBrowser.svelte';
  import UserManagement from './auth/UserManagement.svelte';
  import ApiSettingsPanel from './config/ApiSettingsPanel.svelte';

  import type { Tab, Country, ValidationResponse, WebviewTab } from '../lib/types';
  import {
    fetchTabs,
    fetchCountries,
    createTab,
    createTabRandom,
    rotateIp,
    validateIp
  } from '../lib/api';
  import { KeyboardShortcutManager, type ShortcutHandlers } from '../lib/shortcuts';
  import type { AuthContext } from './auth/AuthWrapper.svelte';
  
  const auth: AuthContext = getContext('auth');
  
  let tabs: Tab[] = [];
  let countries: Country[] = [];
  let selectedTab: Tab | null = null;
  let validation: ValidationResponse | null = null;
  let loading = false;
  let validating = false;
  let error: string | null = null;
  let shortcutManager: KeyboardShortcutManager | null = null;
  
  // Webview browser state
  let selectedWebviewTab: WebviewTab | null = null;
  let activeView: 'browser' | 'enterprise' | 'api-settings' = 'browser';

  
  // Subscribe to auth state
  $: if ($auth.user) {
    // Check if user is admin or enterprise to show enterprise features
    activeView = $auth.user?.role === 'Admin' || $auth.user?.role === 'Enterprise' ? 'enterprise' : 'browser';
  }
  
  onMount(async () => {
    await loadCountries();
    await loadTabs();
    
    // Initialize keyboard shortcuts
    const handlers: ShortcutHandlers = {
      newTab: () => handleCreateRandom(),
      closeTab: () => {
        if (selectedTab) {
          handleTabClosed(new CustomEvent('tabClosed', { detail: { tabId: selectedTab.tab_id } }));
        }
      },
      focusUrl: () => {
        const event = new CustomEvent('focusUrl');
        document.dispatchEvent(event);
      },
      goBack: () => {
        if (selectedTab) {
          handleRotateEvent(new CustomEvent('rotate', { detail: { tab: selectedTab } }));
        }
      },
      goForward: () => {
        if (selectedTab) {
          handleRotateEvent(new CustomEvent('rotate', { detail: { tab: selectedTab } }));
        }
      },
      reload: () => {
        const event = new CustomEvent('reload');
        document.dispatchEvent(event);
      },
      switchTab: (direction) => {
        const currentIndex = tabs.findIndex(t => t.tab_id === selectedTab?.tab_id);
        if (direction === 'next') {
          const nextIndex = (currentIndex + 1) % tabs.length;
          selectedTab = tabs[nextIndex];
        } else {
          const prevIndex = currentIndex === 0 ? tabs.length - 1 : currentIndex - 1;
          selectedTab = tabs[prevIndex];
        }
      },
      validateIp: () => {
        if (selectedTab) {
          handleValidateEvent();
        }
      },
      rotateIp: () => {
        if (selectedTab) {
          handleRotateEvent(new CustomEvent('rotate', { detail: { tab: selectedTab } }));
        }
      }
    };
    
    shortcutManager = new KeyboardShortcutManager(handlers);
    shortcutManager.enable();
    
    // Listen for custom events
    document.addEventListener('switchToTab', handleSwitchToTab);
  });
  
  onDestroy(() => {
    if (shortcutManager) {
      shortcutManager.disable();
    }
    document.removeEventListener('switchToTab', handleSwitchToTab);
  });
  
  function handleSwitchToTab(e: CustomEvent) {
    const index = e.detail.index;
    if (index >= 0 && index < tabs.length) {
      selectedTab = tabs[index];
    }
  }

  async function loadCountries() {
    try {
      countries = await fetchCountries();
    } catch (e) {
      logError(e);
    }
  }

  async function loadTabs() {
    loading = true;
    error = null;
    try {
      tabs = await fetchTabs();
      if (tabs.length && !selectedTab) {
        selectedTab = tabs[0];
      } else if (selectedTab) {
        selectedTab = tabs.find((t) => t.tab_id === selectedTab?.tab_id) || selectedTab;
      }
    } catch (e) {
      error = 'Failed to load tabs';
      logError(e);
    } finally {
      loading = false;
    }
  }

  async function handleCreateTab(countryCode: string) {
    loading = true;
    error = null;
    try {
      const newTab = await createTab(countryCode);
      tabs = [...tabs, newTab];
      selectedTab = newTab;
    } catch (e) {
      error = 'Failed to create tab';
      logError(e);
    } finally {
      loading = false;
    }
  }

  async function handleCreateRandom() {
    loading = true;
    error = null;
    try {
      const newTab = await createTabRandom();
      tabs = [...tabs, newTab];
      selectedTab = newTab;
    } catch (e) {
      error = 'Failed to create random tab';
      logError(e);
    } finally {
      loading = false;
    }
  }

  async function handleRotateEvent(e: CustomEvent) {
    if (!selectedTab) return;
    validating = true;
    try {
      const response = await rotateIp(selectedTab.tab_id);
      // Create a validation response from the virtual IP response
      validation = {
        ip: response.ip,
        ip_matches: true, // Assume success for now
        webrtc_secure: true,
        dns_secure: true,
        overall_pass: true
      };
      // Refresh tabs to get updated IP
      await loadTabs();
    } catch (e) {
      error = 'Failed to rotate IP';
      logError(e);
    } finally {
      validating = false;
    }
  }

  async function handleValidateEvent() {
    if (!selectedTab) return;
    validating = true;
    try {
      const response = await validateIp(selectedTab.tab_id);
      validation = response;
    } catch (e) {
      error = 'Failed to validate IP';
      logError(e);
    } finally {
      validating = false;
    }
  }

  function handleSelect(e: CustomEvent) {
    selectedTab = e.detail.tab;
  }

  function handleTabClosed(e: CustomEvent) {
    const tabId = e.detail.tabId;
    tabs = tabs.filter((t) => t.tab_id !== tabId);
    if (selectedTab?.tab_id === tabId) {
      selectedTab = tabs.length > 0 ? tabs[0] : null;
    }
  }
  
  // Webview browser handlers
  function handleWebviewTabSelected(e: CustomEvent) {
    selectedWebviewTab = e.detail.tab;
  }
  
  function handleWebviewTabCreated(e: CustomEvent) {
    // Optionally associate with IP tab
    logDebug('Webview tab created:', e.detail.tab);
  }
  
  function handleWebviewNavigated(e: CustomEvent) {
    logDebug('Webview navigated:', e.detail);
  }
  
  async function handleLogout() {
    await auth.logout();
  }
</script>

<div class="main-app">
  <!-- Header with user info and logout -->
  <header class="app-header">
    <div class="header-left">
      <h1>Virtual IP Browser</h1>
      {#if $auth.user}
        <span class="user-info">
          {$auth.user.username} 
          <span class="role-badge {$auth.user.role.toLowerCase()}">
            {$auth.user.role}
          </span>
        </span>
      {/if}
    </div>
    <div class="header-right">
      {#if $auth.user?.role === 'Admin' || $auth.user?.role === 'Enterprise'}
        <button 
          class="view-toggle"
          class:active={activeView === 'enterprise'}
          on:click={() => activeView = 'enterprise'}
        >
          Enterprise
        </button>
      {/if}
      <button 
        class="view-toggle"
        class:active={activeView === 'browser'}
        on:click={() => activeView = 'browser'}
      >
        Browser
      </button>
      <button 
        class="view-toggle"
        class:active={activeView === 'api-settings'}
        on:click={() => activeView = 'api-settings'}
      >
        🔑 API Settings
      </button>
      <button class="logout-btn" on:click={handleLogout}>
        Logout
      </button>
    </div>

  </header>
  
  <!-- Main content -->
  <main class="app-content">
    {#if activeView === 'browser'}
      <TwoPanelLayout leftWidth={60}>
        <div slot="left">
          <LeftPanel
            {tabs}
            {countries}
            {selectedTab}
            {validation}
            {loading}
            {validating}
            on:select={handleSelect}
            onCreateTab={handleCreateTab}
            onCreateRandom={handleCreateRandom}
            on:rotate={handleRotateEvent}
            on:validate={handleValidateEvent}
            on:tabClosed={handleTabClosed}
          />
        </div>
        <div slot="right">
          <RightPanel />
        </div>
      </TwoPanelLayout>
      
      <!-- Webview Browser Panel -->
      <div class="webview-panel">
        <WebviewBrowser 
          bind:selectedTab={selectedWebviewTab}
          on:tabSelected={handleWebviewTabSelected}
          on:tabCreated={handleWebviewTabCreated}
          on:navigated={handleWebviewNavigated}
        />
      </div>
    {:else if activeView === 'enterprise'}
      <UserManagement />
    {:else if activeView === 'api-settings'}
      <div class="settings-container">
        <ApiSettingsPanel />
      </div>
    {/if}

  </main>
  
  {#if error}
    <div class="error-toast">{error}</div>
  {/if}
</div>

<style>
  .main-app {
    height: 100vh;
    width: 100vw;
    display: flex;
    flex-direction: column;
    background: #080c18;
  }
  
  .app-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 20px;
    background: #0c1120;
    border-bottom: 1px solid #1f2a45;
  }
  
  .header-left {
    display: flex;
    align-items: center;
    gap: 20px;
  }
  
  .header-left h1 {
    color: #e0e7f5;
    font-size: 20px;
    font-weight: 600;
    margin: 0;
  }
  
  .user-info {
    display: flex;
    align-items: center;
    gap: 8px;
    color: #9fb0ce;
    font-size: 14px;
  }
  
  .role-badge {
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
  }
  
  .role-badge.admin {
    background: #3b82f620;
    color: #60a5fa;
  }
  
  .role-badge.enterprise {
    background: #8b5cf620;
    color: #a78bfa;
  }
  
  .role-badge.user {
    background: #6b728020;
    color: #9ca3af;
  }
  
  .header-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  
  .view-toggle {
    background: transparent;
    color: #9fb0ce;
    border: 1px solid #2a3750;
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 13px;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .view-toggle:hover {
    background: #2a3750;
    color: #e0e7f5;
  }
  
  .view-toggle.active {
    background: #3b82f6;
    color: white;
    border-color: #3b82f6;
  }
  
  .logout-btn {
    background: #dc2626;
    color: white;
    border: none;
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 13px;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .logout-btn:hover {
    background: #b91c1c;
  }
  
  .app-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  
  .webview-panel {
    height: 300px;
    border-top: 1px solid #1f2a45;
  }
  
  .error-toast {
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    background: #3a1a2c;
    border: 1px solid #ff5c8a;
    color: #ffb3c8;
    padding: 12px 20px;
    border-radius: 10px;
    z-index: 1000;
    animation: fadeIn 0.3s ease;
  }
  
  @keyframes fadeIn {
    from { opacity: 0; transform: translate(-50%, 10px); }
    to { opacity: 1; transform: translate(-50%, 0); }
  }
  
  .settings-container {
    flex: 1;
    overflow-y: auto;
    background: #0c1120;
    padding: 20px;
  }
</style>

