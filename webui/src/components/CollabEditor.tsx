import { useRef, useEffect } from 'react'
import * as Y from 'yjs'
import { WebsocketProvider } from 'y-websocket'
import { MonacoBinding } from 'y-monaco'
import Editor from '@monaco-editor/react'

interface CollabEditorProps {
  roomId: string
  wsUrl: string
  userName: string
  initialContent?: string
  onChange?: (content: string) => void
}

export default function CollabEditor({ roomId, wsUrl, userName, initialContent = '', onChange }: CollabEditorProps) {
  const editorRef = useRef<any>(null)
  const ydocRef = useRef<Y.Doc | null>(null)
  const providerRef = useRef<WebsocketProvider | null>(null)
  const bindingRef = useRef<MonacoBinding | null>(null)

  useEffect(() => {
    const ydoc = new Y.Doc()
    const provider = new WebsocketProvider(wsUrl, roomId, ydoc)
    const ytext = ydoc.getText('content')

    if (initialContent && ytext.length === 0) {
      ytext.insert(0, initialContent)
    }

    provider.awareness.setLocalStateField('user', {
      name: userName,
      color: `#${Math.floor(Math.random() * 16777215).toString(16)}`,
    })

    ydocRef.current = ydoc
    providerRef.current = provider

    return () => {
      bindingRef.current?.destroy()
      provider.destroy()
      ydoc.destroy()
    }
  }, [roomId, wsUrl, userName, initialContent])

  const handleEditorMount = (ed: any) => {
    editorRef.current = ed
    if (ydocRef.current) {
      const ytext = ydocRef.current.getText('content')
      const model = ed.getModel()
      if (model) {
        bindingRef.current = new MonacoBinding(ytext, model, new Set([ed]), providerRef.current?.awareness)
      }
    }
    ed.onDidChangeModelContent(() => {
      onChange?.(ed.getValue())
    })
  }

  return (
    <div className="h-full w-full">
      <Editor
        height="100%"
        defaultLanguage="markdown"
        onMount={handleEditorMount}
        theme="vs-dark"
      />
    </div>
  )
}
