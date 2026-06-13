<script lang="ts">
  interface Props {
    src?: string;
    alt?: string;
    name?: string;
    size?: 'sm' | 'md' | 'lg';
  }

  let { src, alt = '', name = '', size = 'md' }: Props = $props();

  let initials = $derived(() => {
    if (!name) return '?';
    const parts = name.trim().split(/\s+/);
    if (parts.length >= 2) {
      return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
    }
    return parts[0].substring(0, 2).toUpperCase();
  });

  let imageError = $state(false);
</script>

<div class="avatar avatar--{size}" aria-label={alt || name || 'Avatar'}>
  {#if src && !imageError}
    <!-- svelte-ignore a11y_no_redundant_roles -->
    <img
      {src}
      alt={alt || name}
      class="avatar-image"
      onerror={() => { imageError = true; }}
    />
  {:else}
    <span class="avatar-initials">{initials()}</span>
  {/if}
</div>

<style>
  .avatar {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-full, 9999px);
    background: var(--color-secondary-container, #1e3a5f);
    color: var(--color-on-secondary-container, #c8ddf5);
    overflow: hidden;
    flex-shrink: 0;
  }

  .avatar--sm {
    width: 32px;
    height: 32px;
    font-size: 0.75rem;
  }

  .avatar--md {
    width: 40px;
    height: 40px;
    font-size: 0.875rem;
  }

  .avatar--lg {
    width: 56px;
    height: 56px;
    font-size: 1.25rem;
  }

  .avatar-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .avatar-initials {
    font-family: var(--font-sans, system-ui, sans-serif);
    font-weight: 600;
    user-select: none;
  }
</style>
