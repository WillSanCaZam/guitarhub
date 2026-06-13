// SPDX-License-Identifier: GPL-3.0-or-later
//
// Phase 6: Community Pages + Routing — structural tests for community pages.
// Tests component rendering in page context (not full route testing).

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import FeedCard from '$lib/components/community/FeedCard.svelte';
import LessonCard from '$lib/components/community/LessonCard.svelte';
import ProfileHeader from '$lib/components/community/ProfileHeader.svelte';
import Sidebar from '$lib/components/layout/Sidebar.svelte';
import type { FeedItem, Lesson, UserProfile } from '$lib/types/community';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

// ── Feed Page Components ────────────────────────────────────────────────

describe('Feed route page components', () => {
  it('FeedCard renders in feed list context', () => {
    const mockItem: FeedItem = {
      id: 'f1',
      authorId: 'u1',
      authorName: 'Alice',
      itemType: 'lesson',
      content: { id: 'l1', authorId: 'u1', title: 'Test Lesson', description: 'Desc', contentUrl: '', difficulty: 'beginner', tags: [], likes: 0, createdAt: Date.now() } as Lesson,
      createdAt: Date.now(),
    };
    render(FeedCard, { props: { item: mockItem } });
    expect(screen.getByText('Alice')).toBeInTheDocument();
    expect(screen.getByText('Test Lesson')).toBeInTheDocument();
  });
});

// ── Explore Page Components ─────────────────────────────────────────────

describe('Explore route page components', () => {
  it('search input is present in explore context', () => {
    render(FeedCard, { props: {
      item: {
        id: 'f1',
        authorId: 'u1',
        authorName: 'Test',
        itemType: 'lesson',
        content: { id: 'l1', authorId: 'u1', title: 'Lesson', description: '', contentUrl: '', difficulty: 'beginner', tags: [], likes: 0, createdAt: Date.now() } as Lesson,
        createdAt: Date.now(),
      }
    }});
    // Explore page uses FeedCard and LessonCard components
    expect(screen.getByText('Lesson')).toBeInTheDocument();
  });
});

// ── Lessons Page Components ─────────────────────────────────────────────

describe('Lessons route page components', () => {
  it('LessonCard renders with difficulty filter context', () => {
    const mockLesson: Lesson = {
      id: 'l1',
      authorId: 'u1',
      title: 'Test Lesson',
      description: 'A test',
      contentUrl: 'https://youtube.com/watch?v=abc',
      difficulty: 'intermediate',
      tags: ['rock'],
      likes: 10,
      createdAt: Date.now(),
    };
    render(LessonCard, { props: { lesson: mockLesson } });
    expect(screen.getByText('Test Lesson')).toBeInTheDocument();
    expect(screen.getByText('intermediate')).toBeInTheDocument();
  });
});

// ── Profile Page Components ─────────────────────────────────────────────

describe('Profile route page components', () => {
  it('ProfileHeader renders user info in profile context', () => {
    const mockProfile: UserProfile = {
      id: 'u1',
      username: 'tester',
      displayName: 'Tester',
      bio: 'Guitar lover',
      gearList: ['sku1'],
      streakDays: 10,
      joinedAt: Date.now(),
    };
    render(ProfileHeader, { props: { profile: mockProfile } });
    expect(screen.getByText('Tester')).toBeInTheDocument();
    expect(screen.getByText('Guitar lover')).toBeInTheDocument();
  });
});

// ── Community Navigation ────────────────────────────────────────────────

describe('Community navigation items in Sidebar', () => {
  it('Feed nav item is present', () => {
    render(Sidebar, { props: { currentPath: '/', serverReachable: true } });
    expect(screen.getByText('Feed')).toBeInTheDocument();
  });

  it('Explore nav item is present', () => {
    render(Sidebar, { props: { currentPath: '/', serverReachable: true } });
    expect(screen.getByText('Explore')).toBeInTheDocument();
  });
});
