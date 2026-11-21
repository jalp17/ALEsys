import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import TaskMonitor from '../components/TaskMonitor.vue'

describe('TaskMonitor', () => {
  it('renders without errors', () => {
    const wrapper = mount(TaskMonitor)
    expect(wrapper.exists()).toBe(true)
  })

  it('shows Tareas label', () => {
    const wrapper = mount(TaskMonitor)
    expect(wrapper.text()).toContain('Tareas')
  })
})
