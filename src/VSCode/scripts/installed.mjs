import { execFile } from 'node:child_process';
import { copyFile, mkdir, readdir, stat, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { promisify } from 'node:util';

export const extensionId = 'functional-pascal.functional-pascal-clr';

export async function stageHarness(directory, bundle, extensions, uninstalled = false) {
  await mkdir(directory, { recursive: true });
  await copyFile(bundle, path.join(directory, 'host.cjs'));
  await writeFile(path.join(directory, 'package.json'), JSON.stringify({
    publisher: 'fpas-tests', name: 'installed-harness', version: '0.0.0', engines: { vscode: '^1.145.0' },
  }));
  await writeFile(path.join(directory, 'installation.json'), JSON.stringify({ extensions, uninstalled }));
}

export async function desktopCli(executable) {
  const root = path.dirname(executable);
  const relative = path.join('resources', 'app', 'out', 'cli.js');
  const candidates = [path.join(root, relative), ...(await readdir(root, { withFileTypes: true }))
    .filter(entry => entry.isDirectory()).map(entry => path.join(root, entry.name, relative))];
  for (const candidate of candidates) {
    if (!(await stat(candidate).catch(() => undefined))?.isFile()) continue;
    return async (...args) => (await promisify(execFile)(executable, [candidate, ...args], {
      env: { ...process.env, ELECTRON_RUN_AS_NODE: '1', VSCODE_DEV: '' },
      encoding: 'utf8', timeout: 60_000, windowsHide: true,
    })).stdout;
  }
  throw new Error(`Cannot find the VS Code CLI next to ${executable}.`);
}
