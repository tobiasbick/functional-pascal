import * as vscode from "vscode";
import { LanguageClientController } from "./languageClient";

/** The single editor command for the current unit's read-only export view. */
export const PUBLIC_SIGNATURES_COMMAND = "functionalPascal.showPublicSignatures";
const SCHEME = "fpas-public-signatures";

/** Registers source-derived virtual documents and refreshes them after source edits. */
export function registerPublicSignatures(
  context: vscode.ExtensionContext,
  client: LanguageClientController
): void {
  const changed = new vscode.EventEmitter<vscode.Uri>();
  const views = new Map<string, vscode.Uri>();
  const provider: vscode.TextDocumentContentProvider = {
    onDidChange: changed.event,
    async provideTextDocumentContent(uri, token) {
      const source = await vscode.workspace.openTextDocument(vscode.Uri.parse(uri.query));
      try {
        for (let attempt = 0; attempt < 3 && !token.isCancellationRequested; attempt += 1) {
          const result = await client.publicSignatures(source.uri);
          if (result.version === source.version) return result.text;
        }
        return "// Source changed while public signatures were being read. Reopen the view.\n";
      } catch (error) {
        return `// Public signatures unavailable: ${error instanceof Error ? error.message : String(error)}\n`;
      }
    }
  };
  context.subscriptions.push(
    changed,
    vscode.workspace.registerTextDocumentContentProvider(SCHEME, provider),
    vscode.workspace.onDidChangeTextDocument(event => {
      const view = views.get(event.document.uri.toString());
      if (view !== undefined) changed.fire(view);
    }),
    vscode.workspace.onDidCloseTextDocument(document => {
      if (document.uri.scheme === SCHEME) views.delete(document.uri.query);
    }),
    vscode.commands.registerCommand(PUBLIC_SIGNATURES_COMMAND, async () => {
      const source = vscode.window.activeTextEditor?.document;
      if (source?.uri.scheme !== "file" || source.languageId !== "fpas") {
        void vscode.window.showErrorMessage("Open an FPAS unit source before showing public signatures.");
        return;
      }
      const uri = vscode.Uri.from({
        scheme: SCHEME,
        path: `${source.uri.path}.public-signatures`,
        query: source.uri.toString()
      });
      const document = await vscode.workspace.openTextDocument(uri);
      const displayed = await vscode.languages.setTextDocumentLanguage(document, "fpas");
      views.set(source.uri.toString(), uri);
      await vscode.window.showTextDocument(displayed, { viewColumn: vscode.ViewColumn.Beside, preview: true });
      return displayed;
    })
  );
}
