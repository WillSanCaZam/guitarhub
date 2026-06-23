<script lang="ts">
  interface Props {
    artist: string;
    band?: string;
    rigName: string;
    artistPhoto: string;
    quote: string;
    exploreHref?: string;
  }

  let { artist, band, rigName, artistPhoto, quote, exploreHref = '#' }: Props = $props();
</script>

<div class="featured-rig">
  <div class="rig-eyebrow" aria-hidden="true"></div>
  <div class="rig-content">
    <div class="artist-info">
      <div class="artist-photo-wrap">
        <img src={artistPhoto} alt={artist} class="artist-photo" />
      </div>
      <div class="artist-text">
        <h3 class="rig-label">Featured Rig of the Week</h3>
        <h2 class="artist-name">{artist}</h2>
        {#if band}
          <p class="artist-band">{band}</p>
        {/if}
      </div>
    </div>
    <div class="rig-details">
      <p class="rig-name">{rigName}</p>
      <p class="rig-quote">"{quote}"</p>
      <div class="gear-preview">
        <div class="gear-mini-card">
          <div class="gear-mini-image skeleton"></div>
          <div class="gear-mini-text skeleton"></div>
        </div>
        <div class="gear-mini-card">
          <div class="gear-mini-image skeleton"></div>
          <div class="gear-mini-text skeleton"></div>
        </div>
        <div class="gear-mini-card">
          <div class="gear-mini-image skeleton"></div>
          <div class="gear-mini-text skeleton"></div>
        </div>
      </div>
      <a href={exploreHref} class="explore-btn">
        Explore Rig
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <path d="M5 12h14M12 5l7 7-7 7"/>
        </svg>
      </a>
    </div>
  </div>
</div>

<style>
  .featured-rig {
    border-radius: var(--radius-xl);
    background: var(--void-mid);
    background-image: var(--gradient-hero);
    border: 1px solid var(--glow-featured);
    overflow: hidden;
    min-height: 280px;
    display: flex;
    margin-bottom: var(--space-8);
    position: relative;
    transition: box-shadow 200ms var(--ease-snap);
  }

  .featured-rig:hover {
    box-shadow: 0 0 40px var(--glow-soft);
  }

  .rig-eyebrow {
    width: 3px;
    background: var(--orange-500, var(--glow-primary));
    flex-shrink: 0;
  }

  .rig-content {
    display: flex;
    align-items: center;
    gap: var(--space-8);
    padding: var(--space-8);
    width: 100%;
  }

  .artist-info {
    display: flex;
    align-items: center;
    gap: var(--space-6);
    flex-shrink: 0;
  }

  .artist-photo-wrap {
    width: 140px;
    height: 140px;
    border-radius: 50%;
    overflow: hidden;
    border: 3px solid var(--border-active);
    flex-shrink: 0;
    box-shadow: 0 0 20px var(--glow-soft), 0 0 40px var(--glow-soft);
  }

  .artist-photo {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .artist-text {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .rig-label {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 0.65rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.15em;
    color: var(--orange-500, var(--glow-primary));
  }

  .artist-name {
    margin: 0;
    font-family: var(--font-display);
    font-size: 1.75rem;
    font-weight: 700;
    color: var(--text-bright);
  }

  .artist-band {
    margin: 0;
    font-size: 0.9rem;
    color: var(--text-warm);
  }

  .rig-details {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .rig-name {
    margin: 0;
    font-family: var(--font-display);
    font-size: 1.15rem;
    font-weight: 600;
    color: var(--text-bright);
  }

  .rig-quote {
    margin: 0;
    font-family: var(--font-accent);
    font-size: 1.05rem;
    color: var(--text-secondary, var(--text-warm));
    line-height: 1.5;
    font-style: italic;
  }

  .gear-preview {
    display: flex;
    gap: var(--space-3);
    margin-top: var(--space-2);
  }

  .gear-mini-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding: var(--space-2);
    background: var(--void-raised);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
    flex: 1;
    min-width: 0;
  }

  .gear-mini-image {
    width: 100%;
    aspect-ratio: 16 / 10;
    border-radius: var(--radius-xs);
  }

  .gear-mini-text {
    height: 12px;
    width: 70%;
    border-radius: var(--radius-xs);
  }

  .skeleton {
    background: linear-gradient(90deg, var(--shimmer-base) 25%, var(--shimmer-highlight) 50%, var(--shimmer-base) 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
  }

  .explore-btn {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-4);
    background: var(--glow-primary);
    color: var(--void-deep);
    border-radius: var(--radius-pill);
    font-weight: 700;
    font-size: 0.85rem;
    text-decoration: none;
    align-self: flex-start;
    transition: background 150ms var(--ease-snap), transform 150ms var(--ease-snap);
  }

  .explore-btn:hover {
    background: var(--glow-warm);
    transform: translateY(-1px);
  }

  @media (max-width: 768px) {
    .rig-content {
      flex-direction: column;
      align-items: flex-start;
      padding: var(--space-6);
    }

    .artist-info {
      flex-direction: column;
      align-items: center;
      text-align: center;
    }

    .artist-text {
      align-items: center;
    }

    .artist-photo-wrap {
      width: 110px;
      height: 110px;
    }

    .gear-preview {
      flex-direction: column;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .featured-rig {
      transition: none;
    }
  }
</style>
