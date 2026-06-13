import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import Settings from '../Settings.svelte';
import { invoke } from '@tauri-apps/api/core';

describe('Settings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('save button shows "Saved" feedback after click', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    render(Settings);
    const saveBtn = screen.getByRole('button', { name: /save/i });
    await fireEvent.click(saveBtn);
    await waitFor(() => {
      expect(screen.getByText('Saved ✓')).toBeInTheDocument();
    });
  });

  it('save button is disabled while saving', async () => {
    vi.mocked(invoke).mockImplementation(() => new Promise(resolve => setTimeout(resolve, 50)));
    render(Settings);
    const saveBtn = screen.getByRole('button', { name: /save/i });
    await fireEvent.click(saveBtn);
    expect(saveBtn).toBeDisabled();
  });

  it('save button reverts to "Save" after 2 seconds', async () => {
    vi.useFakeTimers();
    vi.mocked(invoke).mockResolvedValue(undefined);
    render(Settings);
    const saveBtn = screen.getByRole('button', { name: /save/i });
    await fireEvent.click(saveBtn);
    await waitFor(() => {
      expect(screen.getByText('Saved ✓')).toBeInTheDocument();
    });
    vi.advanceTimersByTime(2000);
    expect(screen.getByRole('button', { name: /save/i })).toBeInTheDocument();
    vi.useRealTimers();
  });

  describe('Community Server', () => {
    it('loads community_server_url on mount', async () => {
      vi.mocked(invoke).mockImplementation(async (cmd: string, opts?: unknown) => {
        const args = opts as Record<string, unknown> | undefined;
        if (cmd === 'get_setting' && args?.key === 'community_server_url') {
          return 'https://community.example.com';
        }
        return null;
      });
      render(Settings);
      await waitFor(() => {
        const input = screen.getByLabelText(/server url/i);
        expect(input).toHaveValue('https://community.example.com');
      });
    });

    it('renders community server fieldset with text input', async () => {
      vi.mocked(invoke).mockResolvedValue(null);
      render(Settings);
      await waitFor(() => {
        expect(screen.getByText('Community Server')).toBeInTheDocument();
        expect(screen.getByLabelText(/server url/i)).toBeInTheDocument();
      });
    });

    it('renders test connection button', async () => {
      vi.mocked(invoke).mockResolvedValue(null);
      render(Settings);
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /test connection/i })).toBeInTheDocument();
      });
    });

    it('test connection invokes health_check and shows success', async () => {
      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'health_check') return true;
        return null;
      });
      render(Settings);
      await waitFor(() => {
        expect(screen.getByLabelText(/server url/i)).toBeInTheDocument();
      });
      const testBtn = screen.getByRole('button', { name: /test connection/i });
      await fireEvent.click(testBtn);
      await waitFor(() => {
        expect(screen.getByText('Connected!')).toBeInTheDocument();
      });
      expect(invoke).toHaveBeenCalledWith('health_check', {
        server_url: expect.any(String),
      });
    });

    it('test connection shows failure for unreachable server', async () => {
      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'health_check') return false;
        return null;
      });
      render(Settings);
      await waitFor(() => {
        expect(screen.getByLabelText(/server url/i)).toBeInTheDocument();
      });
      const testBtn = screen.getByRole('button', { name: /test connection/i });
      await fireEvent.click(testBtn);
      await waitFor(() => {
        expect(screen.getByText(/unreachable/i)).toBeInTheDocument();
      });
    });

    it('saveAll persists community_server_url', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);
      render(Settings);
      await waitFor(() => {
        expect(screen.getByLabelText(/server url/i)).toBeInTheDocument();
      });
      const saveBtn = screen.getByRole('button', { name: /save/i });
      await fireEvent.click(saveBtn);
      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('save_setting', {
          key: 'community_server_url',
          value: expect.any(String),
        });
      });
    });
  });
});
