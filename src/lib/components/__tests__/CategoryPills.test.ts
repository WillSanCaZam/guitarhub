import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import CategoryPills from '../ui/CategoryPills.svelte';

const mockCategories = [
  { id: 'guitars', label: 'Guitars', icon: '🎸' },
  { id: 'amps', label: 'Amps', icon: '🔊' },
  { id: 'pedals', label: 'Pedals', icon: '🎛️', count: 42 },
];

describe('CategoryPills', () => {
  it('renders all category labels', () => {
    render(CategoryPills, {
      props: {
        categories: mockCategories,
        selected: [],
        onToggle: vi.fn(),
      },
    });
    expect(screen.getByText('Guitars')).toBeInTheDocument();
    expect(screen.getByText('Amps')).toBeInTheDocument();
    expect(screen.getByText('Pedals')).toBeInTheDocument();
  });

  it('renders icons', () => {
    render(CategoryPills, {
      props: {
        categories: mockCategories,
        selected: [],
        onToggle: vi.fn(),
      },
    });
    expect(screen.getByText('🎸')).toBeInTheDocument();
    expect(screen.getByText('🔊')).toBeInTheDocument();
    expect(screen.getByText('🎛️')).toBeInTheDocument();
  });

  it('shows count badge when count is provided', () => {
    render(CategoryPills, {
      props: {
        categories: mockCategories,
        selected: [],
        onToggle: vi.fn(),
      },
    });
    expect(screen.getByText('42')).toBeInTheDocument();
  });

  it('does not render count badge when count is absent', () => {
    const { container } = render(CategoryPills, {
      props: {
        categories: mockCategories,
        selected: [],
        onToggle: vi.fn(),
      },
    });
    const guitarsPill = screen.getByText('Guitars').closest('button');
    expect(guitarsPill?.querySelector('.cat-count')).not.toBeInTheDocument();
  });

  it('calls onToggle with category id when clicked', async () => {
    const onToggle = vi.fn();
    render(CategoryPills, {
      props: {
        categories: mockCategories,
        selected: [],
        onToggle,
      },
    });
    await fireEvent.click(screen.getByText('Guitars'));
    expect(onToggle).toHaveBeenCalledWith('guitars');
  });

  it('applies active class when selected', () => {
    const { container } = render(CategoryPills, {
      props: {
        categories: mockCategories,
        selected: ['guitars'],
        onToggle: vi.fn(),
      },
    });
    const pills = container.querySelectorAll('.category-pill');
    expect(pills[0].classList.contains('active')).toBe(true);
    expect(pills[1].classList.contains('active')).toBe(false);
  });

  it('sets aria-pressed on active pills', () => {
    render(CategoryPills, {
      props: {
        categories: mockCategories,
        selected: ['amps'],
        onToggle: vi.fn(),
      },
    });
    const ampsBtn = screen.getByText('Amps').closest('button');
    expect(ampsBtn).toHaveAttribute('aria-pressed', 'true');
    const guitarsBtn = screen.getByText('Guitars').closest('button');
    expect(guitarsBtn).toHaveAttribute('aria-pressed', 'false');
  });

  it('has group role with aria-label', () => {
    const { container } = render(CategoryPills, {
      props: {
        categories: mockCategories,
        selected: [],
        onToggle: vi.fn(),
      },
    });
    const group = container.querySelector('[role="group"]');
    expect(group).toBeInTheDocument();
    expect(group).toHaveAttribute('aria-label', 'Categories');
  });

  it('includes count in aria-label when provided', () => {
    render(CategoryPills, {
      props: {
        categories: mockCategories,
        selected: [],
        onToggle: vi.fn(),
      },
    });
    const pedalsBtn = screen.getByText('Pedals').closest('button');
    expect(pedalsBtn).toHaveAttribute('aria-label', 'Pedals, 42 items');
  });

  it('is keyboard accessible — buttons are focusable', async () => {
    render(CategoryPills, {
      props: {
        categories: mockCategories,
        selected: [],
        onToggle: vi.fn(),
      },
    });
    const guitarsBtn = screen.getByText('Guitars').closest('button')!;
    const ampsBtn = screen.getByText('Amps').closest('button')!;
    guitarsBtn.focus();
    expect(document.activeElement).toBe(guitarsBtn);
    ampsBtn.focus();
    expect(document.activeElement).toBe(ampsBtn);
  });
});
