import type { Options } from '@wdio/types';
import { spawn, spawnSync } from 'child_process';
import * as path from 'path';
import * as os from 'os';
import { seedDatabase } from './utils/seedDb';

let tauriDriver: ReturnType<typeof spawn> | undefined;
let exit = false;

function closeTauriDriver(): void {
  exit = true;
  tauriDriver?.kill();
}

function onShutdown(fn: () => void): void {
  const cleanup = () => {
    try {
      fn();
    } finally {
      process.exit();
    }
  };
  process.on('exit', cleanup);
  process.on('SIGINT', cleanup);
  process.on('SIGTERM', cleanup);
  process.on('SIGHUP', cleanup);
  process.on('SIGBREAK', cleanup);
}

onShutdown(() => {
  closeTauriDriver();
});

export const config: Options.Testrunner = {
  runner: 'local',
  framework: 'mocha',
  specs: ['./e2e-tests/specs/**/*.spec.ts'],
  maxInstances: 1,
  hostname: '127.0.0.1',
  port: 4444,
  path: '/',
  capabilities: [
    {
      maxInstances: 1,
      browserName: 'chrome',
      'tauri:options': {
        application: './src-tauri/target/debug/guitarhub',
      },
    },
  ],
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    timeout: 60000,
  },

  onPrepare: () => {
    spawnSync(
      'cargo',
      ['tauri', 'build', '--debug', '--no-bundle'],
      {
        cwd: path.resolve(path.dirname(new URL(import.meta.url).pathname), '..'),
        stdio: 'inherit',
        shell: true,
      }
    );
  },

  beforeSession: async () => {
    await seedDatabase();

    const tauriDriverPath = path.resolve(
      os.homedir(),
      '.cargo',
      'bin',
      'tauri-driver'
    );

    tauriDriver = spawn(tauriDriverPath, [], {
      stdio: [null, process.stdout, process.stderr],
    });

    tauriDriver.on('error', (error) => {
      console.error('tauri-driver error:', error);
      process.exit(1);
    });

    tauriDriver.on('exit', (code) => {
      if (!exit) {
        console.error('tauri-driver exited with code:', code);
        process.exit(1);
      }
    });
  },

  afterSession: () => {
    closeTauriDriver();
  },
};
