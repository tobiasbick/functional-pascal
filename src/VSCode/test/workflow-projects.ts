import assert from 'node:assert/strict';
import { readFile, rm, unlink, writeFile } from 'node:fs/promises';
import path from 'node:path';
import * as vscode from 'vscode';
import { command, eventually, type Fixture } from './support';

export async function projectWorkflows(data: Fixture): Promise<void> {
  await command('selectContext', vscode.Uri.file(data.workflowSolution));
  const tasks = await vscode.tasks.fetchTasks({ type: 'fpas' });
  for (const operation of ['check', 'restore', 'build', 'clean', 'test', 'format'])
    assert(tasks.some(task => task.definition.operation === operation));
  assert.equal(await command('restore'), 0);
  assert.equal(await command('check'), 0);
  assert.equal(await command('clean'), 0);
  const duplicate = path.join(path.dirname(data.engineProject), 'Duplicate.fpas');
  const source = path.join(path.dirname(data.engineProject), 'Engine.fpas');
  await writeFile(duplicate, 'unit Engine; end unit;\n');
  try {
    assert.equal(await command('build'), 1);
    const issue = vscode.languages.getDiagnostics(vscode.Uri.file(source)).find(item => item.code === 'FP2005');
    assert(issue);
    assert(issue.relatedInformation?.some(note => note.location.uri.fsPath === vscode.Uri.file(duplicate).fsPath));
    assert.equal(issue.range.start.line, 0);
    assert.equal(issue.severity, vscode.DiagnosticSeverity.Error);
  } finally { await unlink(duplicate); }

  const hostFile = path.join(path.dirname(data.hostProject), 'Program.cs');
  const host = await readFile(hostFile, 'utf8');
  try {
    await writeFile(hostFile, '#warning Editor workflow warning\n' + host);
    assert.equal(await command('build'), 0);
    assert(vscode.languages.getDiagnostics(vscode.Uri.file(hostFile)).some(item =>
      item.code === 'CS1030' && item.severity === vscode.DiagnosticSeverity.Warning));
  } finally { await writeFile(hostFile, host); }
  assert.equal(await command('build'), 0);
  assert.equal(await command('format'), 0);

  const receipt = path.join(data.root, 'terminal-receipt.json');
  const argument = 'literal ; & ${input} "λ"';
  const before = new Set(vscode.window.terminals);
  const completion = taskExit('run');
  const execution = await command('run', { project: data.hostProject, args: ['input', receipt, argument], cwd: data.root, noBuild: true }) as vscode.TaskExecution;
  assert(execution);
  const terminal = await eventually('integrated FPAS run terminal', () => vscode.window.terminals.find(item => !before.has(item)), value => !!value);
  terminal!.sendText('λ terminal', true);
  assert.equal(await completion, 7);
  const actual = JSON.parse(await readFile(receipt, 'utf8'));
  assert.equal(actual.input, 'λ terminal');
  assert.equal(actual.argument, argument);
  assert.equal(vscode.Uri.file(actual.cwd).fsPath, vscode.Uri.file(data.root).fsPath);
  assert.equal(actual.result, 42);
  assert.equal(actual.configuration, data.hostConfiguration);

  const captured = path.join(data.root, 'captured-receipt.json');
  assert.equal(await command('run', { project: data.hostProject, args: ['input', captured, argument], cwd: data.root, noBuild: true, captured: true }), 7);
  assert.equal(JSON.parse(await readFile(captured, 'utf8')).input, null);

  const waiting = path.join(data.root, 'workflow-wait.pid');
  await Promise.all([rm(waiting, { force: true }), rm(waiting + '.child', { force: true })]);
  const ended = taskExit('run', false);
  const running = await command('run', { project: data.hostProject, args: ['wait', waiting], noBuild: true }) as vscode.TaskExecution;
  assert(running);
  const [pid, child] = await Promise.all([pidFile(waiting), pidFile(waiting + '.child')]);
  running.terminate();
  await ended;
  await waitGone(pid);
  await waitGone(child);
}

function taskExit(operation: string, process = true): Promise<number | undefined> {
  return new Promise((resolve, reject) => {
    const event = process ? vscode.tasks.onDidEndTaskProcess : vscode.tasks.onDidEndTask;
    const subscription = event(result => {
      if (result.execution.task.definition.type !== 'fpas' || result.execution.task.definition.operation !== operation) return;
      subscription.dispose();
      clearTimeout(timer);
      resolve('exitCode' in result && typeof result.exitCode === 'number' ? result.exitCode : undefined);
    });
    const timer = setTimeout(() => { subscription.dispose(); reject(new Error(`FPAS task ${operation} did not finish.`)); }, 40_000);
  });
}

export function pidFile(file: string): Promise<number> {
  return eventually('owned process PID file', async () => Number((await readFile(file, 'utf8')).trim()), pid => Number.isInteger(pid) && pid > 0);
}

export function waitGone(pid: number): Promise<boolean> {
  return eventually('owned process exit', () => {
    try { process.kill(pid, 0); return false; }
    catch (error) { if ((error as NodeJS.ErrnoException).code === 'ESRCH') return true; throw error; }
  }, gone => gone);
}
