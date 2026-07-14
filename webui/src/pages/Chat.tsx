import { useState, useRef, useEffect } from 'react';
import { API_BASE_URL } from '../utils/platform';

interface Message {
  role: 'user' | 'assistant';
  content: string;
  sources?: Array<{
    fragment_id: number;
    path: string;
    similarity: number;
  }>;
  isStreaming?: boolean;
}

export function Chat() {
  const [query, setQuery] = useState('');
  const [messages, setMessages] = useState<Message[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [useWebSocket, setUseWebSocket] = useState(true);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const wsRef = useRef<WebSocket | null>(null);

  // Auto-scroll al final
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  // Limpiar WebSocket al desmontar
  useEffect(() => {
    return () => {
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, []);

  const addMessage = (message: Message) => {
    setMessages(prev => [...prev, message]);
  };

  const updateLastMessage = (content: string, sources?: Message['sources']) => {
    setMessages(prev => {
      const newMessages = [...prev];
      const lastMsg = newMessages[newMessages.length - 1];
      if (lastMsg && lastMsg.role === 'assistant') {
        newMessages[newMessages.length - 1] = {
          ...lastMsg,
          content: lastMsg.content + content,
          sources: sources || lastMsg.sources,
        };
      }
      return newMessages;
    });
  };

  const markLastMessageDone = () => {
    setMessages(prev => {
      const newMessages = [...prev];
      const lastMsg = newMessages[newMessages.length - 1];
      if (lastMsg && lastMsg.role === 'assistant') {
        newMessages[newMessages.length - 1] = {
          ...lastMsg,
          isStreaming: false,
        };
      }
      return newMessages;
    });
  };

  const sendViaWebSocket = (query: string) => {
    // Conectar WebSocket si no está conectado
    if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) {
      const wsUrl = API_BASE_URL.replace(/^http/, 'ws') + '/ws/chat';
      wsRef.current = new WebSocket(wsUrl);

      wsRef.current.onmessage = (event) => {
        const data = JSON.parse(event.data);

        switch (data.type) {
          case 'start':
            // Inicio de respuesta
            break;
          case 'chunk':
            updateLastMessage(data.content || '');
            break;
          case 'done':
            updateLastMessage('', data.sources);
            markLastMessageDone();
            setIsLoading(false);
            break;
          case 'error':
            updateLastMessage(`\n\nError: ${data.error}`);
            markLastMessageDone();
            setIsLoading(false);
            break;
        }
      };

      wsRef.current.onerror = (error) => {
        console.error('WebSocket error:', error);
        updateLastMessage('\n\nError: Conexión WebSocket fallida');
        markLastMessageDone();
        setIsLoading(false);
      };

      wsRef.current.onclose = () => {
        wsRef.current = null;
      };
    }

    // Enviar mensaje cuando esté listo
    const send = () => {
      if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify({
          type: 'chat',
          query,
        }));
      } else {
        setTimeout(send, 100);
      }
    };
    send();
  };

  const sendViaHTTP = async (query: string) => {
    try {
      const response = await fetch(`${API_BASE_URL}/api/chat`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ query }),
      });

      const data = await response.json();

      updateLastMessage(data.response || 'Sin respuesta', data.sources);
      markLastMessageDone();
    } catch (error) {
      console.error('Error:', error);
      updateLastMessage('\n\nError: No se pudo conectar con el servidor');
      markLastMessageDone();
    } finally {
      setIsLoading(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!query.trim() || isLoading) return;

    const userMessage = query;
    setMessages(prev => [...prev, { role: 'user', content: userMessage }]);
    setQuery('');
    setIsLoading(true);

    // Agregar mensaje vacío del asistente
    addMessage({ role: 'assistant', content: '', isStreaming: true });

    if (useWebSocket) {
      sendViaWebSocket(userMessage);
    } else {
      await sendViaHTTP(userMessage);
    }
  };

  return (
    <div className="max-w-4xl mx-auto h-full flex flex-col">
      {/* Chat messages */}
      <div className="flex-1 overflow-y-auto mb-4 space-y-4">
        {messages.length === 0 ? (
          <div className="text-center text-gray-400 mt-20">
            <h2 className="text-2xl font-semibold mb-2">
              Bienvenido a ALEsys
            </h2>
            <p className="text-lg">
              Haz una pregunta sobre tu base de conocimiento
            </p>
            <div className="mt-4 text-sm text-gray-500">
              <p>Modo de conexión: 
                <button 
                  onClick={() => setUseWebSocket(!useWebSocket)}
                  className="ml-2 text-primary-400 hover:text-primary-300"
                >
                  {useWebSocket ? 'WebSocket' : 'HTTP'}
                </button>
              </p>
            </div>
          </div>
        ) : (
          messages.map((msg, idx) => (
            <div
              key={idx}
              className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}
            >
              <div
                className={`max-w-[80%] p-4 rounded-lg ${
                  msg.role === 'user'
                    ? 'bg-primary-600 text-white'
                    : 'bg-dark-800 text-gray-100'
                }`}
              >
                <div className="whitespace-pre-wrap">{msg.content}</div>
                {msg.sources && msg.sources.length > 0 && (
                  <div className="mt-3 pt-3 border-t border-gray-700">
                    <p className="text-xs text-gray-400 mb-1">Fuentes:</p>
                    <div className="flex flex-wrap gap-2">
                      {msg.sources.map((source, i) => (
                        <span
                          key={i}
                          className="text-xs bg-dark-700 px-2 py-1 rounded"
                        >
                          {source.path} ({(source.similarity * 100).toFixed(0)}%)
                        </span>
                      ))}
                    </div>
                  </div>
                )}
                {msg.isStreaming && (
                  <span className="inline-block w-2 h-4 ml-1 bg-primary-400 animate-pulse"></span>
                )}
              </div>
            </div>
          ))
        )}
        
        {isLoading && messages[messages.length - 1]?.role !== 'assistant' && (
          <div className="flex justify-start">
            <div className="bg-dark-800 p-4 rounded-lg">
              <div className="flex gap-2">
                <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce"></div>
                <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '0.2s' }}></div>
                <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '0.4s' }}></div>
              </div>
            </div>
          </div>
        )}
        
        <div ref={messagesEndRef} />
      </div>

      {/* Input form */}
      <form onSubmit={handleSubmit} className="flex gap-3">
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Escribe tu pregunta..."
          className="flex-1 px-4 py-3 bg-dark-800 border border-gray-700 rounded-lg focus:outline-none focus:border-primary-500 text-white placeholder-gray-500"
          disabled={isLoading}
        />
        <button
          type="submit"
          disabled={isLoading || !query.trim()}
          className="px-6 py-3 bg-primary-600 rounded-lg hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed transition font-semibold"
        >
          Enviar
        </button>
      </form>
    </div>
  );
}