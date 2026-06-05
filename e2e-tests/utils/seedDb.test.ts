import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import * as fs from 'fs';
import * as path from 'path';
import { tmpdir } from 'os';
import { seedDatabase } from './seedDb';

describe('seedDatabase', () => {
  let tempDir: string;
  let originalEnv: string | undefined;

  beforeEach(() => {
    tempDir = fs.mkdtempSync(path.join(tmpdir(), 'seedDb-test-'));
    originalEnv = process.env.GUITARHUB_DB_PATH;
  });

  afterEach(() => {
    fs.rmSync(tempDir, { recursive: true, force: true });
    if (originalEnv === undefined) {
      delete process.env.GUITARHUB_DB_PATH;
    } else {
      process.env.GUITARHUB_DB_PATH = originalEnv;
    }
  });

  it('copies seed.db fixture to the path in GUITARHUB_DB_PATH', async () => {
    const targetPath = path.join(tempDir, 'data', 'guitarhub.db');
    process.env.GUITARHUB_DB_PATH = targetPath;

    await seedDatabase();

    expect(fs.existsSync(targetPath)).toBe(true);
    const sourceContent = fs.readFileSync(
      path.join(__dirname, '../fixtures/seed.db')
    );
    const targetContent = fs.readFileSync(targetPath);
    expect(targetContent).toEqual(sourceContent);
  });

  it('creates missing target directories before copying', async () => {
    const targetPath = path.join(tempDir, 'deep', 'nested', 'guitarhub.db');
    process.env.GUITARHUB_DB_PATH = targetPath;

    await seedDatabase();

    expect(fs.existsSync(targetPath)).toBe(true);
  });

  it('returns early and does not throw when fixture is missing', async () => {
    delete process.env.GUITARHUB_DB_PATH;
    // Temporarily rename the fixture so it appears missing
    const fixturePath = path.join(__dirname, '../fixtures/seed.db');
    const backupPath = fixturePath + '.bak';
    fs.renameSync(fixturePath, backupPath);

    try {
      // Should not throw even though fixture is missing
      await expect(seedDatabase()).resolves.toBeUndefined();
    } finally {
      fs.renameSync(backupPath, fixturePath);
    }
  });

  it('falls back to default path when GUITARHUB_DB_PATH is not set', async () => {
    delete process.env.GUITARHUB_DB_PATH;
    const originalHome = process.env.HOME;
    process.env.HOME = tempDir;

    try {
      await seedDatabase();
      const defaultPath = path.join(tempDir, '.local/share/guitarhub/guitarhub.db');
      expect(fs.existsSync(defaultPath)).toBe(true);
    } finally {
      process.env.HOME = originalHome;
    }
  });
});
