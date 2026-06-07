<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  let alertChannel = $state('app');
  let alertConfig = $state('');
  let testResult = $state(null);
  let testLoading = $state(false);
  let exportResult = $state(null);
  let saving = $state(false);
  let saved = $state(false);
  let allowedImageDomains = $state('');

  onMount(async () => {
    try {
      const savedChannel = await invoke('get_setting', { key: 'alert_channel' });
      if (savedChannel) {
        alertChannel = savedChannel;
      }
      const savedConfig = await invoke('get_setting', { key: 'alert_config' });
      if (savedConfig) {
        alertConfig = savedConfig;
      }
      const savedDomains = await invoke('get_setting', { key: 'allowed_image_domains' });
      if (savedDomains) {
        allowedImageDomains = savedDomains;
      }
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  });

  async function onChannelChange(newChannel) {
    alertChannel = newChannel;
    try {
      await invoke('save_setting', { key: 'alert_channel', value: newChannel });
    } catch (e) {
      console.error('Failed to save channel:', e);
    }
  }

  async function onConfigChange() {
    try {
      await invoke('save_setting', { key: 'alert_config', value: alertConfig });
    } catch (e) {
      console.error('Failed to save config:', e);
    }
  }

  async function testNotification() {
    testLoading = true;
    testResult = null;
    try {
      const res = await invoke('test_alert_channel', {
        channel: alertChannel,
        config: alertChannel === 'app' ? '' : alertConfig,
      });
      testResult = { success: res.success, message: res.message };
    } catch (e) {
      testResult = { success: false, message: String(e) };
    } finally {
      testLoading = false;
    }
  }

  async function exportData() {
    try {
      // Dynamic import to avoid bundling issue when plugin is not available
      const { save } = await import('@tauri-apps/plugin-dialog');
      const path = await save({
        filters: [{ name: 'ZIP Archive', extensions: ['zip'] }],
      });
      if (!path) return; // user cancelled
      const result = await invoke('export_data', { path });
      exportResult = result;
    } catch (e) {
      exportResult = { success: false, size_bytes: 0, file_count: 0 };
    }
  }

  async function onDomainsChange() {
    try {
      await invoke('save_setting', { key: 'allowed_image_domains', value: allowedImageDomains });
    } catch (e) {
      console.error('Failed to save allowed image domains:', e);
    }
  }

  async function saveAll() {
    saving = true;
    saved = false;
    try {
      await invoke('save_setting', { key: 'alert_channel', value: alertChannel });
      await invoke('save_setting', { key: 'alert_config', value: alertConfig });
      await invoke('save_setting', { key: 'allowed_image_domains', value: allowedImageDomains });
      saved = true;
      setTimeout(() => {
        saved = false;
      }, 2000);
    } catch (e) {
      console.error('Failed to save settings:', e);
    } finally {
      saving = false;
    }
  }

</script>

<div class="settings" data-testid="settings-form">
  <h2>Settings</h2>

  <fieldset class="channel-section">
    <legend>Alert Channel</legend>

    <label class="radio-option">
      <input
        type="radio"
        name="channel"
        value="app"
        checked={alertChannel === 'app'}
        onchange={() => onChannelChange('app')}
      />
      App notifications (default)
    </label>

    <label class="radio-option">
      <input
        type="radio"
        name="channel"
        value="ntfy"
        checked={alertChannel === 'ntfy'}
        onchange={() => onChannelChange('ntfy')}
      />
      Ntfy.sh
    </label>

    <label class="radio-option">
      <input
        type="radio"
        name="channel"
        value="webhook"
        checked={alertChannel === 'webhook'}
        onchange={() => onChannelChange('webhook')}
      />
      Webhook POST
    </label>

    {#if alertChannel === 'ntfy' || alertChannel === 'webhook'}
      <div class="config-input">
        <label for="alert-config">
          {alertChannel === 'ntfy' ? 'Topic' : 'URL'}:
        </label>
        <input
          id="alert-config"
          type="text"
          bind:value={alertConfig}
          oninput={onConfigChange}
          placeholder={alertChannel === 'ntfy' ? 'guitar-deals' : 'https://hooks.example.com/alert'}
        />
      </div>
    {/if}

    <button onclick={testNotification} disabled={testLoading}>
      {testLoading ? 'Sending...' : 'Test Notification'}
    </button>

    {#if testResult}
      <p class="test-result" class:success={testResult.success} class:error={!testResult.success}>
        {testResult.success ? 'Sent!' : 'Failed: ' + testResult.message}
      </p>
    {/if}

    <button type="submit" onclick={saveAll} disabled={saving}>
      {saved ? 'Saved ✓' : (saving ? 'Saving...' : 'Save')}
    </button>
  </fieldset>

  <fieldset class="domain-section">
    <legend>Image Domain Allowlist</legend>

    <div class="config-input">
      <label for="allowed-image-domains">
        Allowed domains (comma-separated):
      </label>
      <input
        id="allowed-image-domains"
        type="text"
        bind:value={allowedImageDomains}
        oninput={onDomainsChange}
        placeholder="reverb.com, mlstatic.com"
      />
    </div>
    <p class="hint">
      Only images from these domains will be loaded. Leave empty to use the defaults (reverb.com, mlstatic.com).
    </p>
  </fieldset>

  <fieldset class="export-section">
    <legend>Data Export</legend>

    <button onclick={exportData}>Export All Data</button>

    {#if exportResult}
      <p class="export-result">
        {exportResult.success
          ? 'Exported: ' + exportResult.size_bytes + ' bytes (' + exportResult.file_count + ' files)'
          : 'Export failed'}
      </p>
    {/if}
  </fieldset>
</div>

<style>
  .settings {
    max-width: 480px;
    padding: 16px;
  }
  .settings h2 {
    margin: 0 0 16px;
    font-size: 1.25rem;
  }
  fieldset {
    border: 1px solid #ddd;
    border-radius: 8px;
    padding: 16px;
    margin-bottom: 16px;
  }
  legend {
    font-weight: 600;
    padding: 0 8px;
  }
  .radio-option {
    display: block;
    margin: 8px 0;
    cursor: pointer;
  }
  .radio-option input {
    margin-right: 8px;
  }
  .config-input {
    margin: 12px 0;
  }
  .config-input label {
    display: block;
    font-size: 0.85rem;
    margin-bottom: 4px;
    color: #666;
  }
  .config-input input {
    width: 100%;
    padding: 8px;
    border: 1px solid #ccc;
    border-radius: 4px;
    box-sizing: border-box;
  }
  button {
    padding: 8px 16px;
    border: 1px solid #888;
    border-radius: 4px;
    background: #fff;
    cursor: pointer;
    font-size: 0.9rem;
  }
  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .test-result,
  .export-result {
    margin: 8px 0 0;
    font-size: 0.85rem;
  }
  .success {
    color: #155724;
  }
  .error {
    color: #721c24;
  }
</style>
