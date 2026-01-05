<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  interface BackupInfo {
    id: string;
    filename: string;
    created_at: string;
    size_bytes: number;
    is_encrypted: boolean;
  }

  // Props
  let {
    backup,
    onRestore,
    onDelete
  }: {
    backup: BackupInfo;
    onRestore?: (backup: BackupInfo) => void;
    onDelete?: (backup: BackupInfo) => void;
  } = $props();

  // Local state
  let isRestoring = $state(false);
  let isDeleting = $state(false);

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  async function handleRestore() {
    if (isRestoring) return;
    isRestoring = true;
    try {
      if (onRestore) onRestore(backup);
    } finally {
      isRestoring = false;
    }
  }

  async function handleDelete() {
    if (isDeleting) return;
    if (!confirm(`Are you sure you want to delete backup "${backup.filename}"?`)) {
      return;
    }
    isDeleting = true;
    try {
      if (onDelete) onDelete(backup);
    } finally {
      isDeleting = false;
    }
  }
</script>

<div class="backup-item">
  <div class="backup-icon">
    {#if backup.is_encrypted}
      🔒
    {:else}
      📁
    {/if}
  </div>
  
  <div class="backup-info">
    <span class="backup-name">{backup.filename}</span>
    <div class="backup-meta">
      <span class="backup-date">{formatDate(backup.created_at)}</span>
      <span class="backup-size">{formatSize(backup.size_bytes)}</span>
      {#if backup.is_encrypted}
        <span class="backup-encrypted">Encrypted</span>
      {/if}
    </div>
  </div>
  
  <div class="backup-actions">
    <button 
      class="btn-restore" 
      onclick={handleRestore}
      disabled={isRestoring}
      title="Restore this backup"
    >
      {isRestoring ? '...' : '↩️'}
    </button>
    <button 
      class="btn-delete" 
      onclick={handleDelete}
      disabled={isDeleting}
      title="Delete this backup"
    >
      {isDeleting ? '...' : '🗑️'}
    </button>
  </div>
</div>

<style>
  .backup-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: #1a1a2e;
    border: 1px solid #2a2a45;
    border-radius: 8px;
    transition: all 0.2s;
  }

  .backup-item:hover {
    border-color: #3a3a55;
    background: #2a2a45;
  }

  .backup-icon {
    font-size: 24px;
    flex-shrink: 0;
  }

  .backup-info {
    flex: 1;
    min-width: 0;
  }

  .backup-name {
    display: block;
    font-size: 14px;
    font-weight: 500;
    color: #e0e0e0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .backup-meta {
    display: flex;
    gap: 12px;
    margin-top: 4px;
  }

  .backup-date, .backup-size {
    font-size: 12px;
    color: #808090;
  }

  .backup-encrypted {
    font-size: 11px;
    color: #10b981;
    background: rgba(16, 185, 129, 0.1);
    padding: 2px 6px;
    border-radius: 4px;
  }

  .backup-actions {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }

  .btn-restore, .btn-delete {
    padding: 8px;
    background: transparent;
    border: 1px solid #3a3a55;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    transition: all 0.2s;
  }

  .btn-restore:hover:not(:disabled) {
    background: #4f46e5;
    border-color: #4f46e5;
  }

  .btn-delete:hover:not(:disabled) {
    background: #ef4444;
    border-color: #ef4444;
  }

  .btn-restore:disabled, .btn-delete:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
