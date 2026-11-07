import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, realpath, rm, symlink, writeFile } from 'node:fs/promises';
import path from 'node:path';
import * as vscode from 'vscode';
import { command, compiler, editor, errors, eventually, execute, fixture, lspChildren, type Fixture } from './support';
import { languageFeatures, watchInputs, linkedOwnership, snippetPrograms, recoverToolchain, multilineStrings } from './scenarios';
import { editorServices } from './editor-services';
import { projectWorkflows } from './workflow-projects';
import { testingWorkflows } from './workflow-testing';
import { debugWorkflows } from './workflow-debug';
import { refactorings } from './refactorings';

export async function run(): Promise<void> {
  const { data, restricted } = await fixture();
  const results: string[] = [];
  const record = (name: string) => { results.push(name); console.log(`PASS ${name}`); };
  assert.equal(process.platform, process.platform === 'win32' ? 'win32' : 'linux');
  assert.equal(vscode.env.remoteName, process.platform === 'linux' ? 'wsl' : undefined);
  assert.equal(vscode.workspace.isTrusted, !restricted);
  const installation = await readFile(path.join(__dirname, 'installation.json'), 'utf8')
    .then(content => JSON.parse(content) as { extensions: string; uninstalled: boolean }, (failure: NodeJS.ErrnoException) => {
      if (failure.code !== 'ENOENT') throw failure;
      return undefined;
    });
  const extension = vscode.extensions.getExtension('functional-pascal.functional-pascal-clr');
  if (installation?.uninstalled) {
    assert.equal(extension, undefined);
    assert.deepEqual(await lspChildren(), []);
    await writeFile(path.join(data.root, 'uninstalled-result.json'), JSON.stringify({ vscode: vscode.version, remote: vscode.env.remoteName ?? null }));
    return;
  }
  assert(extension);
  const installed = installation ? await installedBundle(extension, installation.extensions) : undefined;
  if (process.platform === 'linux') {
    assert.equal(vscode.workspace.getConfiguration('functionalPascalClr').get('executablePath'), '', 'The Remote-WSL test uses its own host PATH.');
    await rm(data.activeCompiler);
    await symlink(restricted ? data.probeExecutable : data.compiler, data.activeCompiler);
    process.env.PATH = `${path.dirname(data.activeCompiler)}${path.delimiter}${process.env.PATH ?? ''}`;
  }
  await extension.activate();

  let environment: unknown;
  if (restricted) {
    if (process.platform === 'win32') {
      assert.equal(vscode.workspace.getConfiguration('functionalPascalClr').get('executablePath'), data.probeExecutable);
    }
    const target = await editor(path.join(path.dirname(data.restrictedProject), 'main.fpas'));
    assert.equal(target.document.languageId, 'fpas');
    // The profile supplies the probe executable as its machine setting. A CLI invocation leaves a durable marker.
    await command('selectContext', vscode.Uri.file(data.restrictedProject));
    await command('selectExecutable');
    await command('selectConfiguration');
    await command('restartLanguageServer');
    await command('build');
    await command('refreshTests');
    await command('organizeImports');
    await assert.rejects(readFile(path.join(data.probeOutput, 'calls.log')), { code: 'ENOENT' });
    assert.deepEqual(await lspChildren(), []);
    record('Restricted Mode keeps static language support and blocks all toolchain commands');
  } else {
    assert.equal(vscode.workspace.workspaceFolders?.length, 2);
    await compiler(data, data.compiler);
    if (installed) {
      const { stdout } = await execute(data.compiler, ['env', '--json'], {
        cwd: path.dirname(data.mainProject), encoding: 'utf8', windowsHide: true, timeout: 35_000,
      }).catch((failure: { stdout?: string }) => {
        if (!failure.stdout) throw failure;
        return { stdout: failure.stdout };
      });
      const report = JSON.parse(stdout);
      assert.equal(report.runtime.os, process.platform === 'win32' ? 'windows' : 'linux');
      assert.equal(report.runtime.architecture, process.arch);
      assert.match(report.dotnet.selectedSdk, /^10\./);
      const project = await readFile(data.mainProject, 'utf8');
      const version = /Sdk="Fpas\.NET\.Sdk\/([^"]+)"/.exec(project)?.[1];
      assert(version);
      const sdk = report.sdkCache.installations.find((entry: { compatibility: string; version: string }) =>
        entry.compatibility === 'matching' && entry.version === version);
      assert(sdk, 'The selected compiler and restored SDK must match.');
      environment = { cli: report.fpas.cli.sha256, compiler: report.fpas.compiler.sha256,
        sdk: sdk.version, dotnet: report.dotnet.selectedSdk, framework: report.runtime.framework };
    }
    assert.deepEqual(await lspChildren(), [], 'Several contexts require explicit selection.');
    if (!data.workflowScenario) {
      await command('selectContext', vscode.Uri.file(data.mainProject));
      let target = await editor(path.join(path.dirname(data.mainProject), 'main.fpas'));
      assert.equal(target.document.languageId, 'fpas');
      target = await languageFeatures(data, target);
      record('Definition, references, workspace symbols, rename, formatting, and versioned unsaved diagnostics');
      target = await editorServices(data, target);
      record('Language services, CLR XML documentation, checked quick fixes and read-only source views');
      await watchInputs(data, target);
      record('Project imports, assemblies, symbols and restore inputs invalidate live analysis');
      await linkedOwnership(data);
      record('Linked ownership and separate roots follow the selected context');
      await snippetPrograms(data, extension.extensionPath);
      record('Editor-rendered program and unit snippets compile through the matching SDK');
      await recoverToolchain(data);
      record('Missing or incompatible inputs and broken initialization allow clean recovery');
      assert(await vscode.workspace.saveAll());
    }
    const scenario = (name: Fixture['workflowScenario']) =>
      data.workflowScenario === undefined || data.workflowScenario === 'all' || data.workflowScenario === name;
    if (scenario('multiline')) {
      await multilineStrings(data);
      record('Multiline strings preserve content and report unterminated unsaved literals');
    }
    if (scenario('projects')) {
      await projectWorkflows(data);
      record('Project commands, mapped mixed-host run, Problems, terminal input and owned cleanup');
    }
    if (scenario('testing')) {
      await testingWorkflows(data);
      record('Real Testing view uses native VSTest and MTP identities, artifacts and cancellation');
    }
    if (scenario('debug')) {
      await debugWorkflows(data);
      record('CLR debugger launches direct and mixed targets, evaluates stops and rejects mismatched input');
    }
    if (scenario('refactoring')) {
      await refactorings(data);
      record('Checked refactorings preview, reject stale and unsafe edits, apply, undo and preserve execution');
    }
    await command('selectContext', vscode.Uri.file(data.mainProject));
    const target = await editor(path.join(path.dirname(data.mainProject), 'main.fpas'));
    await eventually('clean diagnostics after recovery', () => errors(target.document.uri), diagnostics => diagnostics.length === 0);
    const previous = await lspChildren();
    assert.equal(previous.length, 1);
    await command('restartLanguageServer');
    const current = await lspChildren();
    assert.equal(current.length, 1);
    assert.notEqual(current[0], previous[0]);
    assert.throws(() => process.kill(previous[0], 0), { code: 'ESRCH' });
    record('Restart joins the previous LSP process');
    await writeFile(path.join(data.root, 'shutdown-pid.txt'), `${current[0]}\n`);
  }
  await writeFile(path.join(data.root, restricted ? 'restricted-result.json' : 'trusted-result.json'), JSON.stringify({
    configuration: data.configuration, platform: process.platform, remote: vscode.env.remoteName ?? null,
    vscode: vscode.version, node: process.versions.node, trusted: vscode.workspace.isTrusted, installed, environment, results,
  }, null, 2));
}

async function installedBundle(extension: vscode.Extension<unknown>, directory: string) {
  const installedDirectory = await realpath(extension.extensionPath);
  assert.equal(path.relative(await realpath(directory), path.dirname(installedDirectory)), '');
  const main = await readFile(path.join(installedDirectory, extension.packageJSON.main));
  return { version: extension.packageJSON.version, mainSha256: createHash('sha256').update(main).digest('hex') };
}
