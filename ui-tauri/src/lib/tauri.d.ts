/**
 * Type declarations for Tauri-specific window properties
 */

interface TauriMetadata {
  __currentWindow?: {
    label: string;
  };
}

declare global {
  interface Window {
    __TAURI_METADATA__?: TauriMetadata;
  }
}

export {};
