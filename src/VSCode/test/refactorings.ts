import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import * as vscode from 'vscode';
import { command, editor, eventually, execute, replace, type Fixture } from './support';

export async function refactorings(data: Fixture): Promise<void> {
  await command('selectContext', vscode.Uri.file(data.refactorProject));
  const original = await readFile(data.refactorSource, 'utf8');
  const dependency = await readFile(data.refactorDependency, 'utf8');
  let source = await editor(data.refactorSource);
  const run = vscode.commands.executeCommand;
  async function select(marker: string, range = false): Promise<void> {
    source = await editor(data.refactorSource);
    const offset = source.document.getText().indexOf(marker);
    assert(offset >= 0, marker);
    source.selection = new vscode.Selection(source.document.positionAt(offset),
      source.document.positionAt(range ? offset + marker.length : offset));
  }
  const preview = () => {
    const active = vscode.window.activeTextEditor;
    assert.equal(active?.document.uri.scheme, 'fpas-refactoring');
    return active!;
  };
  const parameter = { name: 'Added', type: 'int', argument: '99' };
  await select('Compute(Value: int; Extra');
  assert.equal(await command('addParameter', { name: 'Collision', type: 'string', argument: "'x'" }), false);
  assert.equal(source.document.getText(), original);
  await select('Step(7)', true);
  assert.equal(await command('extractVariable', { name: 'Moved' }), false);

  await select('Compute(Value: int; Extra');
  assert.equal(await command('addParameter', parameter), true);
  assert.match(preview().document.getText(), /Added: int; /);
  assert.equal(await command('discardRefactoring'), true);
  assert.equal(await command('applyRefactoring'), false);

  await select('Compute(Value: int; Extra');
  assert.equal(await command('addParameter', parameter), true);
  assert(!vscode.workspace.textDocuments.some(document => document.uri.fsPath === data.refactorDependency));
  try {
    await writeFile(data.refactorDependency, dependency.replace('17', '18'));
    assert.equal(await command('applyRefactoring'), false);
    assert.equal(source.document.getText(), original);
  } finally { await writeFile(data.refactorDependency, dependency); }
  await command('restartLanguageServer');

  const dirty = original.replace("'😀'", "'😀 unsaved'");
  await select('Compute(Value: int; Extra');
  assert.equal(await command('addParameter', parameter), true);
  const applying = command('applyRefactoring');
  await replace(source, dirty);
  assert.equal(await applying, false);
  assert.equal(source.document.getText(), dirty);
  assert.equal(await readFile(data.refactorSource, 'utf8'), original);

  await select('Compute(Value: int; Extra');
  assert.equal(await command('addParameter', parameter), true);
  assert.match(preview().document.getText(), /Extra := /);
  const previewUri = preview().document.uri.toString();
  assert.equal(await command('applyRefactoring'), true);
  assert.match(source.document.getText(), /Compute\(Step\(1\), Extra := Step\(2\), Added := 99\)/);
  assert.match(source.document.getText(), /Compute\(Step\(3\), Extra := Step\(4\), Added := 99\)/);
  assert.equal(await readFile(data.refactorSource, 'utf8'), original);
  const previewTab = vscode.window.tabGroups.all.flatMap(group => group.tabs).find(tab =>
    tab.input instanceof vscode.TabInputText && tab.input.uri.toString() === previewUri);
  assert(previewTab, 'The checked refactoring preview is open.');
  assert(await vscode.window.tabGroups.close(previewTab));
  // In the full workflow, the preceding debugger scenarios can leave the terminal panel open.
  await run('workbench.action.closePanel');
  source = await editor(data.refactorSource);
  assert.equal(vscode.window.activeTextEditor?.document, source.document);
  await run('workbench.action.focusActiveEditorGroup');
  await run('undo');
  await eventually('refactoring undo restores the unsaved buffer', () => source.document.getText(), text => text === dirty);
  await run('redo');
  await eventually('refactoring redo restores the checked edits', () => source.document.getText(), text => text.includes('Added: int'));

  await select('Step(3)', true);
  assert.equal(await command('extractVariable', { name: 'Local' }), true);
  assert.match(preview().document.getText(), /const Local: int := Step\(3\);/);
  assert.equal(await command('applyRefactoring'), true);
  assert.match(source.document.getText(), /Compute\(Local, Extra := Step\(4\), Added := 99\)/);

  await select('ReadRows(2)', true);
  assert.equal(await command('extractVariable', { name: 'CheckedRows' }), true);
  assert.match(preview().document.getText(), /const CheckedRows: array of \(array of string\?\)\? := ReadRows\(2\);/);
  assert.equal(await command('applyRefactoring'), true);
  assert.match(source.document.getText(), /const Rows: array of \(array of string\?\)\? := CheckedRows;/);

  await select('System.Text');
  assert.equal(await command('organizeImports'), true);
  assert.match(preview().document.getText(), /System\.Text as Unused/);
  assert.equal(await command('applyRefactoring'), true);
  const edited = source.document.getText();
  assert(!edited.includes('System.Text'));
  assert(edited.includes('RefactoringDependency as Keep'));
  assert(edited.includes("'😀 unsaved'"));
  assert.equal(await readFile(data.refactorSource, 'utf8'), original);

  source = await editor(data.refactorSource);
  try {
    assert(await source.document.save());
    const result = await execute(data.compiler, ['run', data.refactorProject, '--configuration', data.configuration, '--report', 'json'],
      { cwd: path.dirname(data.refactorProject), encoding: 'utf8', windowsHide: true, timeout: 60_000 });
    assert.equal(JSON.parse(result.stdout).program.output.trim().replaceAll('\r\n', '\n'), '12345\n19');
  } finally {
    await writeFile(data.refactorSource, original);
    await eventually('original refactoring fixture restored', () => source.document.getText(), value => value === original);
  }
}
