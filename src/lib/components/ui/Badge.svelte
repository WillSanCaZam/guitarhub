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
    border-radius: var(--radius-full);
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    font-weight: 500;
    line-height: 1.4;
    letter-spacing: 0.03em;
    white-space: nowrap;
  }

  .badge--primary {
    background: var(--color-primary-container);
    color: var(--color-on-primary-container);
  }

  .badge--secondary {
    background: var(--color-secondary-container);
    color: var(--color-on-secondary-container);
  }

  .badge--success {
    background: var(--color-success-container);
    color: var(--color-on-success-container);
  }

  .badge--warning {
    background: var(--color-warning-container);
    color: var(--color-on-warning-container);
  }

  .badge--error {
    background: var(--color-error-container);
    color: var(--color-on-error-container);
  }

  .badge--info {
    background: var(--color-info-container);
    color: var(--color-on-info-container);
  }
</style>
