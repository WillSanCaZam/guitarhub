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
      <a href={exploreHref} class="explore-btn">Explore Rig →</a>
    </div>
  </div>
</div>

<style>
  .featured-rig {
    border-radius: var(--radius-xl);
    background: var(--void-mid);
    background-image: var(--gradient-hero);
    border: 1px solid var(--glow-soft);
    overflow: hidden;
    min-height: 280px;
    display: flex;
    align-items: center;
    margin-bottom: var(--space-8);
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
    width: 160px;
    height: 160px;
    border-radius: 50%;
    overflow: hidden;
    border: 3px solid var(--glow-soft);
    flex-shrink: 0;
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
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--glow-primary);
  }

  .artist-name {
    margin: 0;
    font-family: var(--font-display);
    font-size: 2rem;
    font-weight: 700;
    color: var(--text-bright);
  }

  .artist-band {
    margin: 0;
    font-size: 0.95rem;
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
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text-bright);
  }

  .rig-quote {
    margin: 0;
    font-family: var(--font-accent);
    font-size: 1.1rem;
    color: var(--text-warm);
    line-height: 1.5;
    font-style: italic;
  }

  .explore-btn {
    display: inline-flex;
    align-items: center;
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
      width: 120px;
      height: 120px;
    }
  }
</style>
