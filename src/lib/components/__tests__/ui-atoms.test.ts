import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Avatar from '../ui/Avatar.svelte';
import Badge from '../ui/Badge.svelte';
import Chip from '../ui/Chip.svelte';
import Input from '../ui/Input.svelte';
import ProgressBar from '../ui/ProgressBar.svelte';
import SnippetTestWrapper from './SnippetTestWrapper.svelte';

describe('UI Atoms — Design System', () => {
  describe('Button', () => {
    it('renders with primary variant by default', () => {
      render(SnippetTestWrapper, { props: { component: 'button' } });
      const btn = screen.getByRole('button', { name: 'Child content' });
      expect(btn).toBeInTheDocument();
      expect(btn).toBeEnabled();
    });

    it('renders disabled state', () => {
      render(SnippetTestWrapper, { props: { component: 'button', disabled: true } });
      const btn = screen.getByRole('button', { name: 'Child content' });
      expect(btn).toBeDisabled();
    });

    it('renders secondary variant', () => {
      render(SnippetTestWrapper, { props: { component: 'button', variant: 'secondary' } });
      const btn = screen.getByRole('button', { name: 'Child content' });
      expect(btn).toBeInTheDocument();
    });

    it('renders ghost variant', () => {
      render(SnippetTestWrapper, { props: { component: 'button', variant: 'ghost' } });
      const btn = screen.getByRole('button', { name: 'Child content' });
      expect(btn).toBeInTheDocument();
    });
  });

  describe('Card', () => {
    it('renders children content', () => {
      render(SnippetTestWrapper, { props: { component: 'card', text: 'Card content' } });
      expect(screen.getByText('Card content')).toBeInTheDocument();
    });

    it('renders with media slot', () => {
      render(SnippetTestWrapper, {
        props: { component: 'card', media: 'https://example.com/image.jpg', text: 'Body' },
      });
      expect(screen.getByText('Body')).toBeInTheDocument();
    });
  });

  describe('Avatar', () => {
    it('renders image when url provided', () => {
      render(Avatar, {
        props: { src: 'https://example.com/avatar.jpg', alt: 'User avatar', size: 'md' },
      });
      const img = screen.getByRole('img', { name: 'User avatar' });
      expect(img).toBeInTheDocument();
      expect(img).toHaveAttribute('src', 'https://example.com/avatar.jpg');
    });

    it('renders initials fallback when no url', () => {
      render(Avatar, {
        props: { name: 'John Doe', size: 'md' },
      });
      expect(screen.getByText('JD')).toBeInTheDocument();
    });

    it('supports sm size', () => {
      render(Avatar, {
        props: { name: 'Test User', size: 'sm' },
      });
      expect(screen.getByText('TU')).toBeInTheDocument();
    });

    it('supports lg size', () => {
      render(Avatar, {
        props: { name: 'Large User', size: 'lg' },
      });
      expect(screen.getByText('LU')).toBeInTheDocument();
    });
  });

  describe('Badge', () => {
    it('renders status badge', () => {
      render(Badge, { props: { label: 'Online', variant: 'success' } });
      expect(screen.getByText('Online')).toBeInTheDocument();
    });

    it('renders role badge', () => {
      render(Badge, { props: { label: 'Admin', variant: 'primary' } });
      expect(screen.getByText('Admin')).toBeInTheDocument();
    });

    it('renders warning variant', () => {
      render(Badge, { props: { label: 'Warning', variant: 'warning' } });
      expect(screen.getByText('Warning')).toBeInTheDocument();
    });
  });

  describe('Chip', () => {
    it('renders genre tag', () => {
      render(Chip, { props: { label: 'Rock' } });
      expect(screen.getByText('Rock')).toBeInTheDocument();
    });

    it('renders difficulty tag', () => {
      render(Chip, { props: { label: 'Intermediate' } });
      expect(screen.getByText('Intermediate')).toBeInTheDocument();
    });
  });

  describe('Input', () => {
    it('renders with label', () => {
      render(Input, { props: { label: 'Email', value: '' } });
      const input = screen.getByLabelText('Email');
      expect(input).toBeInTheDocument();
    });

    it('renders with placeholder', () => {
      render(Input, { props: { placeholder: 'Enter email...', value: '' } });
      expect(screen.getByPlaceholderText('Enter email...')).toBeInTheDocument();
    });

    it('renders disabled state', () => {
      render(Input, { props: { label: 'Locked', value: 'locked', disabled: true } });
      expect(screen.getByLabelText('Locked')).toBeDisabled();
    });
  });

  describe('ProgressBar', () => {
    it('renders with value', () => {
      render(ProgressBar, { props: { value: 50, max: 100 } });
      const progressbar = screen.getByRole('progressbar');
      expect(progressbar).toBeInTheDocument();
      expect(progressbar).toHaveAttribute('aria-valuenow', '50');
    });

    it('renders at 0%', () => {
      render(ProgressBar, { props: { value: 0, max: 100 } });
      const progressbar = screen.getByRole('progressbar');
      expect(progressbar).toHaveAttribute('aria-valuenow', '0');
    });

    it('renders at 100%', () => {
      render(ProgressBar, { props: { value: 100, max: 100 } });
      const progressbar = screen.getByRole('progressbar');
      expect(progressbar).toHaveAttribute('aria-valuenow', '100');
    });
  });
});
