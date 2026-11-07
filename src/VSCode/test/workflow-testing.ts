import assert from 'node:assert/strict';
import { readFile, readdir, rm, stat, unlink, writeFile } from 'node:fs/promises';
import path from 'node:path';
import * as vscode from 'vscode';
import { command, editor, eventually, execute, type Fixture } from './support';
import { pidFile, waitGone } from './workflow-projects';

export async function testingWorkflows(data: Fixture): Promise<void> {
  await command('selectContext', vscode.Uri.file(data.testSolution));
  const results = path.join(path.dirname(data.testSolution), 'TestResults');
  const commands = await vscode.commands.getCommands(true);
  for (const name of ['testing.runAll', 'testing.runAtCursor', 'testing.cancelRun']) assert(commands.includes(name), name);
  await command('refreshTests');
  const whole = await nativeRun(results, 'testing.runAll', 2);
  assert(whole.some(text => text.includes('Vstest.Addition') && text.includes('Vstest.Failure')));
  assert(whole.some(text => text.includes('Native+Cases') && text.includes('Same display name')));
  for (const report of whole) {
    assert.match(report, /outcome="Passed"/);
    assert.match(report, /outcome="Failed"/);
    assert.match(report, /outcome="NotExecuted"/);
  }
  const vstest = path.join(path.dirname(data.vstestProject), 'Vstest.fpas');
  const mtp = path.join(path.dirname(data.mtpProject), 'Native.fpas');
  await place(vstest, 'X.Assert.Equal<int>(Expected');
  const rows = await nativeRun(results, 'testing.runAtCursor', 1);
  assert.equal((rows[0].match(/<UnitTestResult\b/g) ?? []).length, 2);
  assert(!rows[0].includes('Vstest.Failure'));
  await place(mtp, 'T.Assert.AreEqual<int>(4');
  const selected = await nativeRun(results, 'testing.runAtCursor', 1);
  assert.equal((selected[0].match(/<UnitTestResult\b/g) ?? []).length, 1);
  assert(!selected[0].includes('outcome="Failed"'));

  const noMatch = await execute(data.compiler, ['test', data.testSolution, '--configuration', data.configuration, '--project', data.vstestProject,
    '--no-build', '--filter', 'FullyQualifiedName=Absent', '--report', 'json'], { cwd: data.root, encoding: 'utf8', windowsHide: true });
  assert.equal(JSON.parse(noMatch.stdout).projects[0].result.counts.total, 0);
  const output = path.join(path.dirname(data.vstestProject), 'bin', data.configuration, 'net10.0');
  const marker = path.join(output, 'wait-for-cancellation');
  try {
    await rm(path.join(output, 'waiting-test.pid'), { force: true });
    await writeFile(marker, '');
    await place(vstest, 'Base: string');
    const invocation = vscode.commands.executeCommand('testing.runAtCursor');
    const pid = await pidFile(path.join(output, 'waiting-test.pid'));
    await vscode.commands.executeCommand('testing.cancelRun');
    await waitGone(pid);
    await invocation;
  } finally { await unlink(marker); }

  const input = ['test', data.testSolution, '--configuration', data.configuration, '--project', data.vstestProject, '--no-build', '--report', 'json'];
  const discovery = await execute(data.compiler, [...input, '--list'], { cwd: data.root, encoding: 'utf8', windowsHide: true });
  const fingerprint = JSON.parse(discovery.stdout).projects[0].project.outputIdentity;
  const original = await readFile(vstest, 'utf8');
  try {
    await writeFile(vstest, original + '\n');
    const stale = await execute(data.compiler, [...input, '--list'], { cwd: data.root, encoding: 'utf8', windowsHide: true });
    const project = JSON.parse(stale.stdout).projects[0];
    assert.equal(project.project.staleSources, true);
    assert(project.discovery.every((item: { location?: unknown }) => !item.location));
    await assert.rejects(execute(data.compiler, [...input, '--expected-output', fingerprint], { cwd: data.root, encoding: 'utf8', windowsHide: true }),
      (error: unknown) => JSON.parse((error as { stdout: string }).stdout).error.code === 'test_output_changed');
  } finally { await writeFile(vstest, original); }

  assert.equal(await command('clean'), 0);
  const dll = path.join(output, 'Vstest.dll');
  await assert.rejects(stat(dll), { code: 'ENOENT' });
  await command('refreshTests');
  await command('buildTests');
  assert((await stat(dll)).isFile());
}

async function place(file: string, marker: string): Promise<void> {
  const target = await editor(file);
  const offset = target.document.getText().indexOf(marker);
  assert(offset >= 0);
  const point = target.document.positionAt(offset);
  target.selection = new vscode.Selection(point, point);
}

async function nativeRun(root: string, name: string, count: number): Promise<string[]> {
  const before = new Set(await artifacts(root));
  const invocation = vscode.commands.executeCommand(name);
  const files = await eventually(name + ' native TRX', async () => (await artifacts(root)).filter(file => !before.has(file)), values => values.length === count);
  await invocation;
  return await Promise.all(files.map(file => eventually('finished native TRX', () => readFile(file, 'utf8'), text => text.includes('</TestRun>'))));
}

async function artifacts(root: string): Promise<string[]> {
  const names = await readdir(root, { recursive: true }).catch(() => []);
  return names.filter(file => path.basename(file) === 'results.trx').map(file => path.join(root, file));
}
