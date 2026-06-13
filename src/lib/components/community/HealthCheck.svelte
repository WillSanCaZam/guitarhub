<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!--
  HealthCheck — Server health check component.
  Pings the server on mount and sets the serverReachable flag.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { checkServerHealth } from '$lib/stores/auth.svelte';

  onMount(() => {
    checkServerHealth();
  });
</script>

<!-- HealthCheck is a silent component — renders nothing, just checks server health -->
