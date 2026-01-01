<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import VirtualList from '../ui/VirtualList.svelte';
  import type { Tab } from '../../lib/types';
  import { virtualScrollEnabled, virtualScrollThreshold } from '../../lib/settings';
  
  export let tabs: Tab[] = [];
  export let selectedTab: Tab | null = null;
  export let height: number = 400;
  export let itemHeight: number = 48;
  
  const dispatch = createEventDispatcher();
  
  // Interface for VirtualList component methods
  interface VirtualListRef {
    scrollToItem: (index: number) => void;
    getVisibleRange: () => { start: number; end: number };
  }
  
  let virtualList: VirtualListRef | null = null;
  let useVirtualScroll = false;

  
  // Determine whether to use virtual scroll
  $: useVirtualScroll = $virtualScrollEnabled && tabs.length > $virtualScrollThreshold;
  
  // Render function for virtual list
  function renderTabItem(tab: Tab, index: number): string {
    const isSelected = selectedTab?.tab_id === tab.tab_id;
    return `
      <div class="tab-item ${isSelected ? 'selected' : ''}" data-tab-id="${tab.tab_id}">
        <div class="tab-info">
          <span class="tab-country">${tab.country_name || tab.country_code}</span>
          <span class="tab-ip">${tab.ip}</span>
        </div>
        <div class="tab-details">
          <span class="tab-city">${tab.city || 'Unknown'}</span>
          <span class="tab-isp">${tab.isp || 'Unknown ISP'}</span>
        </div>
      </div>
    `;
  }
  
  function handleTabClick(event: MouseEvent) {
    const tabElement = (event.target as HTMLElement).closest('[data-tab-id]');
    if (tabElement) {
      const tabId = tabElement.getAttribute('data-tab-id');
      const tab = tabs.find(t => t.tab_id === tabId);
      if (tab) {
        selectedTab = tab;
        dispatch('select', { tab });
      }
    }
  }
  
  function handleVirtualSelect(event: CustomEvent) {
    const { index, item } = event.detail;
    selectedTab = item;
    dispatch('select', { tab: item });
  }
  
  function scrollToTab(tabId: string) {
    const index = tabs.findIndex(t => t.tab_id === tabId);
    if (index >= 0) {
      if (useVirtualScroll && virtualList) {
        virtualList.scrollToItem(index);
      } else {
        const element = document.querySelector(`[data-tab-id="${tabId}"]`) as HTMLElement;
        if (element) {
          element.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        }
      }
    }
  }
  
  // Keyboard navigation
  function handleKeydown(event: KeyboardEvent) {
    const currentIndex = selectedTab ? tabs.findIndex(t => t.tab_id === selectedTab.tab_id) : -1;
    let newIndex = -1;
    
    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        newIndex = Math.min(currentIndex + 1, tabs.length - 1);
        break;
      case 'ArrowUp':
        event.preventDefault();
        newIndex = Math.max(currentIndex - 1, 0);
        break;
      case 'Home':
        event.preventDefault();
        newIndex = 0;
        break;
      case 'End':
        event.preventDefault();
        newIndex = tabs.length - 1;
        break;
    }
    
    if (newIndex >= 0 && newIndex !== currentIndex) {
      const tab = tabs[newIndex];
      selectedTab = tab;
      scrollToTab(tab.tab_id);
      dispatch('select', { tab });
    }
  }
  
  // Performance optimization: Debounce scroll events
  let scrollTimeout: number;
  function handleScroll(event: Event) {
    clearTimeout(scrollTimeout);
    scrollTimeout = setTimeout(() => {
      dispatch('scroll', event);
    }, 16); // ~60fps
  }
  
  onMount(() => {
    // Auto-scroll to selected tab
    if (selectedTab) {
      setTimeout(() => scrollToTab(selectedTab.tab_id), 100);
    }
  });
  
  // Export methods for parent component
  export { scrollToTab };
</script>

<div class="enhanced-tab-list" style="height: {height}px;" on:keydown={handleKeydown} tabindex="0">
  {#if useVirtualScroll}
    <VirtualList
      bind:this={virtualList}
      {items: tabs}
      {itemHeight}
      containerHeight={height}
      overscan={5}
      renderItem={renderTabItem}
      on:select={handleVirtualSelect}
      on:scroll={handleScroll}
    />
  {:else}
    <div class="regular-tab-list" on:click={handleTabClick} on:scroll={handleScroll}>
      {#each tabs as tab (tab.tab_id)}
        <div 
          class="tab-item {selectedTab?.tab_id === tab.tab_id ? 'selected' : ''}"
          data-tab-id={tab.tab_id}
          style="height: {itemHeight}px;"
        >
          <div class="tab-info">
            <span class="tab-country">{tab.country_name || tab.country_code}</span>
            <span class="tab-ip">{tab.ip}</span>
          </div>
          <div class="tab-details">
            <span class="tab-city">{tab.city || 'Unknown'}</span>
            <span class="tab-isp">{tab.isp || 'Unknown ISP'}</span>
          </div>
        </div>
      {/each}
      
      {#if tabs.length === 0}
        <div class="empty-state">
          <p>No tabs created yet</p>
          <p class="hint">Create a tab to get started</p>
        </div>
      {/if}
    </div>
  {/if}
  
  <!-- Performance indicator -->
  {#if tabs.length > 100}
    <div class="performance-indicator" class:virtual={useVirtualScroll}>
      {useVirtualScroll ? '🚀 Virtual Scroll Active' : '⚠️ Large List - Consider Virtual Scroll'}
    </div>
  {/if}
</div>

<style>
  .enhanced-tab-list {
    position: relative;
    outline: none;
    border: 1px solid #2a3750;
    border-radius: 8px;
    background: #0c1120;
    overflow: hidden;
  }
  
  .regular-tab-list {
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-width: thin;
    scrollbar-color: #2a3750 #0c1120;
  }
  
  .regular-tab-list::-webkit-scrollbar {
    width: 8px;
  }
  
  .regular-tab-list::-webkit-scrollbar-track {
    background: #0c1120;
  }
  
  .regular-tab-list::-webkit-scrollbar-thumb {
    background: #2a3750;
    border-radius: 4px;
  }
  
  .tab-item {
    display: flex;
    flex-direction: column;
    padding: 8px 12px;
    border-bottom: 1px solid #1a2332;
    cursor: pointer;
    transition: all 0.15s ease;
    gap: 4px;
  }
  
  .tab-item:hover {
    background: #1a2332;
  }
  
  .tab-item.selected {
    background: #1e3a5f;
    border-left: 3px solid #3b82f6;
  }
  
  .tab-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 13px;
  }
  
  .tab-country {
    color: #e0e7f5;
    font-weight: 500;
  }
  
  .tab-ip {
    color: #60a5fa;
    font-family: 'Courier New', monospace;
    font-size: 12px;
  }
  
  .tab-details {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 11px;
    color: #6b7a9a;
  }
  
  .tab-city {
    color: #9fb0ce;
  }
  
  .tab-isp {
    color: #6b7a9a;
    max-width: 150px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #6b7a9a;
    text-align: center;
  }
  
  .empty-state p {
    margin: 4px 0;
  }
  
  .empty-state .hint {
    font-size: 12px;
    opacity: 0.7;
  }
  
  .performance-indicator {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    padding: 4px 12px;
    font-size: 11px;
    text-align: center;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(4px);
    z-index: 10;
  }
  
  .performance-indicator.virtual {
    background: rgba(16, 185, 129, 0.1);
    color: #10b981;
  }
  
  .performance-indicator:not(.virtual) {
    background: rgba(251, 146, 60, 0.1);
    color: #fb923c;
  }
  
  /* Virtual list styles override */
  :global(.virtual-list-item) {
    padding: 0 !important;
  }
  
  :global(.virtual-list-item .tab-item) {
    margin: 0;
    border-radius: 0;
  }
</style>
