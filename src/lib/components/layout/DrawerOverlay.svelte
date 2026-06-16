<script lang="ts">
  interface Props {
    open: boolean;
    onclose: () => void;
  }

  let { open, onclose }: Props = $props();
</script>

<div
  class="overlay"
  class:open
  onclick={onclose}
  aria-hidden="true"
  inert={!open}
></div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: var(--z-overlay);
    opacity: 0;
    pointer-events: none;
    transition: opacity 200ms ease;
  }

  .overlay.open {
    opacity: 1;
    pointer-events: auto;
  }
</style>