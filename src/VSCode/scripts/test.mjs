import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { downloadAndUnzipVSCode, runTests } from '@vscode/test-electron';
import { desktopCli, extensionId, stageHarness } from './installed.mjs';

const [fixtureArg, mode, versionOrExecutable = '1.145.2', vsix] = process.argv.slice(2);
if (!fixtureArg || !['trusted', 'restricted'].includes(mode)) {
  throw new Error('Usage: npm test -- FIXTURE_DIRECTORY trusted|restricted [VS_CODE_VERSION_OR_EXECUTABLE] [VSIX]');
}
const root = path.resolve(fixtureArg);
const restricted = mode === 'restricted';
const workspace = path.join(root, restricted ? 'Restricted' : 'phase1.code-workspace');
const folder = path.join(root, restricted ? 'Restricted' : 'Workspace λ');
const fixture = JSON.parse(await readFile(path.join(folder, 'editor-fixture.json'), 'utf8'));
const profile = await mkdtemp(path.join(root, `profile-${mode}-`));
await mkdir(path.join(profile, 'User'));
await writeFile(path.join(profile, 'User', 'settings.json'), JSON.stringify({
  'functionalPascalClr.executablePath': restricted ? fixture.probeExecutable : fixture.compiler,
  'security.workspace.trust.enabled': restricted,
  'security.workspace.trust.startupPrompt': 'never',
  'security.workspace.trust.banner': 'never',
  'telemetry.telemetryLevel': 'off',
  'extensions.autoUpdate': false,
  'update.mode': 'none',
}));
const extension = fileURLToPath(new URL('..', import.meta.url));
const bundle = path.join(extension, 'out/test/host.cjs');
const extensions = path.join(profile, 'extensions');
const profileArgs = ['--user-data-dir', profile, '--extensions-dir', extensions];
const launchArgs = [workspace, ...profileArgs, ...(vsix ? [] : ['--disable-extensions']),
  '--disable-updates', '--skip-welcome', '--skip-release-notes',
  '--log', 'functional-pascal.functional-pascal-clr:trace'];
const executable = path.isAbsolute(versionOrExecutable) ? versionOrExecutable : await downloadAndUnzipVSCode(versionOrExecutable);
const developmentPath = vsix ? path.join(profile, 'harness') : extension;
const testPath = vsix ? path.join(developmentPath, 'host.cjs') : bundle;
const resultPath = path.join(root, `${mode}-result.json`);
const cli = vsix ? await desktopCli(executable) : undefined;
if (cli) {
  await cli(...profileArgs, '--install-extension', path.resolve(vsix));
  const list = await cli(...profileArgs, '--list-extensions');
  assert(list.split(/\r?\n/).includes(extensionId), 'The VSIX must be installed in the fresh profile.');
  await stageHarness(developmentPath, bundle, extensions);
}
await rm(resultPath, { force: true });
await runHost();
const result = JSON.parse(await readFile(resultPath, 'utf8'));
assert.equal(result.trusted, !restricted);
assert.equal(result.configuration, fixture.configuration);
assert.equal(result.platform, process.platform);
if (!restricted) {
  const pid = Number(await readFile(path.join(root, 'shutdown-pid.txt'), 'utf8'));
  assert.throws(() => process.kill(pid, 0), { code: 'ESRCH' }, 'The extension host must join its language server on shutdown.');
}
if (cli) {
  assert(result.installed, 'The host must load the installed extension.');
  await cli(...profileArgs, '--uninstall-extension', extensionId);
  await stageHarness(developmentPath, bundle, extensions, true);
  const receipt = path.join(root, 'uninstalled-result.json');
  await rm(receipt, { force: true });
  await runHost();
  assert.equal(JSON.parse(await readFile(receipt, 'utf8')).vscode, result.vscode);
  result.uninstalled = true;
}
console.log(JSON.stringify(result));

async function runHost() {
  if (!restricted) {
    await runTests({ vscodeExecutablePath: executable, extensionDevelopmentPath: developmentPath, extensionTestsPath: testPath, launchArgs });
    return;
  }
  // The test-electron launcher always disables workspace trust. Start the restricted host directly.
  const child = spawn(executable, [...launchArgs, `--extensionDevelopmentPath=${developmentPath}`, `--extensionTestsPath=${testPath}`],
    { stdio: 'inherit', windowsHide: true });
  let timer;
  try {
    const exit = await new Promise((resolve, reject) => {
      child.once('error', reject);
      child.once('close', resolve);
      timer = setTimeout(() => { child.kill(); reject(new Error(`Restricted editor did not exit. See ${profile}.`)); }, 120_000);
    });
    assert.equal(exit, 0, `Restricted editor failed. See ${profile}.`);
  } finally { clearTimeout(timer); }
}
