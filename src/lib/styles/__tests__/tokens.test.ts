import { describe, it, expect } from 'vitest';
import fs from 'fs';
import path from 'path';

const tokensPath = path.resolve(__dirname, '../design-tokens.css');
const tokensContent = fs.readFileSync(tokensPath, 'utf-8');

describe('Design Tokens', () => {
  it('should contain Amp Glow primary color', () => {
    expect(tokensContent).toMatch(/--glow-primary:\s*#FF7A3D/);
  });

  it('should contain void-deep background color', () => {
    expect(tokensContent).toMatch(/--void-deep:\s*#07070C/);
  });

  it('should contain void-mid surface color', () => {
    expect(tokensContent).toMatch(/--void-mid:\s*#0F0F18/);
  });

  it('should contain text-bright color', () => {
    expect(tokensContent).toMatch(/--text-bright:\s*#F5F0EB/);
  });

  it('should contain text-warm color', () => {
    expect(tokensContent).toMatch(/--text-warm:\s*#C4B8A8/);
  });

  it('should contain warning color', () => {
    expect(tokensContent).toMatch(/--warning:\s*#FFB300/);
  });

  it('should contain danger color', () => {
    expect(tokensContent).toMatch(/--danger:\s*#FF1744/);
  });

  it('should contain shadow tokens', () => {
    expect(tokensContent).toMatch(/--shadow-card:/);
    expect(tokensContent).toMatch(/--shadow-hover:/);
    expect(tokensContent).toMatch(/--shadow-glow:/);
  });

  it('should contain font family tokens', () => {
    expect(tokensContent).toMatch(/--font-display:.*Space Grotesk/);
    expect(tokensContent).toMatch(/--font-body:.*Inter/);
    expect(tokensContent).toMatch(/--font-mono:.*JetBrains Mono/);
  });

  it('should contain legacy color aliases mapping to Amp Glow tokens', () => {
    expect(tokensContent).toMatch(/--color-primary:\s*var\(--glow-primary\)/);
    expect(tokensContent).toMatch(/--color-on-primary:\s*var\(--void-deep\)/);
    expect(tokensContent).toMatch(/--color-surface:\s*var\(--void-mid\)/);
    expect(tokensContent).toMatch(/--color-on-surface:\s*var\(--text-bright\)/);
    expect(tokensContent).toMatch(/--color-on-surface-variant:\s*var\(--text-warm\)/);
  });

  it('should not contain Obsidian values', () => {
    expect(tokensContent).not.toMatch(/#0D0D0F/);
    expect(tokensContent).not.toMatch(/#F5A623/);
    expect(tokensContent).not.toMatch(/#1A1A1F/);
  });

  it('should contain radius tokens', () => {
    expect(tokensContent).toMatch(/--radius-sm:/);
    expect(tokensContent).toMatch(/--radius-md:/);
    expect(tokensContent).toMatch(/--radius-lg:/);
    expect(tokensContent).toMatch(/--radius-xl:/);
    expect(tokensContent).toMatch(/--radius-pill:/);
  });

  it('should contain glow effect tokens', () => {
    expect(tokensContent).toMatch(/--glow-soft:/);
    expect(tokensContent).toMatch(/--glow-medium:/);
    expect(tokensContent).toMatch(/--glow-intense:/);
    expect(tokensContent).toMatch(/--glow-text-shadow:/);
  });
});
