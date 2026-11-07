import * as vscode from 'vscode';
import type { ClientSession } from './client';
import { requireTrusted, settings } from './context';

const scheme = 'fpas-refactoring';
const stale = 'The FPAS preview is out of date. Prepare it again.';
const operations = [
  ['addParameter', 'add-parameter', 'Add Parameter', vscode.CodeActionKind.RefactorRewrite],
  ['extractVariable', 'extract-variable', 'Extract Variable', vscode.CodeActionKind.RefactorExtract],
  ['organizeImports', 'organize-imports', 'Organize Imports', vscode.CodeActionKind.SourceOrganizeImports],
] as const;
type Operation = typeof operations[number];
interface Position { line: number; character: number }
interface Change {
  range: { start: Position; end: Position };
  newText: string;
  expectedText: string;
}
interface FilePreview { uri: string; version?: number; edits: Change[] }
interface Plan {
  schemaVersion: number; id: string; kind: string; project: string; snapshotId: string;
  files: FilePreview[]; canApply: boolean; notes: string[]; binaryBoundaries: string[];
}
interface Revalidated {
  schemaVersion: number; id: string; edit: { documentChanges: {
    textDocument: { uri: string; version: number }; edits: Omit<Change, 'expectedText'>[];
  }[] };
}
interface Options { name?: string; type?: string; argument?: string }
interface Preview { plan: Plan; client: ClientSession; uri: vscode.Uri; title: string }

export class Refactorings implements vscode.TextDocumentContentProvider, vscode.Disposable {
  private readonly changed = new vscode.EventEmitter<vscode.Uri>();
  readonly onDidChange = this.changed.event;
  private preview?: Preview;
  private revision = 0;

  constructor(private readonly session: () => ClientSession | undefined) {}

  register(context: vscode.ExtensionContext): void {
    const command = (name: string, action: (options?: Options) => Promise<boolean>) =>
      vscode.commands.registerCommand(settings + '.' + name, async (options?: Options) => {
        try { requireTrusted(); return await action(options); }
        catch (error) {
          if (!(error instanceof vscode.CancellationError)) void vscode.window.showWarningMessage(
            error instanceof Error ? error.message : String(error));
          return false;
        }
      });
    context.subscriptions.push(this, vscode.workspace.registerTextDocumentContentProvider(scheme, this),
      ...operations.map(operation => command(operation[0], options => this.prepare(operation, options))),
      command('applyRefactoring', () => this.apply()),
      command('discardRefactoring', () => this.discard()),
      vscode.workspace.onDidCloseTextDocument(document => {
        if (this.preview?.uri.toString() === document.uri.toString()) void this.discard();
      }),
      vscode.languages.registerCodeActionsProvider({ scheme: 'file', language: 'fpas' }, {
        provideCodeActions: (_, range, context) => vscode.workspace.isTrusted && this.session()
          ? operations.filter((operation, index) => (index !== 1 || !range.isEmpty) &&
            (context.only ? context.only.contains(operation[3]) : index < 2))
            .map(operation => {
              const title = 'FPAS: Preview ' + operation[2];
              const action = new vscode.CodeAction(title, operation[3]);
              action.command = { title, command: settings + '.' + operation[0] };
              return action;
            })
          : [],
      }, { providedCodeActionKinds: operations.map(operation => operation[3]) }));
  }

  invalidate(): void {
    this.revision++;
    const previous = this.preview;
    this.preview = undefined;
    if (previous) this.changed.fire(previous.uri);
  }

  dispose(): void { this.preview = undefined; this.changed.dispose(); }

  provideTextDocumentContent(uri: vscode.Uri): string {
    const current = this.preview;
    if (!current || current.uri.toString() !== uri.toString()) return stale;
    const { plan, title } = current;
    return [title, 'Project: ' + vscode.Uri.parse(plan.project).fsPath,
      'Snapshot: ' + plan.snapshotId, `Read-only binary references: ${plan.binaryBoundaries.length}`, '',
      plan.files.length === 0 ? 'No source changes.' : plan.canApply
        ? 'Review the changes, then run Functional Pascal: Apply Refactoring or Discard Refactoring.'
        : 'Preview only. Open all affected files and prepare the edit again to apply it.',
      ...plan.notes, '',
      ...plan.files.flatMap(file => [vscode.Uri.parse(file.uri).fsPath,
        ...file.edits.map(edit => {
          const line = edit.range.start.line + 1;
          const column = edit.range.start.character + 1;
          return [`  Line ${line}, column ${column}`, ...lines(edit.expectedText, '-'), ...lines(edit.newText, '+')].join('\n');
        }), '']),
    ].join('\n');
  }

  private async prepare(operation: Operation, supplied?: Options): Promise<boolean> {
    const editor = vscode.window.activeTextEditor;
    const client = this.session();
    if (!client || editor?.document.languageId !== 'fpas' || editor.document.uri.scheme !== 'file')
      throw new Error('Open an FPAS source file with a selected analysis context.');
    const document = editor.document;
    const version = document.version;
    const { start, end } = editor.selection;
    const point = operation[1] === 'extract-variable' ? start : editor.selection.active;
    if (operation[1] === 'extract-variable' && editor.selection.isEmpty)
      throw new Error('Select the value expression to extract.');
    const options = supplied ?? await inputs(operation);
    if (!options) return false;
    const revision = ++this.revision;
    const request = {
      schemaVersion: 1, kind: operation[1], textDocument: { uri: document.uri.toString(), version },
      position: wirePosition(point), end: wirePosition(operation[1] === 'extract-variable' ? end : point),
      name: options.name, type: options.type, argument: options.argument,
    };
    const current = () => client === this.session() && revision === this.revision && !document.isClosed && document.version === version;
    const plan = await progress('Checking FPAS refactoring', async token => {
      for (let attempt = 0; ; attempt++) {
        if (token.isCancellationRequested || !current()) throw new vscode.CancellationError();
        try { return await client.request<Plan>('fpas/prepareRefactoring', request, token); }
        catch (error) {
          if (attempt === 2 || (error as { code?: number }).code !== -32801) throw error;
        }
      }
    });
    if (!current() || plan.schemaVersion !== 1 || plan.kind !== operation[1]) throw new Error(stale);
    if (this.preview) this.changed.fire(this.preview.uri);
    const uri = vscode.Uri.from({ scheme, path: '/' + operation[2], query: plan.id });
    this.preview = { plan, client, uri, title: operation[2] };
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(uri),
      { preview: true, viewColumn: vscode.ViewColumn.Beside });
    return true;
  }

  private async apply(): Promise<boolean> {
    const current = this.preview;
    if (!current || current.uri.toString() !== vscode.window.activeTextEditor?.document.uri.toString()) throw new Error(stale);
    if (current.plan.files.length === 0) return false;
    if (!current.plan.canApply) throw new Error('This preview cannot be applied. Open all affected source files and prepare again.');
    const documents = current.plan.files.map(file => {
      const document = vscode.workspace.textDocuments.find(item => sameFile(item.uri, file.uri));
      if (!document || document.version !== file.version) throw new Error(stale);
      return document;
    });
    const { id } = current.plan;
    const result = await progress('Rechecking FPAS refactoring', token => current.client.request<Revalidated>(
      'fpas/applyRefactoring', { schemaVersion: 1, id }, token));
    if (this.preview !== current || current.client !== this.session() || result.schemaVersion !== 1 || result.id !== id ||
      result.edit.documentChanges.length !== documents.length) throw new Error(stale);
    const edit = new vscode.WorkspaceEdit();
    for (const [index, change] of result.edit.documentChanges.entries()) {
      const document = documents[index];
      const file = current.plan.files[index];
      if (document.isClosed || document.version !== file.version || document.version !== change.textDocument.version ||
        !sameFile(document.uri, change.textDocument.uri) || change.edits.length !== file.edits.length) throw new Error(stale);
      for (const [item, replacement] of change.edits.entries()) {
        const range = toRange(replacement.range);
        const expected = file.edits[item];
        if (!range.isEqual(toRange(expected.range)) || replacement.newText !== expected.newText ||
          document.getText(range) !== expected.expectedText) throw new Error(stale);
        edit.replace(document.uri, range, replacement.newText);
      }
    }
    requireTrusted();
    // The VS Code extension host captures open document versions when serializing this text-only edit.
    if (!await vscode.workspace.applyEdit(edit)) throw new Error(stale);
    this.invalidate();
    return true;
  }

  private async discard(): Promise<boolean> {
    const current = this.preview;
    this.invalidate();
    if (!current || current.client !== this.session()) return false;
    const source = new vscode.CancellationTokenSource();
    try { await current.client.request('fpas/discardRefactoring', { schemaVersion: 1, id: current.plan.id }, source.token); }
    catch { /* Session changes already invalidate the retained preview. */ }
    finally { source.dispose(); }
    return true;
  }
}

function wirePosition(point: vscode.Position): Position { return { line: point.line, character: point.character }; }

function toRange(range: Change['range']): vscode.Range {
  return new vscode.Range(range.start.line, range.start.character, range.end.line, range.end.character);
}

function sameFile(uri: vscode.Uri, wire: string): boolean {
  const path = vscode.Uri.parse(wire).fsPath;
  return uri.scheme === 'file' && (process.platform === 'win32' ? uri.fsPath.toLowerCase() === path.toLowerCase() : uri.fsPath === path);
}

function lines(text: string, prefix: string): string[] {
  return text ? text.split(/\r\n|\r|\n/).map(line => '    ' + prefix + ' ' + line) : [];
}

function progress<T>(title: string, action: (token: vscode.CancellationToken) => Thenable<T>): Thenable<T> {
  return vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title, cancellable: true }, (_, token) => action(token));
}

async function inputs(operation: Operation): Promise<Options | undefined> {
  if (operation[1] === 'organize-imports') return {};
  const name = await vscode.window.showInputBox({ prompt: operation[1] === 'extract-variable' ? 'New constant name' : 'New parameter name' });
  if (!name) return undefined;
  if (operation[1] === 'extract-variable') return { name };
  const type = await vscode.window.showInputBox({ prompt: 'FPAS parameter type', placeHolder: 'int' });
  if (!type) return undefined;
  const argument = await vscode.window.showInputBox({ prompt: 'FPAS value expression to add to each call', placeHolder: '0' });
  return argument ? { name, type, argument } : undefined;
}
