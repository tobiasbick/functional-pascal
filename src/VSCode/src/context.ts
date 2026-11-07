import { stat } from 'node:fs/promises';
import path from 'node:path';
import * as vscode from 'vscode';

export const settings = 'functionalPascalClr';
const contextKey = 'analysisContext';
const extensions = new Set(['.fpasproj', '.slnx']);

export function requireTrusted(): void {
  if (!vscode.workspace.isTrusted) {
    throw new Error('Trust the workspace to run the Functional Pascal toolchain.');
  }
}

export class ContextSelection {
  constructor(private readonly state: vscode.Memento) {}

  async current(): Promise<string | undefined> {
    requireTrusted();
    const remembered = this.state.get<string>(contextKey);
    if (remembered && await this.isContext(vscode.Uri.file(remembered))) return remembered;
    const candidates = await this.candidates();
    return candidates.length === 1 ? candidates[0].fsPath : undefined;
  }

  async select(uri?: vscode.Uri): Promise<boolean> {
    requireTrusted();
    if (uri === undefined) {
      const candidates = await this.candidates();
      const choices = candidates.map(candidate => ({
        label: path.basename(candidate.fsPath),
        description: vscode.workspace.asRelativePath(candidate, true),
        uri: candidate,
      }));
      const selected = await vscode.window.showQuickPick([
        ...choices,
        { label: 'Browse for a project or solution…', description: '', uri: undefined },
      ], { placeHolder: 'Select an FPAS project or solution for live analysis' });
      if (!selected) return false;
      uri = selected.uri ?? (await vscode.window.showOpenDialog({
        canSelectMany: false, filters: { 'FPAS context': ['fpasproj', 'slnx'] },
      }))?.[0];
    }
    if (!uri) return false;
    if (!await this.isContext(uri)) {
      throw new Error('Select an existing .fpasproj or .slnx inside a workspace folder.');
    }
    await this.state.update(contextKey, uri.fsPath);
    return true;
  }

  private async candidates(): Promise<vscode.Uri[]> {
    return (await vscode.workspace.findFiles('**/*.{fpasproj,slnx}',
      '**/{bin,obj,node_modules,.git,.temp-data}/**', 257))
      .filter(uri => uri.scheme === 'file' && uri.authority === '')
      .sort((left, right) => left.fsPath.localeCompare(right.fsPath));
  }

  private async isContext(uri: vscode.Uri): Promise<boolean> {
    if (uri.scheme !== 'file' || uri.authority || !vscode.workspace.getWorkspaceFolder(uri) ||
        !extensions.has(path.extname(uri.fsPath).toLowerCase())) return false;
    try { return (await stat(uri.fsPath)).isFile(); }
    catch { return false; }
  }
}
