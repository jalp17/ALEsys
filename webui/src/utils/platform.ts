/**
 * Detecta si se está ejecutando en modo desktop (Tauri) o web
 */
export const isDesktop = import.meta.env.VITE_APP_MODE === 'desktop';

/**
 * URL base de la API
 */
export const API_BASE_URL = isDesktop 
  ? 'http://localhost:3000' 
  : import.meta.env.VITE_API_URL || 'http://localhost:3000';

/**
 * URL del WebSocket
 */
export const WS_URL = isDesktop
  ? 'ws://localhost:3000/ws/chat'
  : import.meta.env.VITE_WS_URL || 'ws://localhost:3000/ws/chat';