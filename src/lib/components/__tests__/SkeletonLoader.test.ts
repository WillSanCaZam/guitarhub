import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/svelte'
import SkeletonLoader from '../ui/SkeletonLoader.svelte'

describe('SkeletonLoader', () => {
  it('renders card-grid variant with correct structure', () => {
    render(SkeletonLoader, { props: { variant: 'card-grid', count: 3 } })
    const grid = screen.getByLabelText('Loading content')
    expect(grid).toBeInTheDocument()
    expect(grid.querySelectorAll('.skeleton-card')).toHaveLength(3)
  })

  it('renders card-list variant', () => {
    render(SkeletonLoader, { props: { variant: 'card-list', count: 2 } })
    const list = screen.getByLabelText('Loading content')
    expect(list).toBeInTheDocument()
    expect(list.querySelectorAll('.skeleton-card-row')).toHaveLength(2)
  })

  it('renders text variant with correct line count', () => {
    render(SkeletonLoader, { props: { variant: 'text', count: 4 } })
    const textBlock = screen.getByLabelText('Loading text')
    expect(textBlock).toBeInTheDocument()
    expect(textBlock.querySelectorAll('.skeleton-text-line')).toHaveLength(4)
  })

  it('renders hero variant', () => {
    render(SkeletonLoader, { props: { variant: 'hero' } })
    const hero = screen.getByLabelText('Loading hero')
    expect(hero).toBeInTheDocument()
    expect(hero.querySelector('.skeleton-hero-title')).toBeInTheDocument()
    expect(hero.querySelector('.skeleton-hero-subtitle')).toBeInTheDocument()
    expect(hero.querySelector('.skeleton-hero-search')).toBeInTheDocument()
  })

  it('renders detail variant', () => {
    render(SkeletonLoader, { props: { variant: 'detail' } })
    const detail = screen.getByLabelText('Loading product details')
    expect(detail).toBeInTheDocument()
    expect(detail.querySelector('.skeleton-gallery')).toBeInTheDocument()
    expect(detail.querySelector('.skeleton-info')).toBeInTheDocument()
  })

  it('defaults count to 1 when not provided', () => {
    render(SkeletonLoader, { props: { variant: 'text' } })
    const textBlock = screen.getByLabelText('Loading text')
    expect(textBlock.querySelectorAll('.skeleton-text-line')).toHaveLength(1)
  })

  it('has aria-busy attribute for accessibility', () => {
    render(SkeletonLoader, { props: { variant: 'card-grid' } })
    const el = screen.getByLabelText('Loading content')
    expect(el).toHaveAttribute('aria-busy', 'true')
  })
})
