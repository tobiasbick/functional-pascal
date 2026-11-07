import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import * as vscode from 'vscode';
import { command, editor, errors, eventually, replace, type Fixture } from './support';

export async function editorServices(data: Fixture, target: vscode.TextEditor): Promise<vscode.TextEditor> {
  const document = target.document;
  const uri = document.uri;
  const original = document.getText();
  const run = vscode.commands.executeCommand;
  const position = (marker: string, delta = 0) => {
    const offset = document.getText().indexOf(marker);
    assert(offset >= 0, `Missing test marker ${marker}`);
    return document.positionAt(offset + delta);
  };
  await replace(target, original + '\n');
  await eventually('outline of the live buffer', async () => {
    const outline = await run<vscode.DocumentSymbol[]>('vscode.executeDocumentSymbolProvider', uri);
    // VS Code caches an empty outline for the current version if an earlier request was invalidated.
    if (!outline?.length) await replace(target, document.getText() === original ? original + '\n' : original);
    return outline ?? [];
  }, outline => outline.some(item => item.children.some(child => child.name === 'Double')));
  const folds = await run<vscode.FoldingRange[]>('vscode.executeFoldingRangeProvider', uri);
  assert(folds.some(item => item.start <= position('return Value').line && item.end >= position('return Value').line));
  const selected = await run<vscode.SelectionRange[]>('vscode.executeSelectionRangeProvider', uri, [position('Echo(')]);
  assert(selected[0]?.parent);

  const hovers = await run<vscode.Hover[]>('vscode.executeHoverProvider', uri, position('Echo('));
  const contents = hovers.flatMap(item => item.contents).map(item => typeof item === 'string' ? item : item.value).join('\n');
  assert.match(contents, /Echo\(value: string/);
  assert.match(contents, /Returns the generic value/);
  const signature = await run<vscode.SignatureHelp>('vscode.executeSignatureHelpProvider', uri, position(', 2)', 2), ',');
  assert.equal(signature.activeParameter, 1);
  assert(signature.signatures.length >= 2);
  assert.match(signature.signatures[signature.activeSignature].label, /Echo<string>/);
  const tokens = await run<vscode.SemanticTokens>('vscode.provideDocumentSemanticTokens', uri);
  assert(tokens.data.length >= 5);
  const calls = await run<vscode.CallHierarchyItem[]>('vscode.prepareCallHierarchy', uri, position('Double(H.'));
  assert.equal(calls[0]?.name, 'Double');

  await replace(target, original.replace("Api.Echo('text', 2)", "Api.('text', 2)"));
  const complete = await eventually('resolved completion from the CLR receiver', async () =>
    await run<vscode.CompletionList>('vscode.executeCompletionItemProvider', uri, position('Api.(', 4), undefined, 2),
    result => result.items.some(item => item.label === 'Echo' && !!item.documentation));
  const echo = complete.items.find(item => item.label === 'Echo');
  assert(echo?.documentation && typeof echo.documentation !== 'string');
  assert.match(echo.documentation.value, /Returns the generic value/);
  assert.equal(echo.documentation.isTrusted, false);
  assert(!complete.items.some(item => item.label === 'Hidden'));

  await replace(target, original.replace('Double(H.Answer()', 'Double(Math.Abs(1) + H.Answer()'));
  await eventually('missing import diagnostic', () => errors(uri), items => items.some(item => item.message.includes('Math')));
  const at = position('Math.Abs');
  const fixes = await run<vscode.CodeAction[]>('vscode.executeCodeActionProvider', uri, new vscode.Range(at, at), 'quickfix');
  const fix = fixes.find(item => item.title.includes('System.Math'))?.command;
  assert(fix);
  assert.equal(await run(fix.command, ...(fix.arguments ?? [])), true);
  assert.match(document.getText(), /System\.Math as Math/);
  assert.equal(await run(fix.command, ...(fix.arguments ?? [])), false);
  await replace(target, original);
  await eventually('clean diagnostics after correction tests', () => errors(uri), items => items.length === 0);

  target.selection = new vscode.Selection(position('Double(H.'), position('Double(H.'));
  await command('inspectDeclaration');
  const view = vscode.window.activeTextEditor;
  assert.equal(view?.document.uri.scheme, 'fpas-view');
  assert.match(view!.document.getText(), /return Value \* 2;/);
  const inspected = view!.document.getText();
  await run('type', { text: 'ignored' });
  assert.equal(view!.document.getText(), inspected);
  target = await vscode.window.showTextDocument(document);
  await replace(target, original.replace('return Value * 2;', 'return Value * 3;'));
  await eventually('invalidated read-only view', () => view!.document.getText(), value => value.includes('out of date'));
  await replace(target, original);
  assert.equal(await readFile(uri.fsPath, 'utf8'), original);
  await vscode.window.showTextDocument(view!.document);
  await run('workbench.action.closeActiveEditor');
  target = await editor(uri.fsPath);
  const search = command('searchMetadata', 'Fixture.Api.Echo');
  try {
    const metadata = await eventually('selected declaration in the CLR metadata picker', async () => {
      await run('workbench.action.acceptSelectedQuickOpenItem');
      return vscode.window.activeTextEditor?.document;
    }, item => item?.uri.scheme === 'fpas-view');
    assert.match(metadata!.getText(), /Returns the generic value/);
    const xml = path.join(data.references, 'Metadata.xml');
    const originalXml = await readFile(xml, 'utf8');
    try {
      await writeFile(xml, originalXml.replace('Returns the generic value.', 'Updated generic documentation.'));
      await eventually('invalidated CLR declaration view', () => metadata!.getText(), value => value.includes('out of date'));
    } finally {
      await writeFile(xml, originalXml);
    }
  } finally {
    await run('workbench.action.closeQuickOpen');
    await search;
  }
  await run('workbench.action.closeActiveEditor');
  return editor(uri.fsPath);
}
