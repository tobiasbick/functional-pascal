import path from 'node:path';
import { stat } from 'node:fs/promises';
import * as vscode from 'vscode';

export interface SourceRange {
  path: string; line: number; column: number; length?: number; endLine?: number; endColumn?: number;
}
export interface ReportDiagnostic {
  project?: string; severity: string; code?: string; message: string; location?: SourceRange | null;
  related?: { message: string; location?: SourceRange | null }[];
}
export interface CapturedBuild {
  output: string; error: string; diagnostics?: ReportDiagnostic[]; diagnosticsTruncated?: boolean;
}

export function range(source: SourceRange): vscode.Range {
  const start = new vscode.Position(Math.max(0, source.line - 1), Math.max(0, source.column - 1));
  const end = source.endLine && source.endColumn ? new vscode.Position(source.endLine - 1, source.endColumn - 1) :
    start.translate(0, Math.max(source.length ?? 0, 1));
  return new vscode.Range(start, end.isBefore(start) ? start : end);
}

export class CommandProblems implements vscode.Disposable {
  private readonly collection = vscode.languages.createDiagnosticCollection('fpas-commands');
  private readonly live = new Map<string, Set<string>>();
  private readonly command = new Map<string, { uri: vscode.Uri; diagnostics: vscode.Diagnostic[] }>();
  private readonly subscriptions: vscode.Disposable[];
  private generation = 0;
  private publication = 0;
  private startedAt = 0;

  constructor() {
    const changed = vscode.workspace.createFileSystemWatcher('**/*.{fpas,cs,fs,fpasproj,csproj,fsproj,slnx,props,targets}');
    const change = (uri: vscode.Uri) => {
      if (uri.fsPath.split(path.sep).some(part => ['bin', 'obj', 'node_modules', 'TestResults'].includes(part))) return;
      const publication = this.publication;
      // A queued watcher notification can describe the disk state the command already read.
      void stat(uri.fsPath).catch(() => stat(path.dirname(uri.fsPath))).then(info => {
        const modifiedAt = Math.floor(Math.max(info.mtimeMs, info.ctimeMs));
        if (publication === this.publication && modifiedAt > this.startedAt) this.invalidate();
      }, () => { if (publication === this.publication) this.invalidate(); });
    };
    this.subscriptions = [changed, changed.onDidChange(change), changed.onDidCreate(change), changed.onDidDelete(change),
      vscode.workspace.onDidChangeTextDocument(event => {
        if (event.contentChanges.length && event.document.uri.scheme === 'file') this.invalidate();
      })];
  }

  begin(): { generation: number; publication: number } {
    this.command.clear();
    this.collection.clear();
    this.startedAt = Date.now();
    return { generation: this.generation, publication: ++this.publication };
  }

  publish(stamp: { generation: number; publication: number }, source: string, diagnostics: ReportDiagnostic[], directory: string): void {
    if (stamp.generation !== this.generation || stamp.publication !== this.publication) return;
    for (const report of diagnostics) {
      if (!report.location?.line || !report.location.path) continue;
      const base = report.project && path.isAbsolute(report.project) ? path.dirname(report.project) : directory;
      const uri = vscode.Uri.file(path.resolve(base, report.location.path));
      if (!/\.(?:fpas|cs|fs|fsi)$/i.test(uri.fsPath)) continue;
      if (vscode.workspace.textDocuments.some(document => document.isDirty && document.uri.toString() === uri.toString())) continue;
      const severity = report.severity === 'warning' ? vscode.DiagnosticSeverity.Warning : vscode.DiagnosticSeverity.Error;
      const item = new vscode.Diagnostic(range(report.location), report.message, severity);
      item.code = report.code;
      item.source = 'FPAS ' + source + (report.project ? ' · ' + path.basename(report.project) : '');
      item.relatedInformation = report.related?.flatMap(note => note.location ?
        [new vscode.DiagnosticRelatedInformation(new vscode.Location(vscode.Uri.file(path.resolve(base, note.location.path)), range(note.location)), note.message)] : []);
      const key = uri.toString();
      const group = this.command.get(key) ?? { uri, diagnostics: [] };
      group.diagnostics.push(item);
      this.command.set(key, group);
    }
    for (const key of this.command.keys()) this.refresh(key);
  }

  onLive(uri: vscode.Uri, diagnostics: vscode.Diagnostic[]): void {
    this.live.set(uri.toString(), new Set(diagnostics.map(identity)));
    this.refresh(uri.toString());
  }

  invalidate(contextChanged = false): void {
    this.generation++;
    this.command.clear();
    this.collection.clear();
    if (contextChanged) this.live.clear();
  }

  private refresh(key: string): void {
    const group = this.command.get(key);
    if (!group) return;
    const seen = new Set(this.live.get(key));
    this.collection.set(group.uri, group.diagnostics.filter(item => {
      const id = identity(item);
      if (seen.has(id)) return false;
      seen.add(id);
      return true;
    }));
  }

  dispose(): void {
    for (const subscription of this.subscriptions) subscription.dispose();
    this.collection.dispose();
  }
}

function identity(item: vscode.Diagnostic): string {
  const code = typeof item.code === 'object' ? item.code.value : item.code;
  return JSON.stringify([code, item.severity, item.message, item.range.start.line, item.range.start.character,
    item.range.end.line, item.range.end.character]);
}
