import { Outlet } from 'react-router-dom'
import type { ReactNode } from 'react'

type Props = {
  children?: ReactNode
}

export default function ResearchLayout({ children }: Props) {
  return (
    <div className="flex h-screen flex-col bg-dark-900 text-gray-100">
      <header className="flex items-center justify-between border-b border-gray-700 px-4 py-2">
        <h1 className="text-lg font-semibold text-primary-400">ResearchLayout</h1>
        <nav className="flex items-center gap-2 text-sm">
          <span className="text-gray-400">Proyecto: <span className="text-gray-200">Sin seleccionar</span></span>
        </nav>
      </header>
      <main className="flex-1 overflow-hidden">
        {children ?? <Outlet />}
      </main>
    </div>
  )
}
