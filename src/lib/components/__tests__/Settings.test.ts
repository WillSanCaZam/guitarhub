import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import Settings from '../Settings.svelte';
import { invoke } from '@tauri-apps/api/core';

describe('Settings', () => {
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
});
