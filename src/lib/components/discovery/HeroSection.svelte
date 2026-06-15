<script lang="ts">
  interface Props {
    onSearch?: (query: string) => void;
  }

  let { onSearch }: Props = $props();

  let searchQuery = $state('');
  let currentPlaceholder = $state(0);

  const placeholders = [
    'Stratocaster...',
    'Tube Screamer...',
    'Marshall stack...',
    'Klon Centaur...',
    'Strymon BigSky...',
  ];

  const categories = [
    { label: 'Guitars', icon: '🎸' },
    { label: 'Amps', icon: '🔊' },
    { label: 'Pedals', icon: '🎛️' },
    { label: 'Pickups', icon: '🎵' },
    { label: 'Drums', icon: '🥁' },
    { label: 'Keys', icon: '🎹' },
    { label: 'Studio', icon: '🎤' },
    { label: 'Accessories', icon: '🔧' },
  ];

  const trending = [
    'John Mayer Strat',
    'Polyphia Tone',
    'Blues Jr',
    'Klon Centaur',
    'Neural DSP',
    'Strymon BigSky',
  ];

  function handleSearch() {
    if (searchQuery.trim().length >= 3) {
      onSearch?.(searchQuery.trim());
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') handleSearch();
  }

  $effect(() => {
    const interval = setInterval(() => {
      currentPlaceholder = (currentPlaceholder + 1) % placeholders.length;
    }, 3000);
    return () => clearInterval(interval);
  });
</script>

<section class="hero-section" role="banner">
  <!-- Background layers -->
  <div class="hero-bg">
    <div class="gradient-mesh"></div>
    <div class="noise-texture"></div>
  </div>

  <div class="hero-content">
    <h1 class="hero-title">
      <span class="title-line">FIND YOUR</span>
      <span class="title-line accent">PERFECT TONE</span>
    </h1>
    <p class="hero-subtitle">
      Search 50,000+ guitars, amps, pedals & gear from the world's best stores. All in one place.
    </p>

    <!-- Search Bar -->
    <div class="search-bar" role="search" aria-label="Search gear">
      <div class="search-input-wrap">
        <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"/>
          <path d="m21 21-4.35-4.35"/>
        </svg>
        <input
          type="text"
          bind:value={searchQuery}
          onkeydown={handleKeydown}
          placeholder={placeholders[currentPlaceholder]}
          class="search-input"
          aria-label="Search guitars, amps, pedals"
        />
        <button class="search-btn" onclick={handleSearch}>Search</button>
      </div>
    </div>

    <!-- Category Pills -->
    <div class="category-pills">
      {#each categories as cat}
        <button class="category-pill">
          <span class="cat-icon">{cat.icon}</span>
          <span class="cat-label">{cat.label}</span>
        </button>
      {/each}
    </div>

    <!-- Trending -->
    <div class="trending-row">
      <span class="trending-label">🔥 Trending:</span>
      {#each trending as query}
        <button class="trending-chip" onclick={() => { searchQuery = query; handleSearch(); }}>
          {query}
        </button>
      {/each}
    </div>
  </div>
</section>

<style>
  .hero-section {
    position: relative;
    min-height: 70vh;
    max-height: 800px;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    margin-bottom: var(--space-8);
  }

  /* Background */
  .hero-bg {
    position: absolute;
    inset: 0;
    z-index: 0;
  }

  .gradient-mesh {
    position: absolute;
    inset: 0;
    background:
      radial-gradient(ellipse at 20% 50%, rgba(255, 122, 61, 0.08) 0%, transparent 50%),
      radial-gradient(ellipse at 80% 50%, rgba(77, 225, 255, 0.05) 0%, transparent 50%),
      radial-gradient(ellipse at 50% 0%, var(--glow-soft) 0%, transparent 60%);
    animation: gradientShift 20s ease infinite;
    background-size: 200% 200%;
  }

  @keyframes gradientShift {
    0% { background-position: 0% 50%; }
    50% { background-position: 100% 50%; }
    100% { background-position: 0% 50%; }
  }

  .noise-texture {
    position: absolute;
    inset: 0;
    opacity: 0.02;
    background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.65' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E");
  }

  .hero-content {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: var(--space-8);
    max-width: 900px;
    width: 100%;
  }

  /* Title */
  .hero-title {
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    margin-bottom: var(--space-4);
  }

  .title-line {
    font-family: var(--font-display);
    font-size: clamp(2rem, 5vw, 4rem);
    font-weight: 700;
    color: var(--text-bright);
    letter-spacing: -0.04em;
    line-height: 0.95;
  }

  .title-line.accent {
    color: var(--glow-primary);
    text-shadow: var(--glow-text-shadow);
  }

  .hero-subtitle {
    margin: 0 0 var(--space-6);
    font-size: 1.1rem;
    color: var(--text-warm);
    max-width: 600px;
    line-height: 1.6;
  }

  /* Search */
  .search-bar {
    width: 100%;
    max-width: 700px;
    margin-bottom: var(--space-6);
  }

  .search-input-wrap {
    display: flex;
    align-items: center;
    background: rgba(15, 15, 24, 0.9);
    backdrop-filter: blur(24px);
    border: 1px solid rgba(255, 122, 61, 0.1);
    border-radius: var(--radius-lg);
    padding: var(--space-1);
    height: 64px;
    transition: border-color 250ms var(--ease-plug), box-shadow 250ms var(--ease-plug);
  }

  .search-input-wrap:focus-within {
    border-color: var(--glow-primary);
    box-shadow: 0 0 30px var(--glow-soft);
  }

  .search-icon {
    width: 24px;
    height: 24px;
    margin: 0 var(--space-3);
    color: var(--text-dim);
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    border: none;
    background: transparent;
    font-size: 1rem;
    color: var(--text-bright);
    outline: none;
    font-family: var(--font-body);
  }

  .search-input::placeholder {
    color: var(--text-dim);
  }

  .search-btn {
    padding: var(--space-2) var(--space-6);
    background: var(--glow-primary);
    color: var(--void-deep);
    border: none;
    border-radius: var(--radius-md);
    font-weight: 700;
    font-size: 0.9rem;
    cursor: pointer;
    transition: background 150ms var(--ease-snap);
    height: 48px;
  }

  .search-btn:hover {
    background: var(--glow-warm);
  }

  /* Categories */
  .category-pills {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    justify-content: center;
    margin-bottom: var(--space-6);
  }

  .category-pill {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-4);
    height: 44px;
    border-radius: var(--radius-pill);
    background: var(--void-mid);
    color: var(--text-warm);
    border: 1px solid rgba(255, 255, 255, 0.06);
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 500;
    transition: background 150ms var(--ease-snap), border-color 150ms var(--ease-snap);
  }

  .category-pill:hover {
    background: var(--void-hover);
    border-color: rgba(255, 122, 61, 0.2);
  }

  .cat-icon {
    font-size: 1rem;
  }

  /* Trending */
  .trending-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-wrap: wrap;
    justify-content: center;
  }

  .trending-label {
    font-size: 0.8rem;
    color: var(--text-dim);
    font-weight: 600;
    white-space: nowrap;
  }

  .trending-chip {
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-pill);
    background: var(--void-raised);
    color: var(--text-warm);
    font-size: 0.8rem;
    border: 1px solid rgba(255, 255, 255, 0.04);
    cursor: pointer;
    transition: background 150ms var(--ease-snap);
  }

  .trending-chip:hover {
    background: var(--void-hover);
    color: var(--text-bright);
  }
</style>
