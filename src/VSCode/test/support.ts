import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { readFile, rm, symlink } from 'node:fs/promises';
import path from 'node:path';
import { setTimeout } from 'node:timers/promises';
import { promisify } from 'node:util';
import * as vscode from 'vscode';

export const execute = promisify(execFile);
export const command = (name: string, ...args: unknown[]) => vscode.commands.executeCommand(`functionalPascalClr.${name}`, ...args);

export async function eventually<T>(label: string, get: () => T | Promise<T>, accept: (value: T) => boolean): Promise<T> {
  const deadline = Date.now() + 35_000;
  let last: T | undefined;
  let error: unknown;
  while (Date.now() < deadline) {
    try {
      last = await get();
      if (accept(last)) return last;
    } catch (failure) { error = failure; }
    await setTimeout(200);
  }
  throw new Error(`${label} timed out. Last result: ${JSON.stringify(last)}. ${error ?? ''}`);
}

export function errors(uri: vscode.Uri): vscode.Diagnostic[] {
  return vscode.languages.getDiagnostics(uri).filter(diagnostic => diagnostic.severity === vscode.DiagnosticSeverity.Error);
}

export async function lspChildren(): Promise<number[]> {
  let stdout: string;
  if (process.platform === 'win32') {
    ({ stdout } = await execute('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command',
      `Get-CimInstance Win32_Process -Filter 'ParentProcessId = ${process.pid}' | Where-Object { $_.CommandLine -match ' lsp$' } | ForEach-Object { $_.ProcessId }`],
    { windowsHide: true, timeout: 10_000 }));
  } else {
    ({ stdout } = await execute('ps', ['-o', 'pid=,args=', '--ppid', `${process.pid}`], { timeout: 5_000 }));
    stdout = stdout.split('\n').filter(line => / lsp$/.test(line)).map(line => line.trim().split(/\s+/)[0]).join('\n');
  }
  return stdout.trim().split(/\s+/).filter(Boolean).map(Number);
}

export async function editor(file: string): Promise<vscode.TextEditor> {
  return vscode.window.showTextDocument(await vscode.workspace.openTextDocument(vscode.Uri.file(file)), { preview: false });
}

export async function replace(target: vscode.TextEditor, text: string): Promise<void> {
  const document = target.document;
  assert(await target.edit(builder => builder.replace(new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)), text)));
}

export async function compiler(data: Fixture, executable: string): Promise<void> {
  if (process.platform === 'win32') {
    await vscode.workspace.getConfiguration('functionalPascalClr').update('executablePath', executable, vscode.ConfigurationTarget.Global);
  } else {
    // The Remote-WSL host finds this link on its isolated test PATH.
    await rm(data.activeCompiler);
    await symlink(executable, data.activeCompiler);
  }
  await command('restartLanguageServer');
}

export type Fixture = {
  configuration: 'Debug' | 'Release'; compiler: string; activeCompiler: string; mainProject: string; secondProject: string;
  noRestore: string; mismatched: string; standalone: string; imports: string; references: string;
  replacements: string; linked: string; solution: string; owners: string[];
  probeExecutable: string; probeOutput: string; restrictedProject: string; root: string;
  workflowSolution: string; engineProject: string; hostProject: string; hostConfiguration: 'Debug' | 'Release';
  testSolution: string; vstestProject: string; mtpProject: string;
  refactorProject: string; refactorSource: string; refactorDependency: string;
  workflowScenario?: 'projects' | 'testing' | 'debug' | 'refactoring' | 'multiline' | 'all';
};

export async function fixture(): Promise<{ data: Fixture; restricted: boolean }> {
  const folder = vscode.workspace.workspaceFolders?.[0]?.uri;
  assert.equal(folder?.scheme, 'file', 'Expected the workspace host to expose native file URIs.');
  assert(folder);
  const data: Fixture = JSON.parse(await readFile(path.join(folder.fsPath, 'editor-fixture.json'), 'utf8'));
  const comparable = (value: string) => process.platform === 'win32' ? value.toLowerCase() : value;
  return { data, restricted: comparable(path.dirname(data.restrictedProject)) === comparable(folder.fsPath) };
}
