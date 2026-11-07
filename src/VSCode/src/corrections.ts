import * as vscode from 'vscode';
import type { ClientSession } from './client';
import { requireTrusted } from './context';

interface Point { line: number; character: number }
interface ResolvedCorrection {
  edit: { documentChanges: {
    textDocument: { uri: string; version: number };
    edits: { range: { start: Point; end: Point }; newText: string }[];
  }[] };
}

export async function applyCorrection(request: object, session: () => ClientSession | undefined): Promise<boolean> {
  requireTrusted();
  const client = session();
  const editor = vscode.window.activeTextEditor;
  if (!client || !editor || editor.document.languageId !== 'fpas') throw new Error('Open the source of the FPAS correction.');
  const result = await vscode.window.withProgress({
    location: vscode.ProgressLocation.Notification, title: 'Checking FPAS correction', cancellable: true,
  }, (_, token) => client.request<ResolvedCorrection>('codeAction/resolve', request, token));
  const changes = result.edit.documentChanges;
  const document = editor.document;
  const change = changes[0];
  const comparable = (uri: vscode.Uri) => process.platform === 'win32' ? uri.fsPath.toLowerCase() : uri.fsPath;
  if (client !== session() || changes.length !== 1 || document.isClosed || change.textDocument.version !== document.version ||
    comparable(vscode.Uri.parse(change.textDocument.uri)) !== comparable(document.uri))
    throw new Error('The FPAS correction is out of date.');
  // TextEditor.edit carries the captured buffer version through the extension-host boundary.
  if (!await editor.edit(builder => {
    for (const edit of change.edits) {
      const { start, end } = edit.range;
      builder.replace(new vscode.Range(start.line, start.character, end.line, end.character), edit.newText);
    }
  })) throw new Error('The FPAS correction is out of date.');
  return true;
}
