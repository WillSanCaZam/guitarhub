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

  it('renders full 3-line title when all optional fields are present', () => {
    render(PriceBadge, {
      props: {
        level: 'green',
        pct: 15,
        confidence: 78,
        cnt_30d: 45,
        source_count_30d: 2,
        last_recorded_at: 2,
        min_30d: 850.00,
        avg_90d: 950.00,
        current: 899.00,
      },
    });
    const badge = screen.getByRole('status');
    const title = badge.getAttribute('title');
    expect(title).not.toBeNull();
    const lines = title!.split('\n');
    expect(lines).toHaveLength(3);
    expect(lines[0]).toBe('Confidence: 78% (Medium)');
    expect(lines[1]).toBe('45 data points · 2 sources · last 2 days ago');
    expect(lines[2]).toBe('Min 30d: $850.00  |  Avg 90d: $950.00  |  Current: $899.00');
  });

  it('omits missing fields gracefully from title', () => {
    render(PriceBadge, {
      props: {
        level: 'green',
        pct: 15,
        confidence: 78,
      },
    });
    const badge = screen.getByRole('status');
    const title = badge.getAttribute('title');
    expect(title).not.toBeNull();
    const lines = title!.split('\n');
    expect(lines).toHaveLength(1);
    expect(lines[0]).toBe('Confidence: 78% (Medium)');
  });

  it('includes full context in aria-label when optional fields are present', () => {
    render(PriceBadge, {
      props: {
        level: 'green',
        pct: 15,
        confidence: 85,
        cnt_30d: 45,
        source_count_30d: 2,
        last_recorded_at: 2,
      },
    });
    const badge = screen.getByRole('status');
    expect(badge).toHaveAttribute(
      'aria-label',
      'Good price, 15% above 30-day low. Confidence 85%, high. 45 data points, 2 sources, last 2 days ago'
    );
  });
});
