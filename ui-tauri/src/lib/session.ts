/**
 * Session Management Module
 * 
 * Provides functionality for saving and restoring browser sessions including:
 * - Tab state persistence
 * - Session auto-save
 * - Session import/export
 */

import { writable, get } from 'svelte/store';
import { persist } from './persist';
import type { Tab } from './types';

/**
 * Represents a saved tab in a session
 */
export interface SessionTab {
  id: string;
  url: string;
  title: string;
  favicon?: string;
  scrollPosition?: { x: number; y: number };
  zoomLevel?: number;
  proxyConfig?: {
    ip: string;
    port: number;
    protocol: string;
    country?: string;
  };
  createdAt: string;
  lastAccessed: string;
}

/**
 * Represents a browser session
 */
export interface BrowserSession {
  id: string;
  name: string;
  tabs: SessionTab[];
  activeTabId?: string;
  createdAt: string;
  updatedAt: string;
  metadata?: {
    windowSize?: { width: number; height: number };
    windowPosition?: { x: number; y: number };
  };
}

/**
 * Session manager state
 */
interface SessionManagerState {
  currentSession: BrowserSession | null;
  savedSessions: BrowserSession[];
  autoSaveEnabled: boolean;
  lastAutoSave: string | null;
}

const defaultState: SessionManagerState = {
  currentSession: null,
  savedSessions: [],
  autoSaveEnabled: true,
  lastAutoSave: null,
};

// Persistent session store
export const sessionStore = persist(
  writable<SessionManagerState>(defaultState),
  'virtual-ip-browser-sessions'
);

// Auto-save interval handle
let autoSaveInterval: ReturnType<typeof setInterval> | null = null;

/**
 * Generate a unique session ID
 */
function generateSessionId(): string {
  return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * Convert a Tab to a SessionTab
 */
export function tabToSessionTab(tab: Tab): SessionTab {
  return {
    id: tab.id,
    url: tab.url || 'about:blank',
    title: tab.title || 'New Tab',
    favicon: tab.favicon,
    zoomLevel: tab.zoomLevel,
    proxyConfig: tab.proxyConfig ? {
      ip: tab.proxyConfig.ip,
      port: tab.proxyConfig.port,
      protocol: tab.proxyConfig.protocol,
      country: tab.proxyConfig.country,
    } : undefined,
    createdAt: tab.createdAt || new Date().toISOString(),
    lastAccessed: new Date().toISOString(),
  };
}

/**
 * Create a new session from current tabs
 */
export function createSession(tabs: Tab[], activeTabId?: string, name?: string): BrowserSession {
  const now = new Date().toISOString();
  return {
    id: generateSessionId(),
    name: name || `Session ${new Date().toLocaleDateString()}`,
    tabs: tabs.map(tabToSessionTab),
    activeTabId,
    createdAt: now,
    updatedAt: now,
  };
}

/**
 * Save current session
 */
export function saveCurrentSession(tabs: Tab[], activeTabId?: string, name?: string): void {
  const session = createSession(tabs, activeTabId, name);
  sessionStore.update(state => ({
    ...state,
    currentSession: session,
    lastAutoSave: new Date().toISOString(),
  }));
}

/**
 * Save session to saved sessions list
 */
export function saveSessionToList(tabs: Tab[], activeTabId?: string, name?: string): BrowserSession {
  const session = createSession(tabs, activeTabId, name);
  sessionStore.update(state => ({
    ...state,
    savedSessions: [...state.savedSessions, session],
  }));
  return session;
}

/**
 * Get the current session
 */
export function getCurrentSession(): BrowserSession | null {
  return get(sessionStore).currentSession;
}

/**
 * Get all saved sessions
 */
export function getSavedSessions(): BrowserSession[] {
  return get(sessionStore).savedSessions;
}

/**
 * Delete a saved session
 */
export function deleteSession(sessionId: string): void {
  sessionStore.update(state => ({
    ...state,
    savedSessions: state.savedSessions.filter(s => s.id !== sessionId),
  }));
}

/**
 * Rename a saved session
 */
export function renameSession(sessionId: string, newName: string): void {
  sessionStore.update(state => ({
    ...state,
    savedSessions: state.savedSessions.map(s =>
      s.id === sessionId ? { ...s, name: newName, updatedAt: new Date().toISOString() } : s
    ),
  }));
}

/**
 * Export session to JSON
 */
export function exportSession(session: BrowserSession): string {
  return JSON.stringify(session, null, 2);
}

/**
 * Export all sessions to JSON
 */
export function exportAllSessions(): string {
  const state = get(sessionStore);
  return JSON.stringify({
    version: '1.0',
    exportedAt: new Date().toISOString(),
    currentSession: state.currentSession,
    savedSessions: state.savedSessions,
  }, null, 2);
}

/**
 * Import session from JSON
 */
export function importSession(json: string): BrowserSession {
  const session = JSON.parse(json) as BrowserSession;
  // Validate session structure
  if (!session.id || !session.tabs || !Array.isArray(session.tabs)) {
    throw new Error('Invalid session format');
  }
  // Add to saved sessions
  sessionStore.update(state => ({
    ...state,
    savedSessions: [...state.savedSessions, session],
  }));
  return session;
}

/**
 * Import multiple sessions from JSON
 */
export function importSessions(json: string): void {
  const data = JSON.parse(json);
  if (data.savedSessions && Array.isArray(data.savedSessions)) {
    sessionStore.update(state => ({
      ...state,
      savedSessions: [...state.savedSessions, ...data.savedSessions],
    }));
  }
}

/**
 * Start auto-save interval
 */
export function startAutoSave(intervalMinutes: number, getTabsFn: () => Tab[], getActiveTabIdFn: () => string | undefined): void {
  stopAutoSave();
  if (intervalMinutes <= 0) return;
  
  autoSaveInterval = setInterval(() => {
    const tabs = getTabsFn();
    const activeTabId = getActiveTabIdFn();
    saveCurrentSession(tabs, activeTabId);
    console.log('Session auto-saved');
  }, intervalMinutes * 60 * 1000);
}

/**
 * Stop auto-save interval
 */
export function stopAutoSave(): void {
  if (autoSaveInterval) {
    clearInterval(autoSaveInterval);
    autoSaveInterval = null;
  }
}

/**
 * Clear all session data
 */
export function clearAllSessions(): void {
  sessionStore.set(defaultState);
}

/**
 * Get session by ID
 */
export function getSessionById(sessionId: string): BrowserSession | undefined {
  const state = get(sessionStore);
  if (state.currentSession?.id === sessionId) {
    return state.currentSession;
  }
  return state.savedSessions.find(s => s.id === sessionId);
}

/**
 * Update session tabs
 */
export function updateSessionTabs(sessionId: string, tabs: Tab[]): void {
  sessionStore.update(state => {
    const now = new Date().toISOString();
    const sessionTabs = tabs.map(tabToSessionTab);
    
    if (state.currentSession?.id === sessionId) {
      return {
        ...state,
        currentSession: {
          ...state.currentSession,
          tabs: sessionTabs,
          updatedAt: now,
        },
      };
    }
    
    return {
      ...state,
      savedSessions: state.savedSessions.map(s =>
        s.id === sessionId ? { ...s, tabs: sessionTabs, updatedAt: now } : s
      ),
    };
  });
}
