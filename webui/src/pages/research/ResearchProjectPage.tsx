import { useState, useEffect } from 'react'
import { useParams } from 'react-router-dom'
import ResearchLayout from '../../layouts/ResearchLayout/ResearchLayout'

interface ProjectMeta {
  id: string
  name: string
  description: string
  status: 'active' | 'archived' | 'draft'
  updated_at: string
}

const STORAGE_KEY = 'alesys_research_projects'

const projectService = {
  getAll(): ProjectMeta[] {
    if (typeof window === 'undefined') return []
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return []
    try {
      return JSON.parse(raw) as ProjectMeta[]
    } catch {
      return []
    }
  },

  save(projects: ProjectMeta[]): void {
    if (typeof window === 'undefined') return
    localStorage.setItem(STORAGE_KEY, JSON.stringify(projects))
  },

  getById(id: string): ProjectMeta | undefined {
    return this.getAll().find((p) => p.id === id)
  },

  create(project: Omit<ProjectMeta, 'id' | 'updated_at'>): ProjectMeta {
    const now = new Date().toISOString()
    const newProject: ProjectMeta = {
      ...project,
      id: crypto.randomUUID(),
      updated_at: now,
    }
    const projects = this.getAll()
    projects.push(newProject)
    this.save(projects)
    return newProject
  },

  update(id: string, patch: Partial<Omit<ProjectMeta, 'id'>>): ProjectMeta | undefined {
    const projects = this.getAll()
    const index = projects.findIndex((p) => p.id === id)
    if (index === -1) return undefined

    projects[index] = {
      ...projects[index],
      ...patch,
      updated_at: new Date().toISOString(),
    }
    this.save(projects)
    return projects[index]
  },

  remove(id: string): void {
    const projects = this.getAll().filter((p) => p.id !== id)
    this.save(projects)
  },
}

export default function ResearchProjectPage() {
  const { projectId } = useParams<{ projectId?: string }>()
  const [projects, setProjects] = useState<ProjectMeta[]>([])
  const [activeProject, setActiveProject] = useState<ProjectMeta | null>(null)
  const [isEditing, setIsEditing] = useState(false)
  const [draft, setDraft] = useState({ name: '', description: '', status: 'draft' as ProjectMeta['status'] })

  useEffect(() => {
    setProjects(projectService.getAll())
  }, [])

  useEffect(() => {
    if (projectId) {
      const project = projectService.getById(projectId)
      setActiveProject(project || null)
      if (project) {
        setDraft({ name: project.name, description: project.description, status: project.status })
      }
    } else {
      setActiveProject(null)
    }
    setIsEditing(false)
  }, [projectId])

  const handleCreate = () => {
    if (!draft.name.trim()) return

    projectService.create({
      name: draft.name.trim(),
      description: draft.description.trim(),
      status: draft.status,
    })

    setProjects(projectService.getAll())
    setDraft({ name: '', description: '', status: 'draft' })
    setIsEditing(false)
  }

  const handleUpdate = () => {
    if (!activeProject || !draft.name.trim()) return

    const updated = projectService.update(activeProject.id, {
      name: draft.name.trim(),
      description: draft.description.trim(),
      status: draft.status,
    })

    if (updated) {
      setProjects(projectService.getAll())
      setActiveProject(updated)
      setIsEditing(false)
    }
  }

  const handleDelete = () => {
    if (!activeProject) return
    if (!confirm('¿Eliminar este proyecto de investigación?')) return

    projectService.remove(activeProject.id)
    setProjects(projectService.getAll())
    setActiveProject(null)
    setIsEditing(false)
  }

  const handleSelectProject = (id: string) => {
    window.location.href = `/research/${id}`
  }

  return (
    <ResearchLayout>
      <div className="flex h-full flex-col">
        <div className="border-b border-gray-700 px-4 py-3">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-sm font-semibold text-gray-200">
                {activeProject ? `Proyecto: ${activeProject.name}` : 'Nuevo proyecto'}
              </h2>
              <p className="mt-1 text-xs text-gray-400">
                Usa los paneles superiores para explorar literatura, citas, notas y síntesis.
              </p>
            </div>
            <div className="flex items-center gap-2">
              {!activeProject && !isEditing && (
                <button
                  onClick={() => setIsEditing(true)}
                  className="rounded bg-primary-500 px-3 py-1.5 text-xs text-white hover:bg-primary-600"
                  type="button"
                >
                  Crear proyecto
                </button>
              )}
              {activeProject && !isEditing && (
                <>
                  <button
                    onClick={() => setIsEditing(true)}
                    className="rounded border border-gray-700 px-3 py-1.5 text-xs text-gray-300 hover:bg-gray-800"
                    type="button"
                  >
                    Editar
                  </button>
                  <button
                    onClick={handleDelete}
                    className="rounded border border-red-800 px-3 py-1.5 text-xs text-red-300 hover:bg-red-900"
                    type="button"
                  >
                    Eliminar
                  </button>
                </>
              )}
            </div>
          </div>
        </div>

        {isEditing && (
          <div className="border-b border-gray-700 px-4 py-3">
            <div className="space-y-2">
              <input
                type="text"
                placeholder="Nombre del proyecto..."
                value={draft.name}
                onChange={(e) => setDraft((prev) => ({ ...prev, name: e.target.value }))}
                className="w-full rounded border border-gray-700 bg-dark-800 px-3 py-2 text-sm text-gray-200 placeholder-gray-500 focus:border-primary-500 focus:outline-none"
                autoFocus
              />
              <textarea
                placeholder="Descripción..."
                value={draft.description}
                onChange={(e) => setDraft((prev) => ({ ...prev, description: e.target.value }))}
                className="w-full rounded border border-gray-700 bg-dark-800 px-3 py-2 text-sm text-gray-200 placeholder-gray-500 focus:border-primary-500 focus:outline-none"
                rows={3}
              />
              <div className="flex items-center gap-2">
                <select
                  value={draft.status}
                  onChange={(e) => setDraft((prev) => ({ ...prev, status: e.target.value as ProjectMeta['status'] }))}
                  className="rounded border border-gray-700 bg-dark-800 px-3 py-2 text-sm text-gray-200 focus:border-primary-500 focus:outline-none"
                >
                  <option value="draft">Borrador</option>
                  <option value="active">Activo</option>
                  <option value="archived">Archivado</option>
                </select>
                <button
                  onClick={activeProject ? handleUpdate : handleCreate}
                  className="rounded bg-primary-500 px-4 py-2 text-sm text-white hover:bg-primary-600"
                  type="button"
                >
                  {activeProject ? 'Guardar' : 'Crear'}
                </button>
                <button
                  onClick={() => setIsEditing(false)}
                  className="rounded border border-gray-700 px-4 py-2 text-sm text-gray-300 hover:bg-gray-800"
                  type="button"
                >
                  Cancelar
                </button>
              </div>
            </div>
          </div>
        )}

        <div className="flex-1 overflow-y-auto p-4">
          {activeProject ? (
            <div className="space-y-4">
              <div className="rounded border border-gray-800 bg-dark-800 p-4">
                <div className="flex items-center justify-between">
                  <div>
                    <h3 className="text-sm font-semibold text-gray-200">{activeProject.name}</h3>
                    <p className="mt-1 text-xs text-gray-400">{activeProject.description}</p>
                  </div>
                  <span className={`text-xs px-2 py-1 rounded ${
                    activeProject.status === 'active' ? 'bg-green-900 text-green-300' :
                    activeProject.status === 'archived' ? 'bg-gray-700 text-gray-300' :
                    'bg-yellow-900 text-yellow-300'
                  }`}>
                    {activeProject.status === 'active' ? 'Activo' : activeProject.status === 'archived' ? 'Archivado' : 'Borrador'}
                  </span>
                </div>
                <div className="mt-2 text-xs text-gray-500">
                  Actualizado: {new Date(activeProject.updated_at).toLocaleString()}
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="rounded border border-gray-800 bg-dark-800 p-4">
                  <h4 className="text-xs font-semibold text-gray-300">Estado del proyecto</h4>
                  <p className="mt-2 text-xs text-gray-400">
                    {activeProject.status === 'active' ? 'En progreso' : activeProject.status === 'archived' ? 'Archivado' : 'Borrador'}
                  </p>
                </div>
                <div className="rounded border border-gray-800 bg-dark-800 p-4">
                  <h4 className="text-xs font-semibold text-gray-300">Última actualización</h4>
                  <p className="mt-2 text-xs text-gray-400">
                    {new Date(activeProject.updated_at).toLocaleDateString()}
                  </p>
                </div>
              </div>
            </div>
          ) : (
            <div className="flex h-full items-center justify-center">
              <div className="text-center">
                <p className="text-sm text-gray-400">
                  {projects.length === 0
                    ? 'No hay proyectos de investigación. Crea uno para comenzar.'
                    : 'Selecciona un proyecto de la lista o crea uno nuevo.'}
                </p>
                {projects.length > 0 && (
                  <div className="mt-4 space-y-2">
                    {projects.map((project) => (
                      <button
                        key={project.id}
                        onClick={() => handleSelectProject(project.id)}
                        className="block w-full rounded border border-gray-800 bg-dark-800 p-3 text-left hover:border-primary-500"
                        type="button"
                      >
                        <p className="text-sm font-medium text-gray-200">{project.name}</p>
                        <p className="text-xs text-gray-400">{project.description}</p>
                      </button>
                    ))}
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      </div>
    </ResearchLayout>
  )
}
