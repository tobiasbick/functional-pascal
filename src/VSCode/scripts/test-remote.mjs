import assert from 'node:assert/strict';
import { execFile, spawn } from 'node:child_process';
import { open, mkdir, mkdtemp, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { setTimeout } from 'node:timers/promises';
import { promisify } from 'node:util';
import { fileURLToPath } from 'node:url';
import { extensionId, stageHarness } from './installed.mjs';

const [code, extensions, distro, extension, fixture, mode, vsix, server] = process.argv.slice(2);
if (process.platform !== 'win32' || !code || !extensions || !distro || ![extension, fixture].every(value => value?.startsWith('/')) ||
    !['trusted', 'restricted'].includes(mode) || (vsix && !server?.startsWith('/'))) {
  throw new Error('Usage on Windows: node scripts/test-remote.mjs CODE_EXE EXTENSIONS_DIR WSL_DISTRO WSL_EXTENSION_DIR WSL_FIXTURE_DIR trusted|restricted [VSIX WSL_CODE_SERVER]');
}
const execute = promisify(execFile);
const wsl = async (...args) => (await execute('wsl.exe', ['-d', distro, '--exec', ...args], { encoding: 'utf8', timeout: 60_000, windowsHide: true })).stdout;
const uri = value => `vscode-remote://wsl+${encodeURIComponent(distro)}${value.split('/').map(encodeURIComponent).join('/')}`;
const restricted = mode === 'restricted';
const localCache = fileURLToPath(new URL('../.vscode-test/', import.meta.url));
await mkdir(localCache, { recursive: true });
const profile = await mkdtemp(path.join(localCache, 'remote-profile-'));
await mkdir(path.join(profile, 'User'));
await writeFile(path.join(profile, 'User', 'settings.json'), JSON.stringify({
  'security.workspace.trust.enabled': restricted,
  'security.workspace.trust.startupPrompt': 'never',
  'security.workspace.trust.banner': 'never',
  'telemetry.telemetryLevel': 'off',
  'extensions.autoUpdate': false,
  'update.mode': 'none',
}));

const harness = path.join(profile, 'harness');
const installedDirectory = vsix ? path.posix.join(server, '../../../../extensions') : undefined;
const bundle = fileURLToPath(new URL('../out/test/host.cjs', import.meta.url));
const remotePath = async value => (await wsl('wslpath', '-a', path.resolve(value))).trim();
const development = vsix ? await remotePath(harness) : extension;
const testPath = vsix ? `${development}/host.cjs` : `${extension}/out/test/host.cjs`;
const receipt = `${fixture}/${mode}-result.json`;
const manage = (...args) => wsl(server, '--extensions-dir', installedDirectory, ...args);
const hasInstalled = async () => (await manage('--list-extensions')).split(/\r?\n/).includes(extensionId);
if (vsix) assert(!await hasInstalled(), 'Choose a WSL test host without an existing FPAS extension.');
try {
  if (vsix) {
    await stageHarness(harness, bundle, installedDirectory);
    await manage('--install-extension', await remotePath(vsix));
  }
  await wsl('rm', '-f', receipt);
  await runHost('workflow');
  const result = JSON.parse(await wsl('cat', receipt));
  assert.equal(result.platform, 'linux');
  assert.equal(result.remote, 'wsl');
  assert.equal(result.trusted, !restricted);
  if (!restricted) {
    const pid = (await wsl('cat', `${fixture}/shutdown-pid.txt`)).trim();
    assert(/^[1-9][0-9]*$/.test(pid));
    const deadline = Date.now() + 10_000;
    while ((await wsl('test', '!', '-d', `/proc/${pid}`).then(() => false, () => true)) && Date.now() < deadline) await setTimeout(200);
    await wsl('test', '!', '-d', `/proc/${pid}`);
  }
  if (vsix) {
    assert(result.installed, 'The remote host must load the installed extension.');
    await manage('--uninstall-extension', extensionId);
    await stageHarness(harness, bundle, installedDirectory, true);
    const uninstallReceipt = `${fixture}/uninstalled-result.json`;
    await wsl('rm', '-f', uninstallReceipt);
    await runHost('uninstall');
    assert.deepEqual(JSON.parse(await wsl('cat', uninstallReceipt)), { vscode: result.vscode, remote: 'wsl' });
    result.uninstalled = true;
  }
  console.log(JSON.stringify(result));
} finally {
  if (vsix && await hasInstalled()) await manage('--uninstall-extension', extensionId);
}

async function runHost(label) {
  const output = await open(path.join(profile, `${label}.stdout.log`), 'w');
  const errors = await open(path.join(profile, `${label}.stderr.log`), 'w');
  const child = spawn(code, ['--new-window', '--skip-welcome', '--skip-release-notes', '--disable-updates',
    '--user-data-dir', profile, '--extensions-dir', extensions,
    `--extensionDevelopmentPath=${uri(development)}`, `--extensionTestsPath=${uri(testPath)}`,
    restricted ? `--folder-uri=${uri(`${fixture}/Restricted`)}` : `--file-uri=${uri(`${fixture}/phase1.code-workspace`)}`,
    ...(restricted ? [] : ['--disable-workspace-trust']),
  ], { windowsHide: true, stdio: ['ignore', output.fd, errors.fd] });
  let timeout;
  try {
    const exit = await new Promise((resolve, reject) => {
      child.once('error', reject);
      child.once('close', resolve);
      timeout = globalThis.setTimeout(() => {
        child.kill();
        reject(new Error(`Remote-WSL test timed out. See ${profile}.`));
      }, 600_000);
    });
    assert.equal(exit, 0, `VS Code failed. See ${profile}.`);
  } finally {
    clearTimeout(timeout);
    await Promise.all([output.close(), errors.close()]);
  }
}
