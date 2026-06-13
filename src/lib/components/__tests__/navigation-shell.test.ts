import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Sidebar from '../layout/Sidebar.svelte';
import BottomNav from '../layout/BottomNav.svelte';
import SnippetTestWrapper from './SnippetTestWrapper.svelte';

// Mock svelte-routing's `link` and `useLocation`
vi.mock('svelte-spa-router', () => ({
  link: () => '',
  useLocation: () => ({ pathname: '/' }),
}));

describe('Navigation Shell', () => {
  describe('Sidebar', () => {
    it('renders GuitarHub logo', () => {
      render(Sidebar, { props: { currentPath: '/', serverReachable: false } });
      expect(screen.getByText(/GuitarHub/)).toBeInTheDocument();
    });

    it('renders all navigation items', () => {
      render(Sidebar, { props: { currentPath: '/', serverReachable: false } });
      expect(screen.getByText('Feed')).toBeInTheDocument();
      expect(screen.getByText('Explore')).toBeInTheDocument();
      expect(screen.getByText('Lessons')).toBeInTheDocument();
      expect(screen.getByText('My Gear')).toBeInTheDocument();
      expect(screen.getByText('Saved Riffs')).toBeInTheDocument();
      expect(screen.getByText('Profile')).toBeInTheDocument();
    });

    it('renders settings link', () => {
      render(Sidebar, { props: { currentPath: '/', serverReachable: false } });
      expect(screen.getByText('Settings')).toBeInTheDocument();
    });

    it('marks active route', () => {
      render(Sidebar, { props: { currentPath: '/feed', serverReachable: false } });
      const feedLink = screen.getByText('Feed').closest('a');
      expect(feedLink).toHaveClass('active');
    });

    it('shows offline indicator for community items when server unreachable', () => {
      render(Sidebar, { props: { currentPath: '/', serverReachable: false } });
      const feedLink = screen.getByText('Feed').closest('a');
      expect(feedLink?.querySelector('.offline-badge')).toBeInTheDocument();
    });
  });

  describe('BottomNav', () => {
    it('renders 5 navigation icons', () => {
      render(BottomNav, { props: { currentPath: '/', serverReachable: false } });
      const navItems = screen.getAllByRole('link');
      expect(navItems.length).toBe(5);
    });

    it('renders navigation items', () => {
      render(BottomNav, { props: { currentPath: '/', serverReachable: false } });
      expect(screen.getByText('Feed')).toBeInTheDocument();
      expect(screen.getByText('Explore')).toBeInTheDocument();
      expect(screen.getByText('My Gear')).toBeInTheDocument();
      expect(screen.getByText('Riffs')).toBeInTheDocument();
      expect(screen.getByText('Profile')).toBeInTheDocument();
    });

    it('marks active route', () => {
      render(BottomNav, { props: { currentPath: '/feed', serverReachable: false } });
      const feedLink = screen.getByText('Feed').closest('a');
      expect(feedLink).toHaveAttribute('aria-current', 'page');
    });
  });

  describe('AppShell', () => {
    it('renders sidebar on desktop', () => {
      render(SnippetTestWrapper, {
        props: { component: 'appshell', text: 'page content' },
      });
      expect(screen.getByText(/GuitarHub/)).toBeInTheDocument();
    });

    it('renders content area', () => {
      render(SnippetTestWrapper, {
        props: { component: 'appshell', text: 'page content' },
      });
      expect(screen.getByText('page content')).toBeInTheDocument();
    });
  });
});
