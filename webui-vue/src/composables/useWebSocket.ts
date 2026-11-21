import { ref, onMounted, onUnmounted } from 'vue'

export function useWebSocket(url: string, onMessage: (data: any) => void) {
  const ws = ref<WebSocket | null>(null)
  const connected = ref(false)
  const reconnectAttempts = ref(0)
  const maxReconnects = 5

  const connect = () => {
    try {
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
      const wsUrl = url.startsWith('ws') ? url : `${protocol}//${window.location.host}${url}`
      ws.value = new WebSocket(wsUrl)

      ws.value.onopen = () => {
        connected.value = true
        reconnectAttempts.value = 0
      }

      ws.value.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data)
          onMessage(data)
        } catch {
          onMessage(event.data)
        }
      }

      ws.value.onclose = () => {
        connected.value = false
        if (reconnectAttempts.value < maxReconnects) {
          reconnectAttempts.value++
          const delay = Math.min(1000 * Math.pow(2, reconnectAttempts.value), 30000)
          setTimeout(connect, delay)
        }
      }

      ws.value.onerror = () => {
        connected.value = false
      }
    } catch {
      connected.value = false
    }
  }

  const send = (data: any) => {
    if (ws.value?.readyState === WebSocket.OPEN) {
      ws.value.send(JSON.stringify(data))
    }
  }

  const disconnect = () => {
    reconnectAttempts.value = maxReconnects // Prevent reconnect
    ws.value?.close()
    ws.value = null
    connected.value = false
  }

  onMounted(connect)
  onUnmounted(disconnect)

  return { connected, send, disconnect }
}
