import { describe, it, expect } from 'vitest';
import fs from 'fs';
import path from 'path';

const tokensPath = path.resolve(__dirname, '../tokens.css');
const tokensContent = fs.readFileSync(tokensPath, 'utf-8');

describe('Design Tokens', () => {
  it('should contain obsidian background color', () => {
    expect(tokensContent).toMatch(/--color-surface:\s*#0D0D0F/);
  });

  it('should contain graphite background color', () => {
    expect(tokensContent).toMatch(/--color-graphite:\s*#1A1A1F/);
  });

  it('should contain amber accent color', () => {
    expect(tokensContent).toMatch(/--color-primary:\s*#F5A623/);
  });

  it('should contain fuzz destructive color', () => {
    expect(tokensContent).toMatch(/--color-error:\s*#E8362A/);
  });

  it('should contain spacing tokens from 1 to 12', () => {
    for (let i = 1; i <= 12; i++) {
      expect(tokensContent).toMatch(new RegExp(`--spacing-${i}:\\s*${i * 4}px`));
    }
  });

  it('should contain radius tokens', () => {
    expect(tokensContent).toMatch(/--radius-sm:/);
    expect(tokensContent).toMatch(/--radius-md:/);
    expect(tokensContent).toMatch(/--radius-lg:/);
    expect(tokensContent).toMatch(/--radius-xl:/);
    expect(tokensContent).toMatch(/--radius-pill:/);
  });

  it('should contain shadow tokens', () => {
    expect(tokensContent).toMatch(/--shadow-card:/);
    expect(tokensContent).toMatch(/--shadow-card-hover:/);
    expect(tokensContent).toMatch(/--shadow-amber-glow:/);
  });

  it('should contain font family tokens', () => {
    expect(tokensContent).toMatch(/--font-display:.*Syne/);
    expect(tokensContent).toMatch(/--font-body:.*Inter/);
    expect(tokensContent).toMatch(/--font-mono:.*JetBrains Mono/);
  });
});