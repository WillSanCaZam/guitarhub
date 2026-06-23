import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/svelte'
import EmptyState from '../ui/EmptyState.svelte'

describe('EmptyState', () => {
  it('renders collection variant with title and description', () => {
    render(EmptyState, {
      props: {
        variant: 'collection',
        title: 'No gear yet',
        description: 'Start adding gear to your collection.',
      },
    })
    expect(screen.getByText('No gear yet')).toBeInTheDocument()
    expect(screen.getByText('Start adding gear to your collection.')).toBeInTheDocument()
  })

  it('renders wishlist variant', () => {
    render(EmptyState, {
      props: {
        variant: 'wishlist',
        title: 'No favorites yet',
        description: 'Search for gear and tap the heart.',
      },
    })
    expect(screen.getByText('No favorites yet')).toBeInTheDocument()
  })

  it('renders search variant', () => {
    render(EmptyState, {
      props: {
        variant: 'search',
        title: 'No results found',
        description: 'Try a different search term.',
      },
    })
    expect(screen.getByText('No results found')).toBeInTheDocument()
  })

  it('renders feed variant', () => {
    render(EmptyState, {
      props: {
        variant: 'feed',
        title: 'No feed items',
        description: 'Follow some users.',
      },
    })
    expect(screen.getByText('No feed items')).toBeInTheDocument()
  })

  it('renders lessons variant', () => {
    render(EmptyState, {
      props: {
        variant: 'lessons',
        title: 'No lessons found',
      },
    })
    expect(screen.getByText('No lessons found')).toBeInTheDocument()
  })

  it('renders riffs variant', () => {
    render(EmptyState, {
      props: {
        variant: 'riffs',
        title: 'No saved riffs',
      },
    })
    expect(screen.getByText('No saved riffs')).toBeInTheDocument()
  })

  it('renders default variant', () => {
    render(EmptyState, {
      props: {
        variant: 'default',
        title: 'Nothing here',
      },
    })
    expect(screen.getByText('Nothing here')).toBeInTheDocument()
  })

  it('shows action button when actionLabel and onAction are provided', () => {
    const onAction = vi.fn()
    render(EmptyState, {
      props: {
        variant: 'collection',
        title: 'Empty',
        actionLabel: 'Browse Gear',
        onAction,
      },
    })
    const btn = screen.getByRole('button', { name: 'Browse Gear' })
    expect(btn).toBeInTheDocument()
    fireEvent.click(btn)
    expect(onAction).toHaveBeenCalledOnce()
  })

  it('hides action button when actionLabel is not provided', () => {
    render(EmptyState, {
      props: {
        variant: 'collection',
        title: 'Empty',
      },
    })
    expect(screen.queryByRole('button')).not.toBeInTheDocument()
  })

  it('does not render description when not provided', () => {
    render(EmptyState, {
      props: {
        variant: 'default',
        title: 'Title only',
      },
    })
    expect(screen.getByText('Title only')).toBeInTheDocument()
    expect(screen.queryByRole('paragraph')).not.toBeInTheDocument()
  })

  it('renders SVG illustration', () => {
    render(EmptyState, {
      props: {
        variant: 'collection',
        title: 'Empty',
      },
    })
    const illustration = document.querySelector('.empty-illustration svg')
    expect(illustration).toBeInTheDocument()
  })

  it('has correct variant illustration for search', () => {
    render(EmptyState, {
      props: {
        variant: 'search',
        title: 'Empty',
      },
    })
    const illustration = document.querySelector('.empty-illustration svg')
    expect(illustration).toBeInTheDocument()
  })
})
