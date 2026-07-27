import { describe, it, expect } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import PanelManager from '../layouts/ResearchLayout/PanelManager'

const renderWithRouter = (ui: React.ReactElement) =>
  render(<BrowserRouter>{ui}</BrowserRouter>)

describe('PanelManager', () => {
  it('renders panel toggles', () => {
    renderWithRouter(<PanelManager />)
    expect(screen.getByRole('button', { name: 'Literatura' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Citas' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Notas' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Síntesis' })).toBeInTheDocument()
  })

  it('toggles panel visibility', () => {
    renderWithRouter(<PanelManager />)
    const literatureButton = screen.getByRole('button', { name: 'Literatura' })
    fireEvent.click(literatureButton)
    expect(screen.queryByText('Literature panel (placeholder)')).not.toBeInTheDocument()
  })

  it('enters fullscreen mode', () => {
    renderWithRouter(<PanelManager />)
    const fullscreenButtons = screen.getAllByText('⛶')
    fireEvent.click(fullscreenButtons[0])
    expect(screen.getByText((content) => content.includes('Salir fullscreen'))).toBeInTheDocument()
  })
})
