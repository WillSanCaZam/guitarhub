<script lang="ts">
  let { storeId, size = 32 }: { storeId: string; size?: number } = $props()
</script>

{#if storeId === 'reverb'}
  <!-- Reverb "R" logo -->
  <svg
    width={size}
    height={size}
    viewBox="0 0 32 32"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    role="img"
    aria-label="Reverb"
  >
    <rect width="32" height="32" rx="6" fill="#E94E1B" />
    <text
      x="16"
      y="22"
      text-anchor="middle"
      fill="white"
      font-family="system-ui, sans-serif"
      font-size="18"
      font-weight="700"
    >R</text>
  </svg>
{:else}
  <!-- Generic store icon -->
  <svg
    width={size}
    height={size}
    viewBox="0 0 32 32"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    role="img"
    aria-label="Store"
  >
    <rect width="32" height="32" rx="6" fill="var(--void-raised)" stroke="var(--border-subtle)" stroke-width="1" />
    <text
      x="16"
      y="22"
      text-anchor="middle"
      fill="var(--text-dim)"
      font-family="system-ui, sans-serif"
      font-size="16"
      font-weight="600"
    >?</text>
  </svg>
{/if}
