import { chmodSync, copyFileSync, mkdirSync, existsSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const scriptDir = dirname(fileURLToPath(import.meta.url));
const desktopDir = resolve(scriptDir, '..');
const repoDir = resolve(desktopDir, '..');
const tauriDir = join(desktopDir, 'src-tauri');

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { stdio: 'inherit', ...options });
  if (result.status !== 0) {
    throw result.error ?? new Error(`${command} exited with status ${result.status}`);
  }
}

const rustc = process.platform === 'win32' ? 'rustc.exe' : 'rustc';
const version = spawnSync(rustc, ['-vV'], { encoding: 'utf8' });
if (version.status !== 0) throw new Error('Could not determine the Rust target triple');
const target = version.stdout.match(/^host:\s*(\S+)$/m)?.[1];
if (!target) throw new Error('rustc -vV did not report a host target triple');

run('cargo', [
  'build', '--locked', '--release', '--target', target,
  '--manifest-path', join(repoDir, 'Cargo.toml'),
]);

const executable = process.platform === 'win32' ? 'wifi.exe' : 'wifi';
const builtBinary = join(repoDir, 'target', target, 'release', executable);
if (!existsSync(builtBinary)) throw new Error(`CLI build output not found: ${builtBinary}`);

const binaryDir = join(tauriDir, 'binaries');
mkdirSync(binaryDir, { recursive: true });
const sidecarName = `wifi-${target}${process.platform === 'win32' ? '.exe' : ''}`;
const sidecarPath = join(binaryDir, sidecarName);
const genericPath = join(binaryDir, executable);
copyFileSync(builtBinary, sidecarPath);
copyFileSync(builtBinary, genericPath);
if (process.platform !== 'win32') {
  chmodSync(sidecarPath, 0o755);
  chmodSync(genericPath, 0o755);
}

console.log(`Prepared ${sidecarName} for Tauri packaging`);
