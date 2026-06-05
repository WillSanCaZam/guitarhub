import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

// Mock Tauri invoke globally for all component tests
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));
