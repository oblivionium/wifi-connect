import { copyFileSync, existsSync, mkdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = dirname(fileURLToPath(import.meta.url));
const desktopDir = resolve(scriptDir, '..');
const repoDir = resolve(desktopDir, '..');
const source = join(repoDir, 'assets', 'wifi.png');
const destination = join(desktopDir, 'dist', 'wifi.png');

if (!existsSync(source)) throw new Error(`Shared logo not found: ${source}`);

mkdirSync(dirname(destination), { recursive: true });
copyFileSync(source, destination);
console.log('Prepared shared Wi-Fi Connect logo for the desktop app');
