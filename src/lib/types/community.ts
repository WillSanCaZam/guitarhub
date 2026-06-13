// SPDX-License-Identifier: GPL-3.0-or-later
//
// Community Hub — TypeScript interfaces for community entities.

export interface UserProfile {
  id: string
  username: string
  displayName: string
  avatarUrl?: string
  bio?: string
  gearList: string[]
  streakDays: number
  joinedAt: number
}

export interface Lesson {
  id: string
  authorId: string
  title: string
  description: string
  contentUrl: string
  difficulty: 'beginner' | 'intermediate' | 'advanced'
  tags: string[]
  likes: number
  createdAt: number
}

export interface Riff {
  id: string
  authorId: string
  title: string
  tablature: string
  bpm: number
  tuning: string
  tags: string[]
  likes: number
  createdAt: number
}

export interface FeedItem {
  id: string
  authorId: string
  authorName: string
  authorAvatar?: string
  itemType: 'lesson' | 'riff' | 'comment'
  content: Lesson | Riff | Comment
  createdAt: number
}

export interface Comment {
  id: string
  authorId: string
  authorName: string
  authorAvatar?: string
  content: string
  createdAt: number
}

export interface Streak {
  currentStreak: number
  longestStreak: number
  lastPracticeDate: string
  calendarHeatmap: Record<string, number>
}

export interface Follow {
  followerId: string
  followingId: string
  createdAt: number
}
