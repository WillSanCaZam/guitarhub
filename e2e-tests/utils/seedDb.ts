import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SEED_DB_PATH = path.join(__dirname, '../fixtures/seed.db');

function getDefaultDbPath(): string {
  const homeDir = process.env.HOME || process.env.USERPROFILE || '';
  return path.join(homeDir, '.local/share/guitarhub/guitarhub.db');
}

export async function seedDatabase(): Promise<void> {
  if (!fs.existsSync(SEED_DB_PATH)) {
    console.warn('Seed DB not found, skipping seed');
    return;
  }

  const targetPath = process.env.GUITARHUB_DB_PATH || getDefaultDbPath();
  const targetDir = path.dirname(targetPath);

  fs.mkdirSync(targetDir, { recursive: true });
  fs.copyFileSync(SEED_DB_PATH, targetPath);
}
