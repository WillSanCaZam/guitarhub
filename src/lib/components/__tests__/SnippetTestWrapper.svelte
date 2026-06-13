<script lang="ts">
  import Button from '$lib/components/ui/Button.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import AppShell from '$lib/components/layout/AppShell.svelte';
  import AuthGuard from '$lib/components/auth/AuthGuard.svelte';

  interface Props {
    component: 'button' | 'card' | 'appshell' | 'authguard';
    variant?: 'primary' | 'secondary' | 'ghost';
    disabled?: boolean;
    media?: string;
    text?: string;
    currentPath?: string;
    serverReachable?: boolean;
  }

  let {
    component,
    variant,
    disabled,
    media,
    text = 'Child content',
    currentPath = '/',
    serverReachable = false,
  }: Props = $props();
</script>

{#if component === 'button'}
  <Button {variant} {disabled}>{text}</Button>
{:else if component === 'card'}
  <Card {media}>{text}</Card>
{:else if component === 'appshell'}
  <AppShell {currentPath} {serverReachable}>{text}</AppShell>
{:else if component === 'authguard'}
  <AuthGuard>{text}</AuthGuard>
{/if}
