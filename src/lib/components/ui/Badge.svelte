<script lang="ts">
  interface Props {
    label: string;
    variant?: 'primary' | 'secondary' | 'success' | 'warning' | 'error' | 'info';
  }

  let { label, variant = 'primary' }: Props = $props();
</script>

<span class="badge badge--{variant}" role="status">
  {label}
</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    padding: 2px 8px;
    border-radius: var(--radius-full, 9999px);
    font-family: var(--font-mono, monospace);
    font-size: 0.6875rem;
    font-weight: 500;
    line-height: 1.4;
    letter-spacing: 0.03em;
    white-space: nowrap;
  }

  .badge--primary {
    background: var(--color-primary-container, #3d3520);
    color: var(--color-on-primary-container, #f5e6b8);
  }

  .badge--secondary {
    background: var(--color-secondary-container, #1e3a5f);
    color: var(--color-on-secondary-container, #c8ddf5);
  }

  .badge--success {
    background: var(--color-success-container, #1a3a1c);
    color: var(--color-on-success-container, #a5d6a7);
  }

  .badge--warning {
    background: var(--color-warning-container, #3d2a10);
    color: var(--color-on-warning-container, #ffcc80);
  }

  .badge--error {
    background: var(--color-error-container, #3d1a18);
    color: var(--color-on-error-container, #f8bbd0);
  }

  .badge--info {
    background: var(--color-info-container, #1a2a3d);
    color: var(--color-on-info-container, #bbdefb);
  }
</style>
