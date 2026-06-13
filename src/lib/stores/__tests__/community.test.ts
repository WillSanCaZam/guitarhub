// SPDX-License-Identifier: GPL-3.0-or-later
//
// Phase 5: Community Components + Stores — tests for community store, profile store,
// FeedCard, LessonCard, ProfileHeader, StreakCounter.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import { communityState, loadFeed, createLesson, likeContent } from '$lib/stores/community.svelte';
import { profileState, loadProfile, updateStreak } from '$lib/stores/profile.svelte';
import FeedCard from '$lib/components/community/FeedCard.svelte';
import LessonCard from '$lib/components/community/LessonCard.svelte';
import ProfileHeader from '$lib/components/community/ProfileHeader.svelte';
import StreakCounter from '$lib/components/community/StreakCounter.svelte';
import SnippetTestWrapper from '$lib/components/__tests__/SnippetTestWrapper.svelte';
import type { FeedItem, Lesson } from '$lib/types/community';

// ── Community Store Tests ───────────────────────────────────────────────

describe('community store', () => {
  beforeEach(() => {
    communityState.feed = [];
    communityState.lessons = [];
    communityState.riffs = [];
    communityState.loading = false;
    communityState.error = null;
    communityState.feedPage = 0;
    communityState.hasMoreFeed = true;
    vi.clearAllMocks();
  });

  it('initializes with default state', () => {
    expect(communityState.feed).toEqual([]);
    expect(communityState.lessons).toEqual([]);
    expect(communityState.loading).toBe(false);
    expect(communityState.error).toBeNull();
  });

  it('loadFeed populates feed items', async () => {
    const mockInvoke = await import('@tauri-apps/api/core');
    const mockFeed: FeedItem[] = [
      { id: '1', authorId: 'u1', authorName: 'Alice', itemType: 'lesson', content: {} as any, createdAt: Date.now() },
    ];
    vi.mocked(mockInvoke.invoke).mockResolvedValueOnce(mockFeed);

    await loadFeed();

    expect(communityState.feed).toHaveLength(1);
    expect(communityState.feed[0].id).toBe('1');
  });

  it('loadFeed handles error', async () => {
    const mockInvoke = await import('@tauri-apps/api/core');
    vi.mocked(mockInvoke.invoke).mockRejectedValueOnce(new Error('Network error'));

    await loadFeed();

    expect(communityState.error).toContain('Network error');
  });

  it('createLesson adds to lessons array', async () => {
    const mockInvoke = await import('@tauri-apps/api/core');
    vi.mocked(mockInvoke.invoke).mockResolvedValueOnce('lesson-123');

    const input = { title: 'Test', description: 'Desc', contentUrl: 'http://x', difficulty: 'beginner' as const, tags: [] };
    await createLesson(input);

    expect(communityState.lessons).toHaveLength(1);
  });
});

// ── Profile Store Tests ─────────────────────────────────────────────────

describe('profile store', () => {
  beforeEach(() => {
    profileState.profile = null;
    profileState.streak = null;
    profileState.loading = false;
    profileState.error = null;
    vi.clearAllMocks();
  });

  it('initializes with default state', () => {
    expect(profileState.profile).toBeNull();
    expect(profileState.streak).toBeNull();
    expect(profileState.loading).toBe(false);
  });

  it('loadProfile populates profile', async () => {
    const mockInvoke = await import('@tauri-apps/api/core');
    vi.mocked(mockInvoke.invoke).mockResolvedValueOnce({
      id: 'u1', username: 'tester', displayName: 'Tester', gearList: [], streakDays: 5, joinedAt: Date.now(),
    });

    await loadProfile('u1');

    expect(profileState.profile).not.toBeNull();
    expect(profileState.profile?.username).toBe('tester');
  });
});

// ── FeedCard Tests ──────────────────────────────────────────────────────

describe('FeedCard', () => {
  const mockItem: FeedItem = {
    id: 'f1',
    authorId: 'u1',
    authorName: 'Alice Johnson',
    authorAvatar: undefined,
    itemType: 'lesson',
    content: {
      id: 'l1',
      title: 'Blues Scale Basics',
      description: 'Learn the pentatonic blues scale',
    } as any,
    createdAt: Date.now() - 3600000,
  };

  it('renders author name', () => {
    render(FeedCard, { props: { item: mockItem } });
    expect(screen.getByText('Alice Johnson')).toBeInTheDocument();
  });

  it('renders lesson title for lesson type', () => {
    render(FeedCard, { props: { item: mockItem } });
    expect(screen.getByText('Blues Scale Basics')).toBeInTheDocument();
  });

  it('renders content type badge', () => {
    render(FeedCard, { props: { item: mockItem } });
    expect(screen.getByText('lesson')).toBeInTheDocument();
  });

  it('displays relative timestamp', () => {
    render(FeedCard, { props: { item: mockItem } });
    expect(screen.getByText(/ago/)).toBeInTheDocument();
  });
});

// ── LessonCard Tests ────────────────────────────────────────────────────

describe('LessonCard', () => {
  const mockLesson: Lesson = {
    id: 'l1',
    authorId: 'u1',
    title: 'Fingerpicking 101',
    description: 'Introduction to fingerpicking technique',
    contentUrl: 'https://youtube.com/watch?v=abc',
    difficulty: 'beginner',
    tags: ['fingerpicking', 'acoustic'],
    likes: 42,
    createdAt: Date.now() - 86400000,
  };

  it('renders lesson title', () => {
    render(LessonCard, { props: { lesson: mockLesson } });
    expect(screen.getByText('Fingerpicking 101')).toBeInTheDocument();
  });

  it('renders difficulty chip', () => {
    render(LessonCard, { props: { lesson: mockLesson } });
    expect(screen.getByText('beginner')).toBeInTheDocument();
  });

  it('renders like count', () => {
    render(LessonCard, { props: { lesson: mockLesson } });
    expect(screen.getByText(/42/)).toBeInTheDocument();
  });

  it('fires onLike callback when like button clicked', async () => {
    const onLike = vi.fn();
    render(LessonCard, { props: { lesson: mockLesson, onLike } });
    const likeBtn = screen.getByRole('button', { name: /like/i });
    await fireEvent.click(likeBtn);
    expect(onLike).toHaveBeenCalledWith('l1');
  });
});

// ── ProfileHeader Tests ─────────────────────────────────────────────────

describe('ProfileHeader', () => {
  const mockProfile = {
    id: 'u1',
    username: 'guitar_hero',
    displayName: 'Guitar Hero',
    bio: 'Playing for 10 years',
    gearList: ['fender-strat', 'boss-rt20'],
    streakDays: 15,
    joinedAt: Date.now() - 86400000 * 365,
  };

  it('renders display name', () => {
    render(ProfileHeader, { props: { profile: mockProfile } });
    expect(screen.getByText('Guitar Hero')).toBeInTheDocument();
  });

  it('renders bio', () => {
    render(ProfileHeader, { props: { profile: mockProfile } });
    expect(screen.getByText('Playing for 10 years')).toBeInTheDocument();
  });

  it('renders gear count', () => {
    render(ProfileHeader, { props: { profile: mockProfile } });
    expect(screen.getByText('2')).toBeInTheDocument();
  });

  it('fires onFollow callback when follow button clicked', async () => {
    const onFollow = vi.fn();
    render(ProfileHeader, { props: { profile: mockProfile, onFollow } });
    const followBtn = screen.getByRole('button', { name: /follow/i });
    await fireEvent.click(followBtn);
    expect(onFollow).toHaveBeenCalledWith('u1');
  });
});

// ── StreakCounter Tests ─────────────────────────────────────────────────

describe('StreakCounter', () => {
  const mockStreak = {
    currentStreak: 7,
    longestStreak: 21,
    lastPracticeDate: '2026-06-10',
    calendarHeatmap: {},
  };

  it('renders current streak', () => {
    render(StreakCounter, { props: { streak: mockStreak } });
    expect(screen.getByText('7')).toBeInTheDocument();
  });

  it('renders longest streak', () => {
    render(StreakCounter, { props: { streak: mockStreak } });
    expect(screen.getByText('21')).toBeInTheDocument();
  });

  it('renders streak labels', () => {
    render(StreakCounter, { props: { streak: mockStreak } });
    expect(screen.getByText(/current/i)).toBeInTheDocument();
    expect(screen.getByText(/longest/i)).toBeInTheDocument();
  });
});
