import { describe, it, expect } from 'vitest';
import fs from 'fs';
import path from 'path';

const dashboardPath = path.resolve(__dirname, '../dashboard.css');
const dashboardContent = fs.readFileSync(dashboardPath, 'utf-8');

describe('dashboard.css token compliance', () => {
  it('should not contain hardcoded hex colors', () => {
    // Match hex colors but exclude CSS variable definitions (which contain hex)
    const lines = dashboardContent.split('\n');
    const hardcodedLines = lines.filter(line => {
      // Skip lines that define CSS variables (they contain hex values)
      if (line.includes('--color-') || line.includes('--spacing-') || line.includes('--radius-')) {
        return false;
      }
      // Check for hex colors
      return /#[0-9a-fA-F]{3,6}/.test(line);
    });
    expect(hardcodedLines).toEqual([]);
  });

  it('should use CSS variables for colors', () => {
    expect(dashboardContent).toMatch(/color:\s*var\(--color-/);
  });

  it('should use CSS variables for spacing', () => {
    expect(dashboardContent).toMatch(/margin-top:\s*var\(--spacing-/);
  });
});