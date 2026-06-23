<script lang="ts">
  interface Props {
    variant: 'collection' | 'wishlist' | 'search' | 'feed' | 'lessons' | 'riffs' | 'default'
    title: string
    description?: string
    actionLabel?: string
    onAction?: () => void
  }

  let { variant, title, description, actionLabel, onAction }: Props = $props()
</script>

<div class="empty-state">
  <div class="empty-illustration" aria-hidden="true">
    {#if variant === 'collection'}
      <svg width="80" height="80" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg">
        <rect x="12" y="20" width="56" height="44" rx="4" stroke="var(--empty-state-illustration)" stroke-width="2"/>
        <path d="M28 42L36 50L52 34" stroke="var(--glow-primary)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        <circle cx="40" cy="16" r="6" stroke="var(--empty-state-illustration)" stroke-width="2"/>
      </svg>
    {:else if variant === 'wishlist'}
      <svg width="80" height="80" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M40 68L14 42C8 36 8 26 14 20C20 14 30 14 36 20L40 24L44 20C50 14 60 14 66 20C72 26 72 36 66 42L40 68Z"
              stroke="var(--empty-state-illustration)" stroke-width="2" fill="none"/>
      </svg>
    {:else if variant === 'search'}
      <svg width="80" height="80" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg">
        <circle cx="36" cy="36" r="20" stroke="var(--empty-state-illustration)" stroke-width="2"/>
        <line x1="50" y1="50" x2="66" y2="66" stroke="var(--empty-state-illustration)" stroke-width="2" stroke-linecap="round"/>
        <line x1="28" y1="36" x2="44" y2="36" stroke="var(--glow-primary)" stroke-width="2" stroke-linecap="round"/>
      </svg>
    {:else if variant === 'feed'}
      <svg width="80" height="80" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg">
        <rect x="16" y="12" width="48" height="56" rx="4" stroke="var(--empty-state-illustration)" stroke-width="2"/>
        <line x1="24" y1="24" x2="56" y2="24" stroke="var(--glow-primary)" stroke-width="2" stroke-linecap="round"/>
        <line x1="24" y1="34" x2="48" y2="34" stroke="var(--empty-state-illustration)" stroke-width="2" stroke-linecap="round"/>
        <line x1="24" y1="44" x2="52" y2="44" stroke="var(--empty-state-illustration)" stroke-width="2" stroke-linecap="round"/>
        <line x1="24" y1="54" x2="40" y2="54" stroke="var(--empty-state-illustration)" stroke-width="2" stroke-linecap="round"/>
      </svg>
    {:else if variant === 'lessons'}
      <svg width="80" height="80" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg">
        <rect x="12" y="16" width="56" height="48" rx="4" stroke="var(--empty-state-illustration)" stroke-width="2"/>
        <path d="M32 36V28L44 36L32 44V36Z" fill="var(--glow-primary)"/>
        <line x1="24" y1="54" x2="56" y2="54" stroke="var(--empty-state-illustration)" stroke-width="2" stroke-linecap="round"/>
      </svg>
    {:else if variant === 'riffs'}
      <svg width="80" height="80" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M20 60V28C20 24 24 20 28 20H52C56 20 60 24 60 28V52" stroke="var(--empty-state-illustration)" stroke-width="2" stroke-linecap="round"/>
        <circle cx="30" cy="40" r="4" stroke="var(--glow-primary)" stroke-width="2"/>
        <circle cx="44" cy="34" r="4" stroke="var(--glow-primary)" stroke-width="2"/>
        <circle cx="54" cy="44" r="4" stroke="var(--glow-primary)" stroke-width="2"/>
        <line x1="34" y1="40" x2="40" y2="34" stroke="var(--empty-state-illustration)" stroke-width="1.5"/>
        <line x1="48" y1="34" x2="50" y2="44" stroke="var(--empty-state-illustration)" stroke-width="1.5"/>
      </svg>
    {:else}
      <svg width="80" height="80" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg">
        <circle cx="40" cy="40" r="24" stroke="var(--empty-state-illustration)" stroke-width="2"/>
        <path d="M32 40H48" stroke="var(--glow-primary)" stroke-width="2" stroke-linecap="round"/>
        <path d="M40 32V48" stroke="var(--glow-primary)" stroke-width="2" stroke-linecap="round"/>
      </svg>
    {/if}
  </div>

  <h2 class="empty-title">{title}</h2>
  {#if description}
    <p class="empty-description">{description}</p>
  {/if}
  {#if actionLabel && onAction}
    <button class="empty-action" onclick={onAction}>{actionLabel}</button>
  {/if}
</div>

<style>
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-3xl) var(--space-lg);
    text-align: center;
  }

  .empty-illustration {
    opacity: 0.7;
  }

  .empty-title {
    margin: 0;
    font-family: var(--font-display);
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--empty-state-title);
  }

  .empty-description {
    margin: 0;
    color: var(--empty-state-description);
    font-size: 0.9rem;
    max-width: 320px;
  }

  .empty-action {
    margin-top: var(--space-2);
    padding: var(--space-2) var(--space-6);
    background: var(--glow-primary);
    color: var(--void-deep);
    border: none;
    border-radius: var(--radius-pill);
    font-weight: 600;
    font-size: 0.9rem;
    cursor: pointer;
    transition: transform var(--transition-fast), box-shadow var(--transition-fast);
  }

  .empty-action:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px var(--shadow-subtle);
  }

  /* Reduced motion */
  @media (prefers-reduced-motion: reduce) {
    .empty-state {
      animation: none;
    }
  }
</style>
