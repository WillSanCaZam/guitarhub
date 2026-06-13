// SPDX-License-Identifier: GPL-3.0-or-later
//
// Community state — Svelte 5 runes implementation.
// Manages feed items, lessons, riffs, loading states, and pagination.

import { invoke } from '@tauri-apps/api/core';
import type { FeedItem, Lesson, Riff } from '$lib/types/community';

export interface CommunityStore {
  feed: FeedItem[];
  lessons: Lesson[];
  riffs: Riff[];
  loading: boolean;
  error: string | null;
  feedPage: number;
  hasMoreFeed: boolean;
}

/** Reactive community state — access directly in components. */
export const communityState: CommunityStore = $state({
  feed: [],
  lessons: [],
  riffs: [],
  loading: false,
  error: null,
  feedPage: 0,
  hasMoreFeed: true,
});

/** Load feed items from backend (paginated). */
export async function loadFeed(reset = false): Promise<void> {
  communityState.loading = true;
  communityState.error = null;
  try {
    if (reset) {
      communityState.feed = [];
      communityState.feedPage = 0;
      communityState.hasMoreFeed = true;
    }
    const page = communityState.feedPage;
    const items = await invoke<FeedItem[]>('get_feed', {
      limit: 20,
      offset: page * 20,
    });
    communityState.feed = [...communityState.feed, ...items];
    communityState.feedPage = page + 1;
    communityState.hasMoreFeed = items.length === 20;
    communityState.loading = false;
  } catch (e) {
    communityState.loading = false;
    communityState.error = String(e);
  }
}

/** Create a new lesson. */
export async function createLesson(input: {
  title: string;
  description: string;
  contentUrl: string;
  difficulty: 'beginner' | 'intermediate' | 'advanced';
  tags: string[];
}): Promise<string> {
  communityState.loading = true;
  communityState.error = null;
  try {
    const id = await invoke<string>('create_lesson', { input });
    // Add optimistic entry
    communityState.lessons.unshift({
      id,
      authorId: '',
      title: input.title,
      description: input.description,
      contentUrl: input.contentUrl,
      difficulty: input.difficulty,
      tags: input.tags,
      likes: 0,
      createdAt: Date.now(),
    });
    communityState.loading = false;
    return id;
  } catch (e) {
    communityState.loading = false;
    communityState.error = String(e);
    throw e;
  }
}

/** Like a lesson or riff. */
export async function likeContent(contentId: string): Promise<void> {
  try {
    await invoke('like_content', { lessonId: contentId });
    // Update local state optimistically
    const item = communityState.feed.find(f => f.id === contentId);
    if (item && item.itemType === 'lesson') {
      (item.content as Lesson).likes += 1;
    }
  } catch (e) {
    communityState.error = String(e);
  }
}

/** Add a comment to content. */
export async function addComment(
  contentType: string,
  contentId: string,
  text: string
): Promise<string> {
  try {
    return await invoke<string>('add_comment', {
      input: { contentType, contentId, text },
    });
  } catch (e) {
    communityState.error = String(e);
    throw e;
  }
}
