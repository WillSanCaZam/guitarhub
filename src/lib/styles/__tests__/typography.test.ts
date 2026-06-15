import { describe, it, expect } from 'vitest';
import fs from 'fs';
import path from 'path';

const typographyPath = path.resolve(__dirname, '../typography.css');
const typographyContent = fs.readFileSync(typographyPath, 'utf-8');

describe('Typography', () => {
  it('should import Syne font', () => {
    expect(typographyContent).toMatch(/family=Syne/);
  });

  it('should import Inter font', () => {
    expect(typographyContent).toMatch(/family=Inter/);
  });

  it('should import JetBrains Mono font', () => {
    expect(typographyContent).toMatch(/family=JetBrains\+Mono/);
  });

  it('should use Syne for display headings', () => {
    expect(typographyContent).toMatch(/\.display-lg\s*\{[^}]*font-family:\s*var\(--font-display\)/);
  });

  it('should use Inter for body text', () => {
    expect(typographyContent).toMatch(/\.body-lg\s*\{[^}]*font-family:\s*var\(--font-body\)/);
  });

  it('should use JetBrains Mono for code', () => {
    expect(typographyContent).toMatch(/\.code-md\s*\{[^}]*font-family:\s*var\(--font-mono\)/);
  });
});