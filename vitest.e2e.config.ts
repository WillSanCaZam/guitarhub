import { defineConfig } from 'vitest/config';
import path from 'path';

export default defineConfig({
  test: {
    include: ['e2e-tests/utils/**/*.{test,spec}.{js,ts}'],
    environment: 'node',
    globals: true,
  },
});
