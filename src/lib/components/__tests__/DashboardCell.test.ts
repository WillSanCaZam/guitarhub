import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import DashboardCell from '../DashboardCell.svelte';

describe('DashboardCell', () => {
  it('renders title and icon', () => {
    render(DashboardCell, { props: { title: 'Search', icon: '🔍' } });
    expect(screen.getByText('Search')).toBeInTheDocument();
    expect(screen.getByText('🔍')).toBeInTheDocument();
  });

  it('shows loading state', () => {
    render(DashboardCell, { props: { title: 'Test', loading: true } });
    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });

  it('shows empty state', () => {
    render(DashboardCell, { props: { title: 'Test', empty: true, emptyMessage: 'Nothing here' } });
    expect(screen.getByText('Nothing here')).toBeInTheDocument();
  });

  it('has glassmorphism class on root element', () => {
    render(DashboardCell, { props: { title: 'Test' } });
    const cell = screen.getByRole('region');
    expect(cell.classList.contains('glassmorphism')).toBe(true);
  });
});
