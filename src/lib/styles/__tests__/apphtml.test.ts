import { describe, it, expect } from 'vitest';
import fs from 'fs';
import path from 'path';

const appHtmlPath = path.resolve(__dirname, '../../../app.html');
const appHtmlContent = fs.readFileSync(appHtmlPath, 'utf-8');

describe('app.html font loading', () => {
  it('should have preconnect to Google Fonts', () => {
    expect(appHtmlContent).toMatch(/<link[^>]*rel="preconnect"[^>]*href="https:\/\/fonts\.gstatic\.com"/);
  });

  it('should load Space Grotesk font', () => {
    expect(appHtmlContent).toMatch(/family=Space\+Grotesk/);
  });

  it('should load Inter font', () => {
    expect(appHtmlContent).toMatch(/family=Inter/);
  });

  it('should load JetBrains Mono font', () => {
    expect(appHtmlContent).toMatch(/family=JetBrains\+Mono/);
  });

  it('should load Caveat font', () => {
    expect(appHtmlContent).toMatch(/family=Caveat/);
  });

  it('should have font-display=swap', () => {
    expect(appHtmlContent).toMatch(/display=swap/);
  });
});
