import { describe, it, expect } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import PanelManager from '../layouts/ResearchLayout/PanelManager'

const renderWithRouter = (ui: React.ReactElement) =>
  render(<BrowserRouter>{ui}</BrowserRouter>)

describe('PanelManager', () => {
  it('renders panel toggles', () => {
    renderWithRouter(<PanelManager />)
    expect(screen.getByText('Literatura')).toBeInTheDocument()
    expect(screen.getByText('Citas')).toBeInTheDocument()
    expect(screen.getByText('Notas')).toBeInTheDocument()
    expect(screen.getByText('Síntesis')).toBeInTheDocument()
  })

  it('toggles panel visibility', () => {
    renderWithRouter(<PanelManager />)
    const literatureButton = screen.getByText('Literatura')
    fireEvent.click(literatureButton)
    expect(screen.queryByText('Literature panel (placeholder)')).not.toBeInTheDocument()
  })

  it('enters fullscreen mode', () => {
    renderWithRouter(<PanelManager />)
    const fullscreenButtons = screen.getAllByText('⛶')
    fireEvent.click(fullscreenButtons[0])
    expect(screen.getByText('Salir fullscreen')).toBeInTheDocument()
  })
})
