import { describe, it, expect } from 'vitest';
import fs from 'fs';
import path from 'path';

const typographyPath = path.resolve(__dirname, '../typography.css');
const typographyContent = fs.readFileSync(typographyPath, 'utf-8');

describe('Typography', () => {
  it('should use Space Grotesk for display headings', () => {
    expect(typographyContent).toMatch(/\.display-lg\s*\{[^}]*font-family:\s*var\(--font-display\)/);
  });

  it('should use Inter for body text', () => {
    expect(typographyContent).toMatch(/\.body-lg\s*\{[^}]*font-family:\s*var\(--font-body\)/);
  });

  it('should use JetBrains Mono for code', () => {
    expect(typographyContent).toMatch(/\.code-md\s*\{[^}]*font-family:\s*var\(--font-mono\)/);
  });

  it('should define display typography classes', () => {
    expect(typographyContent).toMatch(/\.display-lg/);
    expect(typographyContent).toMatch(/\.display-md/);
    expect(typographyContent).toMatch(/\.display-sm/);
  });

  it('should define body typography classes', () => {
    expect(typographyContent).toMatch(/\.body-lg/);
    expect(typographyContent).toMatch(/\.body-md/);
    expect(typographyContent).toMatch(/\.body-sm/);
  });

  it('should define code typography classes', () => {
    expect(typographyContent).toMatch(/\.code-md/);
    expect(typographyContent).toMatch(/\.code-sm/);
  });
});
