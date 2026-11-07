import * as vscode from 'vscode';
import type { ClientSession } from './client';
import { requireTrusted, settings } from './context';

const scheme = 'fpas-view';
const stale = 'This view is out of date. Run its Functional Pascal command again to refresh it.';
interface View<T> { schemaVersion: number; snapshotId: string; version: number; revision: number; value: T }
interface Signatures { project: string; complete: boolean; omitted: number; declarations: { signature: string }[] }
interface Slice { availability: string; text?: string }
interface Declaration { declaration: Slice; body: Slice; documentation: Slice }
interface Inspection {
  status: string; reason?: string;
  inspect?: {
    definition: { symbol: { signature: string } }; source: Declaration;
    related: { definition: { symbol: { signature: string } }; source: Declaration }[];
    types: { display: string }[];
    omissions: { sourceBudget: number; relatedLimit: number };
  };
}
interface Metadata {
  entries: { qualifiedName: string; signature: string; assembly: string; documentationId?: string;
    documentation: { availability: string; content?: string } }[];
  coverage: { complete: boolean }; truncated: boolean; totalCount: number;
}

export class SourceViews implements vscode.TextDocumentContentProvider, vscode.Disposable {
  private readonly change = new vscode.EventEmitter<vscode.Uri>();
  private readonly open = new Map<string, string>();
  private generation = 0;
  private serial = 0;
  readonly onDidChange = this.change.event;

  constructor(private readonly session: () => ClientSession | undefined) {}

  register(context: vscode.ExtensionContext): void {
    const command = (name: string, action: (editor: vscode.TextEditor, argument?: string) => Promise<void>) =>
      vscode.commands.registerCommand(`${settings}.${name}`, async (argument?: unknown) => {
        try {
          requireTrusted();
          const editor = vscode.window.activeTextEditor;
          if (editor?.document.languageId !== 'fpas' || editor.document.uri.scheme !== 'file')
            throw new Error('Open an FPAS source file to use this command.');
          await action(editor, typeof argument === 'string' ? argument : undefined);
        } catch (error) {
          if (!(error instanceof vscode.CancellationError))
            void vscode.window.showWarningMessage(error instanceof Error ? error.message : String(error));
        }
      });
    context.subscriptions.push(this, vscode.workspace.registerTextDocumentContentProvider(scheme, this),
      command('showPublicSignatures', editor => this.signatures(editor)),
      command('inspectDeclaration', editor => this.inspect(editor)),
      command('searchMetadata', (editor, query) => this.metadata(editor, query)),
      vscode.workspace.onDidChangeTextDocument(event => {
        if (event.document.uri.scheme === 'file' && event.contentChanges.length !== 0) this.invalidate();
      }),
      vscode.workspace.onDidCloseTextDocument(document => {
        if (document.uri.scheme === scheme) this.open.delete(document.uri.toString());
      }));
  }

  provideTextDocumentContent(uri: vscode.Uri): string { return this.open.get(uri.toString()) ?? stale; }

  invalidate(): void {
    this.generation++;
    for (const key of this.open.keys()) {
      this.open.delete(key);
      this.change.fire(vscode.Uri.parse(key));
    }
  }

  dispose(): void { this.change.dispose(); this.open.clear(); }

  private async signatures(editor: vscode.TextEditor): Promise<void> {
    const view = await this.read<Signatures>(editor, 'fpas/publicSignatures');
    if (!view) return;
    const value = view.value;
    await this.show('Public signatures', view, [`Project: ${value.project}`,
      value.complete ? '' : 'Some source regions could not be checked.',
      value.omitted ? `${value.omitted} additional declarations omitted.` : '',
      ...value.declarations.map(item => item.signature)].filter(Boolean).join('\n'));
  }

  private async inspect(editor: vscode.TextEditor): Promise<void> {
    const point = editor.selection.active;
    const view = await this.read<Inspection>(editor, 'fpas/inspect', { position: { line: point.line, character: point.character } });
    if (!view) return;
    const value = view.value.inspect;
    if (!value) throw new Error(`No checked declaration at the cursor (${view.value.reason ?? view.value.status}).`);
    const parts = (source: Declaration) => [
      source.documentation.text ?? '', source.declaration.text ?? `Declaration: ${source.declaration.availability}`,
      source.body.text ?? `Implementation: ${source.body.availability}`,
    ].filter(Boolean).join('\n\n');
    const types = [...new Set(value.types.map(type => type.display))];
    await this.show('Declaration', view, [value.definition.symbol.signature, parts(value.source),
      types.length ? 'Types\n' + types.join('\n') : '',
      ...value.related.map(item => item.definition.symbol.signature + '\n\n' + parts(item.source)),
      `Omitted by source budget: ${value.omissions.sourceBudget}. Omitted by related type limit: ${value.omissions.relatedLimit}.`].filter(Boolean).join('\n\n'));
  }

  private async metadata(editor: vscode.TextEditor, name?: string): Promise<void> {
    const query = name ?? await vscode.window.showInputBox({ prompt: 'Search referenced CLR metadata',
      validateInput: value => value.trim().length < 1 || value.trim().length > 128 ? 'Enter 1–128 characters.' : undefined });
    if (!query) return;
    const generation = this.generation;
    const view = await this.read<Metadata>(editor, 'fpas/searchMetadata', { query: query.trim() });
    if (!view) return;
    const value = view.value;
    const entries = value.entries.map(entry => ({ label: entry.qualifiedName, detail: entry.signature,
      description: entry.assembly, entry }));
    const selected = await vscode.window.showQuickPick(entries, {
      placeHolder: `${entries.length} of ${value.totalCount} matches${value.coverage.complete ? '' : ' (partial workspace coverage)'}`,
      matchOnDescription: true, matchOnDetail: true,
    });
    if (!selected) return;
    if (generation !== this.generation) throw new Error(stale);
    const entry = selected.entry;
    await this.show(entry.qualifiedName, view, [entry.signature, entry.assembly, entry.documentationId,
      entry.documentation.content ?? `Documentation: ${entry.documentation.availability}`].filter(Boolean).join('\n\n'));
  }

  private async read<T>(editor: vscode.TextEditor, method: string, parameters: object = {}): Promise<View<T> | undefined> {
    const session = this.session();
    if (!session) throw new Error('Start the FPAS language server with a selected analysis context.');
    const generation = this.generation;
    const document = editor.document;
    const version = document.version;
    const result = await vscode.window.withProgress({ location: vscode.ProgressLocation.Notification,
      title: 'Reading FPAS workspace', cancellable: true }, (_, token) =>
      session.request<View<T>>(method, { schemaVersion: 1, textDocument: { uri: document.uri.toString(), version }, ...parameters }, token));
    if (generation !== this.generation || session !== this.session() || document.isClosed || version !== document.version ||
      result.schemaVersion !== 1 || result.version !== version) throw new Error(stale);
    return result;
  }

  private async show(name: string, view: View<unknown>, content: string): Promise<void> {
    if (this.open.size >= 24) this.invalidate();
    const uri = vscode.Uri.from({ scheme, path: '/' + name.replace(/[\\/]/g, ' '), query: String(++this.serial) });
    this.open.set(uri.toString(), `Snapshot ${view.snapshotId}\nDocument version ${view.version}\n\n${content}\n`);
    try {
      await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(uri), { preview: true, viewColumn: vscode.ViewColumn.Beside });
    } catch (error) {
      this.open.delete(uri.toString());
      throw error;
    }
  }
}
