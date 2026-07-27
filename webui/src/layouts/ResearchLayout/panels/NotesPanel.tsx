import { useState, useEffect } from 'react'

export type NoteType = 'summary' | 'critique' | 'question' | 'idea'

export interface Note {
  id: string
  title: string
  content: string
  type: NoteType
  tags: string[]
  chapter_id?: string
  citation_id?: string
  created_at: string
  updated_at: string
}

const STORAGE_KEY = 'alesys_research_notes'

const noteService = {
  getAll(): Note[] {
    if (typeof window === 'undefined') return []
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return []
    try {
      return JSON.parse(raw) as Note[]
    } catch {
      return []
    }
  },

  save(notes: Note[]): void {
    if (typeof window === 'undefined') return
    localStorage.setItem(STORAGE_KEY, JSON.stringify(notes))
  },

  add(note: Omit<Note, 'id' | 'created_at' | 'updated_at'>): Note {
    const now = new Date().toISOString()
    const newNote: Note = {
      ...note,
      id: crypto.randomUUID(),
      created_at: now,
      updated_at: now,
    }
    const notes = this.getAll()
    notes.push(newNote)
    this.save(notes)
    return newNote
  },

  update(id: string, patch: Partial<Omit<Note, 'id' | 'created_at'>>): Note | undefined {
    const notes = this.getAll()
    const index = notes.findIndex((n) => n.id === id)
    if (index === -1) return undefined

    notes[index] = {
      ...notes[index],
      ...patch,
      updated_at: new Date().toISOString(),
    }
    this.save(notes)
    return notes[index]
  },

  remove(id: string): void {
    const notes = this.getAll().filter((n) => n.id !== id)
    this.save(notes)
  },
}

type Props = {
  onToggleFullscreen?: () => void
  selectedChapterId?: string
  selectedCitationId?: string
}

const NOTE_TYPES: { value: NoteType; label: string; color: string }[] = [
  { value: 'summary', label: 'Resumen', color: 'bg-blue-900 text-blue-300' },
  { value: 'critique', label: 'Crítica', color: 'bg-red-900 text-red-300' },
  { value: 'question', label: 'Pregunta', color: 'bg-yellow-900 text-yellow-300' },
  { value: 'idea', label: 'Idea', color: 'bg-green-900 text-green-300' },
]

export default function NotesPanel({ onToggleFullscreen: _onToggleFullscreen, selectedChapterId, selectedCitationId }: Props) {
  const [notes, setNotes] = useState<Note[]>([])
  const [filter, setFilter] = useState<'all' | NoteType>('all')
  const [search, setSearch] = useState('')
  const [editingId, setEditingId] = useState<string | null>(null)
  const [draft, setDraft] = useState({ title: '', content: '', type: 'idea' as NoteType, tags: '' })

  useEffect(() => {
    setNotes(noteService.getAll())
  }, [])

  useEffect(() => {
    setNotes(noteService.getAll())
  }, [selectedChapterId, selectedCitationId])

  const filteredNotes = notes
    .filter((note) => {
      if (filter !== 'all' && note.type !== filter) return false
      if (search) {
        const q = search.toLowerCase()
        return (
          note.title.toLowerCase().includes(q) ||
          note.content.toLowerCase().includes(q) ||
          note.tags.some((t) => t.toLowerCase().includes(q))
        )
      }
      return true
    })
    .sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime())

  const handleCreate = () => {
    if (!draft.title.trim() && !draft.content.trim()) return

    noteService.add({
      title: draft.title.trim() || 'Sin título',
      content: draft.content.trim(),
      type: draft.type,
      tags: draft.tags
        .split(',')
        .map((t) => t.trim())
        .filter(Boolean),
      chapter_id: selectedChapterId,
      citation_id: selectedCitationId,
    })

    setNotes(noteService.getAll())
    setDraft({ title: '', content: '', type: 'idea', tags: '' })
  }

  const handleUpdate = (id: string, patch: Partial<Note>) => {
    const updated = noteService.update(id, patch)
    if (updated) {
      setNotes(noteService.getAll())
      setEditingId(null)
    }
  }

  const handleDelete = (id: string) => {
    noteService.remove(id)
    setNotes(noteService.getAll())
  }

  const typeColor = (type: NoteType) => NOTE_TYPES.find((t) => t.value === type)?.color || 'bg-gray-700 text-gray-300'

  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-gray-700 px-3 py-2">
        <h2 className="text-sm font-semibold text-gray-200">Notas y Anotaciones</h2>
        <p className="mt-1 text-xs text-gray-400">
          Notas vinculadas a capítulos y citas.
        </p>
      </div>

      <div className="border-b border-gray-700 px-3 py-2">
        <div className="flex items-center gap-2">
          <input
            type="text"
            placeholder="Buscar notas..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="flex-1 rounded border border-gray-700 bg-dark-800 px-2 py-1 text-xs text-gray-200 placeholder-gray-500 focus:border-primary-500 focus:outline-none"
          />
          <select
            value={filter}
            onChange={(e) => setFilter(e.target.value as 'all' | NoteType)}
            className="rounded border border-gray-700 bg-dark-800 px-2 py-1 text-xs text-gray-200 focus:border-primary-500 focus:outline-none"
          >
            <option value="all">Todas</option>
            {NOTE_TYPES.map((t) => (
              <option key={t.value} value={t.value}>
                {t.label}
              </option>
            ))}
          </select>
        </div>

        <div className="mt-2 space-y-1">
          <input
            type="text"
            placeholder="Título..."
            value={draft.title}
            onChange={(e) => setDraft((prev) => ({ ...prev, title: e.target.value }))}
            className="w-full rounded border border-gray-700 bg-dark-800 px-2 py-1 text-xs text-gray-200 placeholder-gray-500 focus:border-primary-500 focus:outline-none"
          />
          <textarea
            placeholder="Contenido..."
            value={draft.content}
            onChange={(e) => setDraft((prev) => ({ ...prev, content: e.target.value }))}
            className="h-16 w-full resize-none rounded border border-gray-700 bg-dark-800 px-2 py-1 text-xs text-gray-200 placeholder-gray-500 focus:border-primary-500 focus:outline-none"
          />
          <div className="flex items-center gap-2">
            <select
              value={draft.type}
              onChange={(e) => setDraft((prev) => ({ ...prev, type: e.target.value as NoteType }))}
              className="flex-1 rounded border border-gray-700 bg-dark-800 px-2 py-1 text-xs text-gray-200 focus:border-primary-500 focus:outline-none"
            >
              {NOTE_TYPES.map((t) => (
                <option key={t.value} value={t.value}>
                  {t.label}
                </option>
              ))}
            </select>
            <input
              type="text"
              placeholder="Tags (coma)..."
              value={draft.tags}
              onChange={(e) => setDraft((prev) => ({ ...prev, tags: e.target.value }))}
              className="flex-1 rounded border border-gray-700 bg-dark-800 px-2 py-1 text-xs text-gray-200 placeholder-gray-500 focus:border-primary-500 focus:outline-none"
            />
            <button
              onClick={handleCreate}
              className="rounded bg-primary-500 px-2 py-1 text-xs text-white hover:bg-primary-600"
              type="button"
            >
              Guardar
            </button>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        {filteredNotes.length === 0 ? (
          <div className="p-3 text-xs text-gray-400">Sin notas.</div>
        ) : (
          <div className="divide-y divide-gray-800">
            {filteredNotes.map((note) => (
              <div key={note.id} className="p-3">
                {editingId === note.id ? (
                  <div className="space-y-1">
                    <input
                      type="text"
                      defaultValue={note.title}
                      onBlur={(e) => handleUpdate(note.id, { title: e.target.value })}
                      className="w-full rounded border border-gray-700 bg-dark-800 px-2 py-1 text-xs text-gray-200 focus:border-primary-500 focus:outline-none"
                    />
                    <textarea
                      defaultValue={note.content}
                      onBlur={(e) => handleUpdate(note.id, { content: e.target.value })}
                      className="h-20 w-full resize-none rounded border border-gray-700 bg-dark-800 px-2 py-1 text-xs text-gray-200 focus:border-primary-500 focus:outline-none"
                    />
                    <div className="flex items-center gap-2">
                      <select
                        defaultValue={note.type}
                        onBlur={(e) => handleUpdate(note.id, { type: e.target.value as NoteType })}
                        className="rounded border border-gray-700 bg-dark-800 px-2 py-1 text-xs text-gray-200"
                      >
                        {NOTE_TYPES.map((t) => (
                          <option key={t.value} value={t.value}>
                            {t.label}
                          </option>
                        ))}
                      </select>
                      <button
                        onClick={() => setEditingId(null)}
                        className="rounded border border-gray-700 px-2 py-1 text-xs text-gray-300"
                        type="button"
                      >
                        Hecho
                      </button>
                    </div>
                  </div>
                ) : (
                  <div>
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <span className={`text-xs px-1.5 py-0.5 rounded ${typeColor(note.type)}`}>
                          {NOTE_TYPES.find((t) => t.value === note.type)?.label}
                        </span>
                        <span className="text-xs font-medium text-gray-200">{note.title}</span>
                      </div>
                      <div className="flex items-center gap-1">
                        <button
                          onClick={() => setEditingId(note.id)}
                          className="rounded px-1.5 py-1 text-xs text-gray-400 hover:text-gray-200"
                          type="button"
                        >
                          Editar
                        </button>
                        <button
                          onClick={() => handleDelete(note.id)}
                          className="rounded px-1.5 py-1 text-xs text-red-400 hover:text-red-300"
                          type="button"
                        >
                          Borrar
                        </button>
                      </div>
                    </div>
                    <p className="mt-1 text-xs text-gray-300">{note.content}</p>
                    <div className="mt-1 flex items-center gap-2">
                      {note.tags.map((tag) => (
                        <span key={tag} className="text-xs text-gray-500">#{tag}</span>
                      ))}
                      <span className="text-xs text-gray-600">
                        {new Date(note.updated_at).toLocaleString()}
                      </span>
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
