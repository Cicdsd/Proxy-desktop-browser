<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import type { ProxySettings, ProxyStatus } from '../../lib/types';
  
  export let proxySettings: ProxySettings | null = null;
  export let isConnected: boolean = false;
  export let currentIp: string = '';
  export let currentCountry: string = '';
  export let latency: number = 0;
  export let bytesDownloaded: number = 0;
  export let bytesUploaded: number = 0;
  export let isLoading: boolean = false;
  export let loadingProgress: number = 0;
  
  const dispatch = createEventDispatcher();
  
  let showDetails = false;
  let updateInterval: ReturnType<typeof setInterval>;
  
  // Format bytes to human readable
  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }
  
  // Format latency with color indication
  function getLatencyClass(ms: number): string {
    if (ms === 0) return 'text-gray-400';
    if (ms < 100) return 'text-green-500';
    if (ms < 300) return 'text-yellow-500';
    return 'text-red-500';
  }
  
  // Get connection status icon
  function getStatusIcon(): string {
    if (!proxySettings?.enabled) return '🔓';
    if (!isConnected) return '🔴';
    return '🟢';
  }
  
  // Get country flag emoji
  function getCountryFlag(countryCode: string): string {
    // Handle both country codes (US) and full names (United States)
    let code = countryCode;
    if (!countryCode || countryCode.length < 2) return '🌐';
    
    // If it's a full country name, try to extract first 2 chars as a fallback
    // In production, you'd want a proper country name to code mapping
    if (countryCode.length > 2) {
      // Common country name to code mappings
      const countryMap: Record<string, string> = {
        'united states': 'US',
        'united kingdom': 'GB',
        'germany': 'DE',
        'france': 'FR',
        'canada': 'CA',
        'australia': 'AU',
        'japan': 'JP',
        'china': 'CN',
        'brazil': 'BR',
        'india': 'IN',
      };
      code = countryMap[countryCode.toLowerCase()] || countryCode.slice(0, 2);
    }
    
    if (code.length !== 2) return '🌐';
    
    const offset = 127397;
    const upper = code.toUpperCase();
    return String.fromCodePoint(
      upper.charCodeAt(0) + offset,
      upper.charCodeAt(1) + offset
    );
  }
  
  function toggleDetails() {
    showDetails = !showDetails;
  }
  
  function handleRefreshIp() {
    dispatch('refresh-ip');
  }
  
  function handleRotateProxy() {
    dispatch('rotate-proxy');
  }
  
  // Update interval in milliseconds (5 seconds to reduce overhead)
  const STATS_UPDATE_INTERVAL = 5000;
  
  onMount(() => {
    // Update statistics periodically
    updateInterval = setInterval(() => {
      dispatch('request-stats');
    }, STATS_UPDATE_INTERVAL);
  });
  
  onDestroy(() => {
    if (updateInterval) {
      clearInterval(updateInterval);
    }
  });
</script>

<div class="status-bar">
  <!-- Left section: Proxy status -->
  <div class="status-section proxy-status" role="button" tabindex="0" 
       on:click={toggleDetails} on:keydown={(e) => e.key === 'Enter' && toggleDetails()}>
    <span class="status-icon">{getStatusIcon()}</span>
    
    {#if proxySettings?.enabled}
      <span class="status-text">
        {#if isConnected}
          <span class="country-flag">{getCountryFlag(currentCountry)}</span>
          <span class="ip-address">{currentIp || 'Connecting...'}</span>
        {:else}
          <span class="connecting">Connecting to proxy...</span>
        {/if}
      </span>
    {:else}
      <span class="status-text direct">Direct Connection</span>
    {/if}
    
    {#if latency > 0}
      <span class="latency {getLatencyClass(latency)}">{latency}ms</span>
    {/if}
  </div>
  
  <!-- Middle section: Loading progress -->
  <div class="status-section loading-section">
    {#if isLoading}
      <div class="loading-bar">
        <div class="loading-progress" style="width: {loadingProgress}%"></div>
      </div>
      <span class="loading-text">Loading... {loadingProgress}%</span>
    {/if}
  </div>
  
  <!-- Right section: Transfer stats -->
  <div class="status-section transfer-stats">
    <span class="stat download" title="Downloaded">
      ↓ {formatBytes(bytesDownloaded)}
    </span>
    <span class="stat upload" title="Uploaded">
      ↑ {formatBytes(bytesUploaded)}
    </span>
  </div>
  
  <!-- Details popup -->
  {#if showDetails && proxySettings?.enabled}
    <div class="details-popup">
      <div class="details-header">
        <h3>Proxy Details</h3>
        <button class="close-btn" on:click|stopPropagation={toggleDetails}>×</button>
      </div>
      <div class="details-content">
        <div class="detail-row">
          <span class="label">Status:</span>
          <span class="value {isConnected ? 'connected' : 'disconnected'}">
            {isConnected ? 'Connected' : 'Disconnected'}
          </span>
        </div>
        <div class="detail-row">
          <span class="label">Proxy IP:</span>
          <span class="value">{currentIp || 'N/A'}</span>
        </div>
        <div class="detail-row">
          <span class="label">Country:</span>
          <span class="value">
            {getCountryFlag(currentCountry)} {currentCountry || 'Unknown'}
          </span>
        </div>
        <div class="detail-row">
          <span class="label">Latency:</span>
          <span class="value {getLatencyClass(latency)}">{latency}ms</span>
        </div>
        <div class="detail-row">
          <span class="label">Type:</span>
          <span class="value">{proxySettings?.type || 'HTTP'}</span>
        </div>
        <div class="detail-row">
          <span class="label">Server:</span>
          <span class="value">{proxySettings?.host || 'N/A'}:{proxySettings?.port || ''}</span>
        </div>
        
        <div class="details-actions">
          <button class="action-btn" on:click|stopPropagation={handleRefreshIp}>
            🔄 Check IP
          </button>
          <button class="action-btn rotate" on:click|stopPropagation={handleRotateProxy}>
            🔀 Rotate Proxy
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 24px;
    padding: 0 12px;
    background: var(--status-bar-bg, #1e1e1e);
    border-top: 1px solid var(--border-color, #333);
    font-size: 12px;
    color: var(--text-secondary, #ccc);
    position: relative;
    user-select: none;
  }
  
  .status-section {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  
  .proxy-status {
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 4px;
    transition: background 0.2s;
  }
  
  .proxy-status:hover {
    background: rgba(255, 255, 255, 0.1);
  }
  
  .status-icon {
    font-size: 10px;
  }
  
  .status-text {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  
  .status-text.direct {
    color: var(--text-muted, #888);
  }
  
  .country-flag {
    font-size: 14px;
  }
  
  .ip-address {
    font-family: 'Monaco', 'Consolas', monospace;
    font-size: 11px;
  }
  
  .connecting {
    color: var(--warning-color, #f0a000);
    font-style: italic;
  }
  
  .latency {
    font-family: monospace;
    font-size: 10px;
    padding: 1px 4px;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 3px;
  }
  
  .loading-section {
    flex: 1;
    max-width: 200px;
    margin: 0 20px;
  }
  
  .loading-bar {
    width: 100%;
    height: 3px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    overflow: hidden;
  }
  
  .loading-progress {
    height: 100%;
    background: var(--primary-color, #4a9eff);
    transition: width 0.3s ease;
  }
  
  .loading-text {
    font-size: 10px;
    margin-left: 8px;
    color: var(--text-muted, #888);
  }
  
  .transfer-stats {
    gap: 16px;
  }
  
  .stat {
    font-family: monospace;
    font-size: 11px;
  }
  
  .stat.download {
    color: var(--success-color, #4caf50);
  }
  
  .stat.upload {
    color: var(--info-color, #2196f3);
  }
  
  .details-popup {
    position: absolute;
    bottom: 100%;
    left: 12px;
    margin-bottom: 4px;
    width: 280px;
    background: var(--popup-bg, #252525);
    border: 1px solid var(--border-color, #444);
    border-radius: 8px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
    z-index: 1000;
    overflow: hidden;
  }
  
  .details-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    background: rgba(0, 0, 0, 0.2);
    border-bottom: 1px solid var(--border-color, #333);
  }
  
  .details-header h3 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary, #fff);
  }
  
  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary, #999);
    font-size: 18px;
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }
  
  .close-btn:hover {
    color: var(--text-primary, #fff);
  }
  
  .details-content {
    padding: 12px;
  }
  
  .detail-row {
    display: flex;
    justify-content: space-between;
    padding: 6px 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }
  
  .detail-row:last-of-type {
    border-bottom: none;
  }
  
  .label {
    color: var(--text-muted, #888);
    font-size: 11px;
  }
  
  .value {
    color: var(--text-primary, #fff);
    font-size: 11px;
    font-family: monospace;
  }
  
  .value.connected {
    color: var(--success-color, #4caf50);
  }
  
  .value.disconnected {
    color: var(--error-color, #f44336);
  }
  
  .details-actions {
    display: flex;
    gap: 8px;
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--border-color, #333);
  }
  
  .action-btn {
    flex: 1;
    padding: 6px 12px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid var(--border-color, #444);
    border-radius: 4px;
    color: var(--text-primary, #fff);
    font-size: 11px;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .action-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    border-color: var(--primary-color, #4a9eff);
  }
  
  .action-btn.rotate:hover {
    border-color: var(--warning-color, #f0a000);
  }
  
  /* Utility classes */
  .text-green-500 { color: #22c55e; }
  .text-yellow-500 { color: #eab308; }
  .text-red-500 { color: #ef4444; }
  .text-gray-400 { color: #9ca3af; }
</style>
