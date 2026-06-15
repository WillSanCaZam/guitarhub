import { describe, it, expect } from 'vitest';
import fs from 'fs';
import path from 'path';

const srcDir = path.resolve(__dirname, '../../..');

function findSvelteFiles(dir: string): string[] {
  const files: string[] = [];
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...findSvelteFiles(fullPath));
    } else if (entry.name.endsWith('.svelte')) {
      files.push(fullPath);
    }
  }
  return files;
}

describe('Svelte files token compliance', () => {
  const svelteFiles = findSvelteFiles(srcDir);

  it('should have no hardcoded hex colors in style blocks', () => {
    const violations: { file: string; line: number; content: string }[] = [];

    for (const file of svelteFiles) {
      const content = fs.readFileSync(file, 'utf-8');
      const lines = content.split('\n');
      let inStyleBlock = false;

      for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        if (line.includes('<style')) {
          inStyleBlock = true;
        } else if (line.includes('</style>')) {
          inStyleBlock = false;
        }

        if (inStyleBlock) {
          // Skip lines that define CSS variables
          if (line.includes('--color-') || line.includes('--spacing-') || line.includes('--radius-') || line.includes('--shadow-') || line.includes('--font-')) {
            continue;
          }
          // Check for hex colors
          const hexMatch = line.match(/#[0-9a-fA-F]{3,6}/);
          if (hexMatch) {
            violations.push({
              file: path.relative(srcDir, file),
              line: i + 1,
              content: line.trim()
            });
          }
        }
      }
    }

    if (violations.length > 0) {
      console.log('Violations:', violations);
    }
    expect(violations).toEqual([]);
  });
});