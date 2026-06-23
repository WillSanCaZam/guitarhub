<script module>
  export const SEARCH_SUGGESTIONS = [
    { label: 'Fender Stratocaster', category: 'guitar' },
    { label: 'Gibson Les Paul', category: 'guitar' },
    { label: 'Klon Centaur', category: 'pedal' },
    { label: 'Marshall JCM800', category: 'amp' },
    { label: 'Fender Telecaster', category: 'guitar' },
    { label: 'Ibanez RG', category: 'guitar' },
    { label: 'Boss DS-1', category: 'pedal' },
    { label: 'Vox AC30', category: 'amp' },
    { label: 'PRS Custom 24', category: 'guitar' },
    { label: 'Strymon BigSky', category: 'pedal' },
  ] as const
</script>

<script lang="ts">
  interface Props {
    placeholder?: string
    onSearch: (query: string) => void
  }

  let { placeholder = 'Search guitars, amps, pedals...', onSearch }: Props = $props()

  const RECENT_KEY = 'guitarhub:recent-searches'
  const MAX_RECENT = 10

  let query = $state('')
  let isOpen = $state(false)
  let activeIndex = $state(-1)
  let recentSearches = $state<string[]>(loadRecents())
  let inputEl: HTMLInputElement | undefined = $state()
  let blurTimeout: ReturnType<typeof setTimeout> | undefined

  function loadRecents(): string[] {
    try {
      return JSON.parse(localStorage.getItem(RECENT_KEY) || '[]')
    } catch {
      return []
    }
  }

  function saveRecent(term: string) {
    const filtered = recentSearches.filter((s) => s !== term)
    filtered.unshift(term)
    recentSearches = filtered.slice(0, MAX_RECENT)
    localStorage.setItem(RECENT_KEY, JSON.stringify(recentSearches))
  }

  let filteredSuggestions = $derived.by(() => {
    const q = query.trim().toLowerCase()
    if (q.length >= 2) {
      return SEARCH_SUGGESTIONS.filter((s) =>
        s.label.toLowerCase().includes(q)
      ).slice(0, 8)
    }
    return []
  })

  let recentMatches = $derived.by(() => {
    if (query.trim().length > 0) {
      return recentSearches
        .filter((r) => r.toLowerCase().includes(query.trim().toLowerCase()))
        .slice(0, 5)
    }
    return recentSearches.slice(0, 5)
  })

  let suggestions = $derived([
    ...recentMatches.map((r) => ({ label: r, category: 'recent' as const })),
    ...filteredSuggestions,
  ])

  let activeDescendant = $derived(
    activeIndex >= 0 ? `searchbar-option-${activeIndex}` : undefined
  )

  function handleFocus() {
    clearTimeout(blurTimeout)
    isOpen = true
    activeIndex = -1
  }

  function handleBlur() {
    blurTimeout = setTimeout(() => {
      isOpen = false
      activeIndex = -1
    }, 150)
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!isOpen && e.key !== 'Enter') return

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault()
        activeIndex = Math.min(activeIndex + 1, suggestions.length - 1)
        break
      case 'ArrowUp':
        e.preventDefault()
        activeIndex = Math.max(activeIndex - 1, -1)
        break
      case 'Enter':
        e.preventDefault()
        if (activeIndex >= 0 && suggestions[activeIndex]) {
          selectSuggestion(suggestions[activeIndex].label)
        } else if (query.trim()) {
          submitSearch(query.trim())
        }
        break
      case 'Escape':
        e.preventDefault()
        clearTimeout(blurTimeout)
        isOpen = false
        activeIndex = -1
        break
    }
  }

  function selectSuggestion(label: string) {
    query = label
    isOpen = false
    activeIndex = -1
    submitSearch(label)
  }

  function submitSearch(term: string) {
    saveRecent(term)
    onSearch(term)
  }

  function getCategoryBadge(category: string): string {
    switch (category) {
      case 'guitar': return '🎸'
      case 'amp': return '🔊'
      case 'pedal': return '🎛️'
      case 'recent': return '🕐'
      default: return ''
    }
  }
</script>

<div class="searchbar-wrapper" role="combobox" aria-expanded={isOpen} aria-haspopup="listbox" aria-controls="searchbar-listbox">
  <div class="searchbar-input-wrap">
    <svg class="search-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
      <circle cx="11" cy="11" r="8"/>
      <path d="m21 21-4.35-4.35"/>
    </svg>
    <input
      bind:this={inputEl}
      bind:value={query}
      type="text"
      class="searchbar-input"
      {placeholder}
      aria-autocomplete="list"
      aria-controls="searchbar-listbox"
      aria-activedescendant={activeDescendant}
      onfocus={handleFocus}
      onblur={handleBlur}
      onkeydown={handleKeydown}
    />
    {#if query}
      <button
        class="clear-btn"
        onclick={() => { query = ''; activeIndex = -1; inputEl?.focus() }}
        aria-label="Clear search"
        type="button"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 6L6 18M6 6l12 12"/>
        </svg>
      </button>
    {/if}
  </div>

  {#if isOpen && suggestions.length > 0}
    <ul id="searchbar-listbox" class="searchbar-dropdown" role="listbox">
      {#if recentMatches.length > 0 && query.trim().length === 0}
        <li class="dropdown-header" role="presentation">Recent searches</li>
      {/if}
      {#each suggestions as suggestion, i (suggestion.label)}
        <li
          id="searchbar-option-{i}"
          class="dropdown-item"
          class:active={i === activeIndex}
          role="option"
          aria-selected={i === activeIndex}
          onmousedown={(e) => { e.preventDefault(); selectSuggestion(suggestion.label) }}
        >
          <span class="item-badge">{getCategoryBadge(suggestion.category)}</span>
          <span class="item-label">{suggestion.label}</span>
          {#if suggestion.category !== 'recent'}
            <span class="item-category">{suggestion.category}</span>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .searchbar-wrapper {
    position: relative;
    width: 100%;
  }

  .searchbar-input-wrap {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: var(--void-raised);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    transition: border-color 150ms var(--ease-snap), box-shadow 150ms var(--ease-snap);
  }

  .searchbar-input-wrap:focus-within {
    border-color: var(--border-active);
    box-shadow: 0 0 0 1px var(--border-active), 0 0 24px var(--glow-soft);
  }

  .search-icon {
    color: var(--text-dim);
    flex-shrink: 0;
  }

  .searchbar-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-bright);
    font-family: var(--font-body);
    font-size: 0.9rem;
    min-width: 0;
  }

  .searchbar-input::placeholder {
    color: var(--text-dim);
  }

  .clear-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-dim);
    padding: var(--space-1);
    border-radius: var(--radius-sm);
    transition: color 150ms var(--ease-snap), background 150ms var(--ease-snap);
  }

  .clear-btn:hover {
    color: var(--text-bright);
    background: var(--void-hover);
  }

  .searchbar-dropdown {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    background: var(--bg-elevated, var(--void-raised));
    border: 1px solid var(--border-default, var(--border-subtle));
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-card);
    z-index: var(--z-dropdown);
    max-height: 320px;
    overflow-y: auto;
    list-style: none;
    margin: 0;
    padding: var(--space-1) 0;
  }

  .dropdown-header {
    padding: var(--space-2) var(--space-3);
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-dim);
  }

  .dropdown-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    cursor: pointer;
    transition: background 100ms var(--ease-snap);
    font-size: 0.85rem;
    color: var(--text-warm);
  }

  .dropdown-item:hover,
  .dropdown-item.active {
    background: var(--void-hover);
    color: var(--text-bright);
  }

  .item-badge {
    flex-shrink: 0;
    font-size: 0.8rem;
  }

  .item-label {
    flex: 1;
  }

  .item-category {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-dim);
    font-weight: 500;
  }

  @media (prefers-reduced-motion: reduce) {
    .searchbar-input-wrap {
      transition: none;
    }
  }
</style>
