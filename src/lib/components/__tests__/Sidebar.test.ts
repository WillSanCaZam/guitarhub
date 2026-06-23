import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import Sidebar from '../layout/Sidebar.svelte';

const defaultProps = {
  currentPath: '/',
  serverReachable: true,
  syncing: false,
  collapsed: false,
  onSync: vi.fn(),
  onToggleCollapse: vi.fn(),
};

describe('Sidebar', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders expanded by default', () => {
    render(Sidebar, { props: defaultProps });
    const sidebar = screen.getByTestId('sidebar');
    expect(sidebar).not.toHaveClass('collapsed');
  });

  it('renders collapsed when collapsed prop is true', () => {
    render(Sidebar, { props: { ...defaultProps, collapsed: true } });
    const sidebar = screen.getByTestId('sidebar');
    expect(sidebar).toHaveClass('collapsed');
  });

  it('shows nav labels when expanded', () => {
    render(Sidebar, { props: defaultProps });
    expect(screen.getByText('My Gear')).toBeInTheDocument();
    expect(screen.getByText('Wishlist')).toBeInTheDocument();
  });

  it('calls onToggleCollapse when collapse button is clicked', async () => {
    const onToggleCollapse = vi.fn();
    render(Sidebar, { props: { ...defaultProps, onToggleCollapse } });
    const toggleBtn = screen.getByRole('button', { name: /collapse sidebar/i });
    await fireEvent.click(toggleBtn);
    expect(onToggleCollapse).toHaveBeenCalledTimes(1);
  });

  it('shows expand label when collapsed', () => {
    render(Sidebar, { props: { ...defaultProps, collapsed: true } });
    const toggleBtn = screen.getByRole('button', { name: /expand sidebar/i });
    expect(toggleBtn).toBeInTheDocument();
  });

  it('renders active indicator for current path', () => {
    render(Sidebar, { props: { ...defaultProps, currentPath: '/' } });
    const indicators = document.querySelectorAll('.active-indicator.visible');
    expect(indicators.length).toBe(1);
  });

  it('disables items with badge off', () => {
    render(Sidebar, { props: defaultProps });
    const lessonsLink = screen.getByText('Lessons').closest('a');
    expect(lessonsLink).toHaveAttribute('aria-disabled', 'true');
    expect(lessonsLink).toHaveAttribute('tabindex', '-1');
  });

  it('shows PRÓXIMO badge for off items', () => {
    render(Sidebar, { props: defaultProps });
    const proximoElements = screen.getAllByText('PRÓXIMO');
    expect(proximoElements.length).toBeGreaterThan(0);
  });
});
