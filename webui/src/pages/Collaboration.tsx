import { useState } from 'react';
import { CollaborativeEditor } from '../components/CollaborationComponents';

export default function Collaboration() {
  const [documentId, setDocumentId] = useState('');
  const [joined, setJoined] = useState(false);
  const [username, setUsername] = useState('');

  const handleJoin = () => {
    if (documentId.trim() && username.trim()) {
      setJoined(true);
    }
  };

  if (!joined) {
    return (
      <div className="max-w-md mx-auto mt-20 p-6">
        <h1 className="text-2xl font-bold mb-6">Real-Time Collaboration</h1>
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-1">Your Name</label>
            <input
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              placeholder="Enter your name"
              className="w-full p-2 border rounded"
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Document ID</label>
            <input
              type="text"
              value={documentId}
              onChange={(e) => setDocumentId(e.target.value)}
              placeholder="Enter document ID or create new"
              className="w-full p-2 border rounded"
            />
          </div>
          <button
            onClick={handleJoin}
            disabled={!documentId.trim() || !username.trim()}
            className="w-full px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
          >
            Join Document
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-6xl mx-auto p-6">
      <div className="flex items-center justify-between mb-4">
        <h1 className="text-2xl font-bold">Document: {documentId}</h1>
        <button
          onClick={() => setJoined(false)}
          className="px-4 py-2 bg-gray-200 rounded hover:bg-gray-300"
        >
          Leave
        </button>
      </div>
      <CollaborativeEditor
        documentId={documentId}
        initialContent="# Collaborative Document\n\nStart typing here..."
        userId={`user-${Date.now()}`}
        username={username}
      />
    </div>
  );
}
