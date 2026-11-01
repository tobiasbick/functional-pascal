import * as vscode from "vscode";
import path from "node:path";
import { pathIdentity } from "../projects/pathIdentity";
import type { LanguageClientController } from "../languageClient";
import type { InspectResponse, Position, SourceLocation } from "./protocol";

export const INSPECT_COMMAND = "functionalPascal.inspect";
const SCHEME = "fpas-inspect";
const STALE = "# Inspection is out of date\n\nSource inputs changed. Run Inspect Declaration again from the source to read the current declaration.\n";

/** Optional explicit selection for commands invoked by another extension or automation. */
export interface InspectArguments {
  readonly uri: vscode.Uri;
  readonly position: Position;
  readonly context?: vscode.Uri;
}

/** Read-only snapshot views never silently retarget an old cursor after source edits. */
export function registerInspect(context: vscode.ExtensionContext, client: LanguageClientController): void {
  const changed = new vscode.EventEmitter<vscode.Uri>();
  const views = new Map<string, { uri: vscode.Uri; text: string; watchers: vscode.Disposable[]; inputs: Set<string>; roots: string[] }>();
  const invalidate = (uri: vscode.Uri, membership = false) => {
    const key = pathIdentity(uri.fsPath);
    for (const view of views.values()) {
      if (!view.inputs.has(key) && !(membership && view.roots.some(root => key.startsWith(`${root}${path.sep}`)))) continue;
      view.text = STALE;
      changed.fire(view.uri);
    }
  };
  const watch = (root: vscode.Uri): vscode.Disposable[] => {
    const watcher = vscode.workspace.createFileSystemWatcher(new vscode.RelativePattern(root, "**/*.{fpas,fpasprj,fpasworkspace}"));
    return [watcher, watcher.onDidCreate(uri => invalidate(uri, true)), watcher.onDidChange(uri => invalidate(uri)), watcher.onDidDelete(uri => invalidate(uri, true))];
  };
  const workspaceWatcher = vscode.workspace.createFileSystemWatcher("**/*.{fpas,fpasprj,fpasworkspace}");
  context.subscriptions.push(
    changed,
    workspaceWatcher,
    workspaceWatcher.onDidCreate(uri => invalidate(uri, true)),
    workspaceWatcher.onDidChange(uri => invalidate(uri)),
    workspaceWatcher.onDidDelete(uri => invalidate(uri, true)),
    vscode.workspace.registerTextDocumentContentProvider(SCHEME, {
      onDidChange: changed.event,
      provideTextDocumentContent: uri => views.get(uri.toString())?.text ?? STALE
    }),
    vscode.workspace.onDidChangeTextDocument(event => {
      if (event.document.uri.scheme === "file") invalidate(event.document.uri);
    }),
    vscode.workspace.onDidCloseTextDocument(document => {
      if (document.uri.scheme === SCHEME) {
        const view = views.get(document.uri.toString());
        view?.watchers.forEach(watcher => watcher.dispose());
        views.delete(document.uri.toString());
      } else if (document.languageId === "fpas") invalidate(document.uri);
    }),
    vscode.window.onDidChangeVisibleTextEditors(editors => {
      const visible = new Set(editors.map(editor => editor.document.uri.toString()));
      for (const [key, view] of views) {
        if (visible.has(key) || view.watchers.length === 0) continue;
        view.watchers.forEach(watcher => watcher.dispose());
        view.watchers = [];
        view.text = STALE;
        changed.fire(view.uri);
      }
    }),
    new vscode.Disposable(() => {
      for (const view of views.values()) view.watchers.forEach(watcher => watcher.dispose());
      views.clear();
    }),
    vscode.commands.registerCommand(INSPECT_COMMAND, async (args?: InspectArguments) => {
      const editor = vscode.window.activeTextEditor;
      const uri = args?.uri ?? editor?.document.uri;
      const position = args?.position ?? editor?.selection.active;
      if (uri?.scheme !== "file" || position === undefined || !uri.path.toLowerCase().endsWith(".fpas")) {
        void vscode.window.showErrorMessage("Select an FPAS declaration or use before inspecting it.");
        return;
      }
      const before = versions();
      try {
        const result = await vscode.window.withProgress({ location: vscode.ProgressLocation.Window, title: "Inspecting FPAS declaration", cancellable: true },
          (_progress, token) => client.inspect({ version: 1, operation: "inspect", context: args?.context?.toString(),
            selector: { kind: "position", uri: uri.toString(), position: { line: position.line, character: position.character } } }, token));
        const viewUri = vscode.Uri.from({ scheme: SCHEME, path: `${uri.path}.inspect.md`, query: `${position.line}:${position.character}` });
        views.get(viewUri.toString())?.watchers.forEach(watcher => watcher.dispose());
        const inventory = result.snapshot.inventory;
        const inputs = new Set([...inventory.files.map(file => file.path), ...inventory.aliases.flatMap(pair => [...pair])].map(value => pathIdentity(value)));
        const roots = [...new Set([result.context.root, result.context.standard_library, ...inventory.globs.map(glob => glob.root)]
          .filter((root): root is string => root !== null).map(value => pathIdentity(value)))];
        const after = versions();
        const stale = [...inputs].some(input => before.get(input) !== after.get(input));
        views.set(viewUri.toString(), { uri: viewUri, text: stale ? STALE : renderInspection(result), inputs, roots,
          watchers: roots.flatMap(root => {
            const location = vscode.Uri.file(root);
            return vscode.workspace.getWorkspaceFolder(location) ? [] : watch(location);
          }) });
        changed.fire(viewUri);
        const document = await vscode.workspace.openTextDocument(viewUri);
        await vscode.window.showTextDocument(document, { viewColumn: vscode.ViewColumn.Beside, preview: true });
        return document;
      } catch (error) {
        void vscode.window.showErrorMessage(`Inspection unavailable: ${error instanceof Error ? error.message : String(error)}`);
        throw error;
      }
    })
  );
}

function versions(): Map<string, number> {
  return new Map(vscode.workspace.textDocuments.filter(document => document.uri.scheme === "file")
    .map(document => [pathIdentity(document.uri.fsPath), document.version]));
}

function renderInspection(result: InspectResponse): string {
  const lines = [`# ${result.target?.qualified_name ?? "Declaration inspection"}`, ""];
  if (result.outcome !== "ok") {
    lines.push(`Analysis: ${result.outcome.replaceAll("_", " ")}.`, "");
    for (const reason of result.coverage.reasons) lines.push(`- ${reason.replaceAll("_", " ")}`);
    for (const diagnostic of result.diagnostics) lines.push(`- ${diagnostic.message}`);
    lines.push("");
  }
  const labels = { signature: "Signature", implementation: "Implementation", supporting_type: "Supporting type" };
  for (const section of result.result.sections) {
    lines.push(`## ${labels[section.kind]}: ${section.qualified_name}`, "",
      `${section.reason.replaceAll("_", " ")} · ${section.origin.replaceAll("_", " ")} · ${section.checked ? "checked" : "analysis incomplete"}`, "");
    if (section.documentation) lines.push(section.documentation, "");
    if (section.excerpts.length === 0 && section.detail) lines.push(code(section.detail), "");
    for (const excerpt of section.excerpts) {
      lines.push(location(excerpt.location), "", code(excerpt.text), "");
    }
    if (section.omitted_bytes) lines.push(`${section.omitted_bytes} of ${section.full_bytes} source bytes omitted.`, "");
  }
  if (result.result.omissions.length) {
    lines.push("## Omitted context", "");
    for (const omission of result.result.omissions) {
      lines.push(`- ${omission.section.replaceAll("_", " ")}: ${omission.reason.replaceAll("_", " ")}${omission.bytes === null ? "" : ` (${omission.bytes} bytes)`}`);
    }
  }
  return `${lines.join("\n")}\n`;
}

function location(value: SourceLocation): string {
  return `${vscode.workspace.asRelativePath(vscode.Uri.parse(value.uri), false)}:${value.range.start.line + 1}:${value.range.start.character + 1} (UTF-16)`;
}
function code(text: string): string {
  const longest = Math.max(2, ...Array.from(text.matchAll(/`+/gu), match => match[0].length));
  const fence = "`".repeat(longest + 1);
  return `${fence}fpas\n${text}\n${fence}`;
}
