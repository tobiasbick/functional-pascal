import assert from 'node:assert/strict';
import { readFile, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import * as vscode from 'vscode';
import { command, editor, eventually, type Fixture } from './support';
import { pidFile, waitGone } from './workflow-projects';

interface Message { type: string; event?: string; command?: string; success?: boolean; message?: string; body?: { threadId?: number; reason?: string } }

export async function debugWorkflows(data: Fixture): Promise<void> {
  const events: Message[] = [];
  let exits = 0;
  const tracker = vscode.debug.registerDebugAdapterTrackerFactory('fpas-clr', {
    createDebugAdapterTracker() {
      return { onDidSendMessage: message => events.push(message), onExit: () => { exits++; } };
    },
  });
  const main = path.join(path.dirname(data.mainProject), 'main.fpas');
  const dll = path.join(path.dirname(data.mainProject), 'bin', data.configuration, 'net10.0', 'EditorProbe.dll');
  const launch = { type: 'fpas-clr', request: 'launch', name: 'FPAS workflow acceptance' };
  try {
    await command('selectContext', vscode.Uri.file(data.mainProject));
    assert.equal(await command('build'), 0);
    await breakpoint(main, 'return Value * 2;');
    assert(await vscode.debug.startDebugging(vscode.workspace.getWorkspaceFolder(vscode.Uri.file(main)), { ...launch, program: dll, stopAtEntry: true }));
    const entry = await stopped(events, 'entry');
    const session = vscode.debug.activeDebugSession;
    assert(session);
    await session.customRequest('continue', { threadId: entry });
    const first = await stopped(events, 'breakpoint');
    await inspect(session, first, main, 'Value');
    await vscode.debug.stopDebugging(session);
    await eventually('FPAS adapter exit', () => exits, count => count === 1);
    vscode.debug.removeBreakpoints(vscode.debug.breakpoints);

    const engine = path.join(path.dirname(data.engineProject), 'Engine.fpas');
    const pid = path.join(data.root, 'debug-host.pid');
    await rm(pid, { force: true });
    await command('selectContext', vscode.Uri.file(data.workflowSolution));
    await breakpoint(engine, 'const Doubled: int');
    assert(await vscode.debug.startDebugging(vscode.workspace.getWorkspaceFolder(vscode.Uri.file(engine)),
      { ...launch, project: data.hostProject, args: ['debug', pid], build: true }));
    const second = await stopped(events, 'breakpoint');
    const mixed = vscode.debug.activeDebugSession;
    assert(mixed);
    await mixed.customRequest('next', { threadId: second });
    const stepped = await stopped(events, 'step');
    await inspect(mixed, stepped, engine, 'Doubled');
    await mixed.customRequest('setExceptionBreakpoints', { filters: ['raised'] });
    await mixed.customRequest('continue', { threadId: stepped });
    const exception = await stopped(events, 'exception');
    const info = await mixed.customRequest('exceptionInfo', { threadId: exception });
    assert.match(JSON.stringify(info), /InvalidOperationException|editor exception/);
    await vscode.debug.stopDebugging(mixed);
    await eventually('mixed adapter exit', () => exits, count => count === 2);
    await waitGone(await pidFile(pid));
    vscode.debug.removeBreakpoints(vscode.debug.breakpoints);

    await command('selectContext', vscode.Uri.file(data.mainProject));
    const originalSource = await readFile(main);
    try {
      await writeFile(main, Buffer.concat([originalSource, Buffer.from('\n')]));
      await rejected('source_mismatch');
    } finally { await writeFile(main, originalSource); }
    const pdb = dll.slice(0, -4) + '.pdb';
    const originalPdb = await readFile(pdb);
    try {
      await writeFile(pdb, await readFile(path.join(path.dirname(data.engineProject), 'bin', data.configuration, 'net10.0', 'Engine.pdb')));
      await rejected('symbols_mismatch');
    } finally { await writeFile(pdb, originalPdb); }
  } finally {
    vscode.debug.removeBreakpoints(vscode.debug.breakpoints);
    await vscode.debug.stopDebugging(vscode.debug.activeDebugSession);
    tracker.dispose();
  }

  async function rejected(expected: string): Promise<void> {
    const from = events.length;
    const before = exits;
    try {
      await vscode.debug.startDebugging(vscode.workspace.getWorkspaceFolder(vscode.Uri.file(main)), { ...launch, program: dll });
    } catch (error) {
      if (!String(error).includes('DialogService: refused to show dialog in tests')) throw error;
    }
    const error = await eventually('DAP launch mismatch', () => events.slice(from).find(message =>
      message.type === 'response' && message.command === 'launch' && !message.success), value => !!value);
    assert.match(JSON.stringify(error), new RegExp(expected));
    await eventually('rejected adapter exit', () => exits, count => count === before + 1);
  }
}

async function breakpoint(file: string, marker: string): Promise<void> {
  const target = await editor(file);
  const offset = target.document.getText().indexOf(marker);
  assert(offset >= 0);
  const line = target.document.positionAt(offset).line;
  vscode.debug.addBreakpoints([new vscode.SourceBreakpoint(new vscode.Location(target.document.uri, new vscode.Position(line, 0)))]);
}

async function stopped(messages: Message[], reason: string): Promise<number> {
  const next = await eventually('DAP ' + reason + ' stop', () =>
    messages.findIndex(message => message.event === 'stopped' && message.body?.reason === reason), index => index >= 0);
  const message = messages.splice(0, next + 1).at(-1);
  assert(message?.body?.threadId);
  return message.body.threadId;
}

async function inspect(session: vscode.DebugSession, threadId: number, file: string, name: string): Promise<void> {
  const threads = await session.customRequest('threads');
  assert(threads.threads.some((thread: { id: number }) => thread.id === threadId));
  const stack = await session.customRequest('stackTrace', { threadId, startFrame: 0, levels: 5 });
  const frame = stack.stackFrames.find((item: { source?: { path?: unknown } }) => sourcePath(item.source?.path) === vscode.Uri.file(file).fsPath);
  assert(frame, 'Expected an FPAS source frame. ' + JSON.stringify(stack.stackFrames.map((item: { name: string; line: number; source?: { path?: unknown } }) =>
    ({ name: item.name, line: item.line, pathType: typeof item.source?.path }))));
  const scopes = await session.customRequest('scopes', { frameId: frame.id });
  assert(scopes.scopes.length);
  const value = await session.customRequest('evaluate', { expression: name, frameId: frame.id, context: 'watch' });
  assert.equal(value.result, '42');
}

function sourcePath(value: unknown): string | undefined {
  if (typeof value === 'string') return /^[a-z-]+:\/\//i.test(value) ? vscode.Uri.parse(value).fsPath : vscode.Uri.file(value).fsPath;
  if (value && typeof value === 'object' && 'scheme' in value && 'path' in value)
    return vscode.Uri.from(value as Parameters<typeof vscode.Uri.from>[0]).fsPath;
  return undefined;
}
