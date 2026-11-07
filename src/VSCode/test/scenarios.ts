import assert from 'node:assert/strict';
import { cp, readFile, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import * as vscode from 'vscode';
import { command, compiler, editor, errors, eventually, execute, lspChildren, replace, type Fixture } from './support';

function position(document: vscode.TextDocument, needle: string, last = false): vscode.Position {
  const text = document.getText();
  const offset = last ? text.lastIndexOf(needle) : text.indexOf(needle);
  assert(offset >= 0, `Missing fixture token ${needle}`);
  return document.positionAt(offset);
}

async function definitions(document: vscode.TextDocument, needle: string, last = false): Promise<vscode.Location[]> {
  return await vscode.commands.executeCommand<vscode.Location[]>('vscode.executeDefinitionProvider', document.uri, position(document, needle, last)) ?? [];
}

async function symbols(name: string): Promise<vscode.SymbolInformation[]> {
  return await vscode.commands.executeCommand<vscode.SymbolInformation[]>('vscode.executeWorkspaceSymbolProvider', name) ?? [];
}

export async function multilineStrings(data: Fixture): Promise<void> {
  await command('selectContext', vscode.Uri.file(data.mainProject));
  const target = await editor(path.join(path.dirname(data.mainProject), 'main.fpas'));
  const document = target.document;
  const original = document.getText();
  const diskSource = await readFile(document.uri.fsPath, 'utf8');
  const unfinished = [
    'program EditorProbe;',
    'begin',
    " const Text:string:='first",
    '\t  // part of the literal  ',
    "last''line",
  ].join('\n');
  try {
    await replace(target, unfinished);
    const error = await eventually('unterminated string diagnostic', () =>
      errors(document.uri).find(diagnostic => diagnostic.code === 'FP1002'), diagnostic => diagnostic !== undefined);
    assert(error);
    assert.deepEqual(error.range.start, position(document, "'first"));

    await replace(target, unfinished + "';\n discard Text;\nend;\nend program;\n");
    await eventually('closing the string clears the diagnostic', () => errors(document.uri), diagnostics => diagnostics.length === 0);
    const edits = await vscode.commands.executeCommand<vscode.TextEdit[]>(
      'vscode.executeFormatDocumentProvider', document.uri, { tabSize: 2, insertSpaces: true });
    assert(edits?.length);
    const formatting = new vscode.WorkspaceEdit();
    formatting.set(document.uri, edits);
    assert(await vscode.workspace.applyEdit(formatting));
    assert(document.getText().replaceAll('\r\n', '\n').includes(
      "  const Text: string := 'first\n\t  // part of the literal  \nlast''line';\n  discard Text;"));
    assert.equal(await readFile(document.uri.fsPath, 'utf8'), diskSource);

    await replace(target, unfinished);
    await eventually('reopening the string restores its diagnostic', () => errors(document.uri),
      diagnostics => diagnostics.some(diagnostic => diagnostic.code === 'FP1002'));
  } finally {
    await replace(target, original);
  }
}

export async function languageFeatures(data: Fixture, target: vscode.TextEditor): Promise<vscode.TextEditor> {
  const document = target.document;
  const original = document.getText();
  const definition = await eventually('definition in real LSP', () => definitions(document, 'Double', true), result => result.length === 1);
  assert.equal(definition[0].uri.fsPath, document.uri.fsPath);
  assert(definition[0].range.start.isEqual(position(document, 'Double')));

  const call = position(document, 'Double', true);
  const references = await vscode.commands.executeCommand<vscode.Location[]>('vscode.executeReferenceProvider', document.uri, position(document, 'Double'));
  assert(references.some(location => location.range.start.isEqual(call)));
  const marker = `${data.configuration}Marker`;
  await eventually('configured symbol', () => symbols(marker), result => result.some(symbol => symbol.name === marker));
  assert.equal((await symbols(data.configuration === 'Debug' ? 'ReleaseMarker' : 'DebugMarker')).length, 0);

  const rename = await vscode.commands.executeCommand<vscode.WorkspaceEdit>('vscode.executeDocumentRenameProvider', document.uri, call, 'Twice');
  assert(rename && await vscode.workspace.applyEdit(rename));
  assert.match(document.getText(), /function Twice\(/);
  assert.match(document.getText(), /Twice\(H\.Answer\(/);
  assert.equal(await readFile(document.uri.fsPath, 'utf8'), original);

  const unformatted = original.replace('  return Value * 2;', 'return   Value*2;');
  await replace(target, unformatted);
  const format = await vscode.commands.executeCommand<vscode.TextEdit[]>('vscode.executeFormatDocumentProvider', document.uri, { tabSize: 2, insertSpaces: true });
  assert(format.length > 0);
  assert(await document.save());
  assert.match(await readFile(document.uri.fsPath, 'utf8'), /  return Value \* 2;/);

  const saved = document.getText();
  await replace(target, saved.replace('Double(H.Answer()', 'Missing(H.Answer()'));
  const broken = await eventually('unsaved diagnostics', () => errors(document.uri), diagnostics => diagnostics.some(item => item.message.includes('Missing')));
  assert(broken.some(item => item.range.start.line === position(document, 'Missing').line));
  assert.equal(await readFile(document.uri.fsPath, 'utf8'), saved);
  await replace(target, saved);
  await eventually('repaired buffer', () => errors(document.uri), diagnostics => diagnostics.length === 0);
  await replace(target, saved.replace('Double(H.Answer()', 'Missing(H.Answer()'));
  await eventually('second unsaved diagnostic', () => errors(document.uri), diagnostics => diagnostics.length > 0);
  await vscode.commands.executeCommand('workbench.action.revertAndCloseActiveEditor');
  await eventually('closed buffer diagnostics', () => errors(document.uri), diagnostics => diagnostics.length === 0);
  assert.equal(await readFile(document.uri.fsPath, 'utf8'), saved);
  const reopened = await editor(document.uri.fsPath);
  await eventually('definition after reopening', () => definitions(reopened.document, 'Double', true), result => result.length === 1);
  return reopened;
}

export async function watchInputs(data: Fixture, target: vscode.TextEditor): Promise<void> {
  const uri = target.document.uri;
  const originalImports = await readFile(data.imports);
  try {
    await writeFile(data.imports, '<Project />\n');
    await eventually('imported props diagnostics without requesting analysis', () => errors(uri), items => items.length > 0);
  } finally { await writeFile(data.imports, originalImports); }
  await eventually('restored import diagnostics', () => errors(uri), items => items.length === 0);

  const dll = path.join(data.references, 'Metadata.dll');
  const pdb = path.join(data.references, 'Metadata.pdb');
  const [originalDll, originalPdb] = await Promise.all([readFile(dll), readFile(pdb)]);
  try {
    await cp(path.join(data.replacements, 'Metadata.pdb'), pdb);
    await cp(path.join(data.replacements, 'Metadata.dll'), dll);
    const call = position(target.document, 'Api.Value');
    await eventually('referenced assembly diagnostics without requesting analysis', () => errors(uri),
      items => items.some(item => item.range.start.line === call.line));
  } finally {
    await writeFile(dll, originalDll);
    await writeFile(pdb, originalPdb);
  }
  await eventually('restored reference diagnostics', () => errors(uri), items => items.length === 0);

  const assets = path.join(path.dirname(data.mainProject), 'obj', 'project.assets.json');
  const originalAssets = await readFile(assets);
  try {
    await rm(assets);
    await eventually('missing restore assets', () => definitions(target.document, 'Double', true), result => result.length === 0);
  } finally { await writeFile(assets, originalAssets); }
  await eventually('restored assets', () => definitions(target.document, 'Double', true), result => result.length === 1);
}

export async function linkedOwnership(data: Fixture): Promise<void> {
  await command('selectContext', vscode.Uri.file(data.solution));
  const linked = await editor(path.join(data.linked, 'Shared.fpas'));
  assert.equal((await definitions(linked.document, 'Answer')).length, 0);
  await command('selectContext', vscode.Uri.file(data.owners[0]));
  await eventually('explicit linked owner', () => definitions(linked.document, 'Answer'), items => items.length === 1);
  await command('selectContext', vscode.Uri.file(data.secondProject));
  await eventually('second workspace root', () => symbols('SecondProbe'), items => items.some(item => item.name === 'SecondProbe'));
  assert.equal((await symbols('Double')).length, 0);
}

export async function snippetPrograms(data: Fixture, extensionPath: string): Promise<void> {
  const snippets: Record<string, { body: string[] }> = JSON.parse(await readFile(path.join(extensionPath, 'snippets/fpas.json'), 'utf8'));
  const directory = path.dirname(data.standalone);
  for (const [name, file] of [['Program', 'main.fpas'], ['Unit', 'Utilities.fpas']]) {
    const source = path.join(directory, file);
    await writeFile(source, '');
    const target = await editor(source);
    assert(await target.insertSnippet(new vscode.SnippetString(snippets[name].body.join('\n'))));
    assert(await target.document.save());
  }
  await execute(data.compiler, ['build', data.standalone, '--no-restore', '--configuration', data.configuration], {
    cwd: directory, timeout: 45_000, windowsHide: true,
  });
}

export async function recoverToolchain(data: Fixture): Promise<void> {
  for (const project of [data.noRestore, data.mismatched]) {
    await command('selectContext', vscode.Uri.file(project));
    const target = await editor(path.join(path.dirname(project), 'main.fpas'));
    assert.equal((await definitions(target.document, 'Console', true)).length, 0);
    assert.equal((await symbols(path.basename(project, '.fpasproj'))).length, 0);
  }
  await command('selectContext', vscode.Uri.file(data.mainProject));
  await compiler(data, path.join(data.root, process.platform === 'win32' ? 'missing.exe' : 'missing'));
  assert.deepEqual(await lspChildren(), []);
  const calls = path.join(data.probeOutput, 'calls.log');
  try {
    await compiler(data, data.probeExecutable);
    const invocations = (await readFile(calls, 'utf8')).trim().split(/\s+/);
    assert(invocations.includes('env') && invocations.includes('lsp'));
    assert.deepEqual(await lspChildren(), []);
  } finally { await rm(calls, { force: true }); }
  await compiler(data, data.compiler);
  const target = await editor(path.join(path.dirname(data.mainProject), 'main.fpas'));
  await eventually('working compiler after recovery', () => definitions(target.document, 'Double', true), items => items.length === 1);
}
