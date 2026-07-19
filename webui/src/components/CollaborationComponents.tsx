import { useState, useEffect } from 'react';

interface UserPresence {
  user_id: string;
  username: string;
  cursor_position?: number;
  selection?: [number, number];
  status: 'Active' | 'Idle' | 'Typing' | 'Offline';
  color: string;
}

interface PresencePanelProps {
  users: UserPresence[];
  currentUserId: string;
}

export function PresencePanel({ users, currentUserId }: PresencePanelProps) {
  return (
    <div className="border rounded-lg p-3 bg-dark-800">
      <h3 className="text-sm font-semibold mb-2 text-gray-400">
        Online ({users.filter((u) => u.status !== 'Offline').length})
      </h3>
      <div className="space-y-2">
        {users.map((user) => (
          <div
            key={user.user_id}
            className={`flex items-center gap-2 text-sm ${
              user.user_id === currentUserId ? 'font-semibold' : ''
            }`}
          >
            <div
              className="w-2 h-2 rounded-full"
              style={{ backgroundColor: user.color }}
            />
            <span className="flex-1 truncate">
              {user.username}
              {user.user_id === currentUserId && ' (you)'}
            </span>
            <span
              className={`text-xs px-1.5 py-0.5 rounded ${
                user.status === 'Active'
                  ? 'bg-green-100 text-green-800'
                  : user.status === 'Typing'
                  ? 'bg-yellow-100 text-yellow-800'
                  : 'bg-gray-100 text-gray-600'
              }`}
            >
              {user.status}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}

interface CollaborativeEditorProps {
  documentId: string;
  initialContent: string;
  userId: string;
  username: string;
}

export function CollaborativeEditor({
  documentId,
  initialContent,
  userId,
  username,
}: CollaborativeEditorProps) {
  const [content, setContent] = useState(initialContent);
  const [users, setUsers] = useState<UserPresence[]>([]);
  const [cursors, setCursors] = useState<
    Map<string, { position: number; color: string; username: string }>
  >(new Map());

  useEffect(() => {
    // Connect to WebSocket for collaboration
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws/collab/${documentId}`;
    const ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      console.log('Connected to collaboration server');
    };

    ws.onmessage = (event) => {
      const msg = JSON.parse(event.data);
      switch (msg.type) {
        case 'presence_update':
          setUsers((prev) => {
            const idx = prev.findIndex((u) => u.user_id === msg.user_id);
            if (idx >= 0) {
              const updated = [...prev];
              updated[idx] = msg.presence;
              return updated;
            }
            return [...prev, msg.presence];
          });
          break;
        case 'cursor_sync':
          setCursors((prev) => {
            const next = new Map(prev);
            next.set(msg.user_id, {
              position: msg.position,
              color: msg.color,
              username: msg.username,
            });
            return next;
          });
          break;
        case 'document_sync':
          setContent(msg.content);
          break;
      }
    };

    return () => ws.close();
  }, [documentId]);

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const newContent = e.target.value;
    const position = e.target.selectionStart;

    setContent(newContent);

    // Send operation to server
    // In production, this would use OT
    const ws = new WebSocket(
      `ws://${window.location.host}/ws/collab/${documentId}`
    );
    ws.onopen = () => {
      ws.send(
        JSON.stringify({
          type: 'operation',
          user_id: userId,
          position,
          action: 'insert',
          content: newContent.slice(content.length),
        })
      );
      ws.close();
    };
  };

  return (
    <div className="flex gap-4">
      <div className="flex-1">
        <textarea
          value={content}
          onChange={handleChange}
          className="w-full h-96 p-4 font-mono text-sm bg-dark-800 border border-gray-700 rounded-lg focus:outline-none focus:border-primary-500 text-white"
          spellCheck={false}
        />
        {/* Cursor indicators */}
        <div className="relative">
          {Array.from(cursors.entries()).map(([uid, cursor]) => {
            if (uid === userId) return null;
            return (
              <div
                key={uid}
                className="absolute text-xs px-1 py-0.5 rounded"
                style={{
                  backgroundColor: cursor.color,
                  left: `${cursor.position * 0.6}px`,
                  top: '-20px',
                }}
              >
                {cursor.username}
              </div>
            );
          })}
        </div>
      </div>
      <div className="w-48">
        <PresencePanel users={users} currentUserId={userId} />
      </div>
    </div>
  );
}
