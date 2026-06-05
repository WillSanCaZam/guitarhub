import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import PriceBadge from '../PriceBadge.svelte';

describe('PriceBadge', () => {
  it('renders green badge with 3 dots for high confidence', () => {
    render(PriceBadge, { props: { level: 'green', pct: 15, confidence: 85 } });
    const badge = screen.getByRole('status');
    expect(badge).toHaveTextContent('Good price');
    expect(badge).toHaveTextContent('•••');
  });

  it('renders amber badge with 2 dots for medium confidence', () => {
    render(PriceBadge, { props: { level: 'amber', pct: 20, confidence: 60 } });
    const badge = screen.getByRole('status');
    expect(badge).toHaveTextContent('Above average');
    expect(badge).toHaveTextContent('••○');
  });
});
