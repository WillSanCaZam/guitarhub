import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import path from 'path';

export default defineConfig({
  plugins: [
    svelte({
      hot: !process.env.VITEST,
    }),
  ],
  resolve: {
    conditions: ['browser'],
    alias: {
      '$lib': path.resolve(import.meta.dirname, './src/lib'),
    },
  },
  test: {
    include: ['src/**/*.{test,spec}.{js,ts}'],
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/setupTests.ts'],
  },
  coverage: {
    provider: 'v8',
    include: ['src/lib'],
  },
});
