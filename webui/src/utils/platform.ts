/**
 * Platform detection utilities for desktop vs web mode.
 */

export const isDesktop = import.meta.env.VITE_APP_MODE === 'desktop';

export const API_BASE_URL = isDesktop
  ? 'http://localhost:3000'
  : import.meta.env.VITE_API_URL || 'http://localhost:3000';

export const WS_URL = isDesktop
  ? 'ws://localhost:3000/ws/chat'
  : import.meta.env.VITE_WS_URL || 'ws://localhost:3000/ws/chat';

export function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export function isWeb(): boolean {
  return !isTauri();
}

export function getPlatform(): 'desktop' | 'web' | 'mobile' {
  if (isTauri()) return 'desktop';
  return 'web';
}
