import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte'
import SearchBar from '../ui/SearchBar.svelte'

const localStorageMock = (() => {
  let store: Record<string, string> = {}
  return {
    getItem: (key: string) => store[key] ?? null,
    setItem: (key: string, value: string) => { store[key] = value },
    removeItem: (key: string) => { delete store[key] },
    clear: () => { store = {} },
  }
})()

vi.stubGlobal('localStorage', localStorageMock)

describe('SearchBar', () => {
  beforeEach(() => {
    localStorage.clear()
  })

  it('renders search input', () => {
    const onSearch = vi.fn()
    render(SearchBar, { props: { onSearch } })
    expect(screen.getByRole('textbox')).toBeInTheDocument()
  })

  it('has combobox role on wrapper', () => {
    const onSearch = vi.fn()
    const { container } = render(SearchBar, { props: { onSearch } })
    const wrapper = container.querySelector('[role="combobox"]')
    expect(wrapper).toBeInTheDocument()
  })

  it('opens dropdown on focus showing recent searches', async () => {
    localStorage.setItem('guitarhub:recent-searches', JSON.stringify(['Stratocaster', 'Les Paul']))
    const onSearch = vi.fn()
    render(SearchBar, { props: { onSearch } })
    const input = screen.getByRole('textbox')
    await fireEvent.focus(input)
    await fireEvent.keyDown(input, { key: 'ArrowDown' })
    expect(screen.getByText('Stratocaster')).toBeInTheDocument()
    expect(screen.getByText('Les Paul')).toBeInTheDocument()
  })

  it('shows recent searches header when recent exist', async () => {
    localStorage.setItem('guitarhub:recent-searches', JSON.stringify(['Strat']))
    const onSearch = vi.fn()
    render(SearchBar, { props: { onSearch } })
    const input = screen.getByRole('textbox')
    await fireEvent.focus(input)
    await fireEvent.keyDown(input, { key: 'ArrowDown' })
    expect(screen.getByText('Recent searches')).toBeInTheDocument()
  })

  it('filters suggestions when typing 2+ chars', async () => {
    const onSearch = vi.fn()
    render(SearchBar, { props: { onSearch } })
    const input = screen.getByRole('textbox')
    await fireEvent.focus(input)
    await fireEvent.keyDown(input, { key: 'ArrowDown' })
    const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value')!.set!
    nativeInputValueSetter.call(input, 'Fen')
    input.dispatchEvent(new Event('input', { bubbles: true }))
    await waitFor(() => {
      expect(screen.getByText('Fender Stratocaster')).toBeInTheDocument()
    })
    expect(screen.getByText('Fender Telecaster')).toBeInTheDocument()
  })

  it('does not show suggestions with less than 2 chars', async () => {
    const onSearch = vi.fn()
    render(SearchBar, { props: { onSearch } })
    const input = screen.getByRole('textbox')
    await fireEvent.focus(input)
    await fireEvent.keyDown(input, { key: 'ArrowDown' })
    const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value')!.set!
    nativeInputValueSetter.call(input, 'F')
    input.dispatchEvent(new Event('input', { bubbles: true }))
    await waitFor(() => {
      expect(screen.queryByText('Fender Stratocaster')).not.toBeInTheDocument()
    })
  })

  it('closes dropdown on Escape', async () => {
    localStorage.setItem('guitarhub:recent-searches', JSON.stringify(['test']))
    const onSearch = vi.fn()
    render(SearchBar, { props: { onSearch } })
    const input = screen.getByRole('textbox')
    await fireEvent.focus(input)
    await fireEvent.keyDown(input, { key: 'ArrowDown' })
    expect(screen.getByText('test')).toBeInTheDocument()
    await fireEvent.keyDown(input, { key: 'Escape' })
    await waitFor(() => {
      expect(screen.queryByText('test')).not.toBeInTheDocument()
    })
  })

  it('navigates suggestions with ArrowDown', async () => {
    const onSearch = vi.fn()
    render(SearchBar, { props: { onSearch } })
    const input = screen.getByRole('textbox')
    await fireEvent.focus(input)
    await fireEvent.keyDown(input, { key: 'ArrowDown' })
    const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value')!.set!
    nativeInputValueSetter.call(input, 'Strat')
    input.dispatchEvent(new Event('input', { bubbles: true }))
    await waitFor(() => {
      const options = screen.getAllByRole('option')
      expect(options.length).toBeGreaterThan(0)
    })
    await fireEvent.keyDown(input, { key: 'ArrowDown' })
    await waitFor(() => {
      const options = screen.getAllByRole('option')
      expect(options[0]).toHaveAttribute('aria-selected', 'true')
    })
  })

  it('navigates suggestions with ArrowUp', async () => {
    const onSearch = vi.fn()
    render(SearchBar, { props: { onSearch } })
    const input = screen.getByRole('textbox')
    await fireEvent.focus(input)
    await fireEvent.keyDown(input, { key: 'ArrowDown' })
    const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value')!.set!
    nativeInputValueSetter.call(input, 'Fen')
    input.dispatchEvent(new Event('input', { bubbles: true }))
    await waitFor(() => {
      expect(screen.getAllByRole('option').length).toBeGreaterThan(0)
    })
    // ArrowDown to first (0), ArrowDown to second (1), ArrowUp back to first (0)
    await fireEvent.keyDown(input, { key: 'ArrowDown' })
    await fireEvent.keyDown(input, { key: 'ArrowDown' })
    await fireEvent.keyDown(input, { key: 'ArrowUp' })
    await waitFor(() => {
      const options = screen.getAllByRole('option')
      expect(options[0]).toHaveAttribute('aria-selected', 'true')
    })
  })

  it('selects highlighted suggestion on Enter', async () => {
    const onSearch = vi.fn()
    render(SearchBar, { props: { onSearch } })
    const input = screen.getByRole('textbox')
    await fireEvent.focus(input)
    await fireEvent.keyDown(input, { key: 'ArrowDown' })
    const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value')!.set!
    nativeInputValueSetter.call(input, 'Fen')
    input.dispatchEvent(new Event('input', { bubbles: true }))
    await waitFor(() => {
      expect(screen.getAllByRole('option').length).toBeGreaterThan(0)
    })
    await fireEvent.keyDown(input, { key: 'ArrowDown' })
    await fireEvent.keyDown(input, { key: 'Enter' })
    expect(onSearch).toHaveBeenCalledWith('Fender Stratocaster')
  })

  it('submits current query on Enter when no suggestion highlighted', async () => {
    const onSearch = vi.fn()
    render(SearchBar, { props: { onSearch } })
    const input = screen.getByRole('textbox')
    const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value')!.set!
    nativeInputValueSetter.call(input, 'Klon Centaur')
    input.dispatchEvent(new Event('input', { bubbles: true }))
    await fireEvent.keyDown(input, { key: 'Enter' })
    expect(onSearch).toHaveBeenCalledWith('Klon Centaur')
  })

  it('saves search to recent searches', async () => {
    const onSearch = vi.fn()
    render(SearchBar, { props: { onSearch } })
    const input = screen.getByRole('textbox')
    const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value')!.set!
    nativeInputValueSetter.call(input, 'test query')
    input.dispatchEvent(new Event('input', { bubbles: true }))
    await fireEvent.keyDown(input, { key: 'Enter' })
    const stored = JSON.parse(localStorage.getItem('guitarhub:recent-searches') || '[]')
    expect(stored).toContain('test query')
  })

  it('caps recent searches at 10 (FIFO)', async () => {
    const existing = Array.from({ length: 10 }, (_, i) => `item-${i}`)
    localStorage.setItem('guitarhub:recent-searches', JSON.stringify(existing))
    const onSearch = vi.fn()
    render(SearchBar, { props: { onSearch } })
    const input = screen.getByRole('textbox')
    const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value')!.set!
    nativeInputValueSetter.call(input, 'new-item')
    input.dispatchEvent(new Event('input', { bubbles: true }))
    await fireEvent.keyDown(input, { key: 'Enter' })
    const stored = JSON.parse(localStorage.getItem('guitarhub:recent-searches') || '[]')
    expect(stored).toHaveLength(10)
    expect(stored[0]).toBe('new-item')
    // FIFO: oldest (item-9) should be evicted
    expect(stored).not.toContain('item-9')
  })

  it('deduplicates recent searches', async () => {
    localStorage.setItem('guitarhub:recent-searches', JSON.stringify(['existing', 'other']))
    const onSearch = vi.fn()
    render(SearchBar, { props: { onSearch } })
    const input = screen.getByRole('textbox')
    const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value')!.set!
    nativeInputValueSetter.call(input, 'existing')
    input.dispatchEvent(new Event('input', { bubbles: true }))
    await fireEvent.keyDown(input, { key: 'Enter' })
    const stored = JSON.parse(localStorage.getItem('guitarhub:recent-searches') || '[]')
    expect(stored.filter((s: string) => s === 'existing')).toHaveLength(1)
  })

  it('has aria-expanded on wrapper', async () => {
    const onSearch = vi.fn()
    const { container } = render(SearchBar, { props: { onSearch } })
    const wrapper = container.querySelector('[role="combobox"]')
    expect(wrapper).toHaveAttribute('aria-expanded', 'false')
  })

  it('sets aria-expanded to true when dropdown is open', async () => {
    localStorage.setItem('guitarhub:recent-searches', JSON.stringify(['test']))
    const onSearch = vi.fn()
    const { container } = render(SearchBar, { props: { onSearch } })
    const input = screen.getByRole('textbox')
    await fireEvent.focus(input)
    await fireEvent.keyDown(input, { key: 'ArrowDown' })
    const wrapper = container.querySelector('[role="combobox"]')
    expect(wrapper).toHaveAttribute('aria-expanded', 'true')
  })

  it('exports SEARCH_SUGGESTIONS', async () => {
    const mod = await import('../ui/SearchBar.svelte')
    expect(mod.SEARCH_SUGGESTIONS).toBeDefined()
    expect(mod.SEARCH_SUGGESTIONS.length).toBeGreaterThan(0)
    expect(mod.SEARCH_SUGGESTIONS[0]).toHaveProperty('label')
    expect(mod.SEARCH_SUGGESTIONS[0]).toHaveProperty('category')
  })

  it('searches across all suggestion categories', async () => {
    const onSearch = vi.fn()
    render(SearchBar, { props: { onSearch } })
    const input = screen.getByRole('textbox')
    await fireEvent.focus(input)
    await fireEvent.keyDown(input, { key: 'ArrowDown' })
    const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value')!.set!
    nativeInputValueSetter.call(input, 'Marshall')
    input.dispatchEvent(new Event('input', { bubbles: true }))
    await waitFor(() => {
      expect(screen.getByText('Marshall JCM800')).toBeInTheDocument()
    })
  })
})
