import * as vscode from "vscode";
import type { LanguageClientController } from "../languageClient";
import type { Position } from "./protocol";

export const ADD_PARAMETER_COMMAND = "functionalPascal.addParameter";
const SCHEME = "fpas-refactor";

/** Complete arguments run without prompts; omitted values are collected interactively. */
export interface AddParameterArguments {
  readonly uri?: vscode.Uri;
  readonly position?: Position;
  readonly symbol?: string;
  readonly context?: vscode.Uri;
  readonly name?: string;
  readonly type?: string;
  readonly mode?: "value" | "var";
  readonly argument?: string;
  /** Show the prepared change without applying it. */
  readonly previewOnly?: boolean;
}

export interface ParameterRequest {
  readonly version: 1;
  readonly phase: "prepare";
  readonly context?: string;
  readonly recipe: {
    readonly action: "add-parameter";
    readonly selector: { readonly kind: "position"; readonly uri: string; readonly position: Position }
      | { readonly kind: "symbol"; readonly name: string };
    readonly name: string;
    readonly type: string;
    readonly mode: "value" | "var";
    readonly argument: string;
  };
}

export interface RefactorResponse {
  readonly version: 1;
  readonly status: "prepared" | "no_change" | "unsupported" | "incomplete" | "already_applied" | "applied";
  readonly handle: string | null;
  readonly edit: { readonly documentChanges: readonly {
    readonly textDocument: { readonly uri: string; readonly version: number | null };
    readonly edits: readonly { readonly range: { readonly start: Position; readonly end: Position }; readonly newText: string }[];
  }[] } | null;
  readonly blockers: readonly { readonly code: string; readonly message: string; readonly path: string | null; readonly start: number | null }[];
  readonly diagnostics: readonly { readonly message: string }[];
  readonly limitations: readonly string[];
}

/** Uses the existing synchronized LSP session and its versioned editor application route. */
export function registerParameter(context: vscode.ExtensionContext, client: LanguageClientController): void {
  const views = new Map<string, string>();
  let next = 0;
  context.subscriptions.push(
    vscode.workspace.registerTextDocumentContentProvider(SCHEME, {
      provideTextDocumentContent: uri => views.get(uri.toString()) ?? "This refactoring preview has expired."
    }),
    vscode.workspace.onDidCloseTextDocument(document => {
      if (document.uri.scheme === SCHEME) views.delete(document.uri.toString());
    }),
    new vscode.Disposable(() => views.clear()),
    vscode.commands.registerCommand(ADD_PARAMETER_COMMAND, async (args: AddParameterArguments = {}) => {
      try {
        const editor = vscode.window.activeTextEditor;
        const uri = args.uri ?? editor?.document.uri;
        const position = args.position ?? editor?.selection.active;
        if (args.symbol !== undefined && !args.symbol.trim()) throw new Error("Symbol selection requires a nonempty exact name.");
        if (args.symbol === undefined && (uri?.scheme !== "file" || !uri.path.toLowerCase().endsWith(".fpas") || position === undefined)) {
          throw new Error("Select an FPAS function or procedure, or provide a symbol and manifest context.");
        }
        if (args.symbol !== undefined && !args.context) throw new Error("Symbol selection requires a manifest context.");
        const input = (value: string | undefined, prompt: string) => value === undefined
          ? vscode.window.showInputBox({ prompt, ignoreFocusOut: true, validateInput: text => text.trim() ? undefined : "Enter a complete value." })
          : Promise.resolve(value);
        const name = await input(args.name, "New parameter name");
        if (name === undefined) return;
        const type = await input(args.type, "FPAS type expression in the declaration's scope");
        if (type === undefined) return;
        const mode = args.mode ?? await vscode.window.showQuickPick(["value", "var"] as const, { title: "Parameter passing mode", ignoreFocusOut: true });
        if (mode === undefined) return;
        if (mode !== "value" && mode !== "var") throw new Error("Parameter mode must be value or var.");
        const argument = await input(args.argument, mode === "var" ? "Writable storage in every caller (without the var marker)" : "Argument expression evaluated in every caller");
        if (argument === undefined) return;
        const selector = args.symbol !== undefined ? { kind: "symbol" as const, name: args.symbol }
          : { kind: "position" as const, uri: uri!.toString(), position: { line: position!.line, character: position!.character } };
        const result = await vscode.window.withProgress({ location: vscode.ProgressLocation.Window, title: "Checking parameter change", cancellable: true },
          (_progress, token) => client.prepareParameter({ version: 1, phase: "prepare", context: args.context?.toString(),
            recipe: { action: "add-parameter", selector, name, type, mode, argument } }, token));
        const previewUri = vscode.Uri.from({ scheme: SCHEME, path: `/parameter-${++next}.md` });
        views.set(previewUri.toString(), render(result));
        const preview = await vscode.workspace.openTextDocument(previewUri);
        await vscode.window.showTextDocument(preview, { viewColumn: vscode.ViewColumn.Beside, preview: true, preserveFocus: true });
        if (result.status !== "prepared" || result.handle === null || args.previewOnly) return { ...result, previewUri };
        // The server revalidates the complete read set immediately before workspace/applyEdit.
        await client.applyRefactor(result.handle);
        return { ...result, status: "applied" as const, previewUri };
      } catch (error) {
        void vscode.window.showErrorMessage(`Parameter change unavailable: ${error instanceof Error ? error.message : String(error)}`);
        throw error;
      }
    })
  );
}

function render(result: RefactorResponse): string {
  const lines = ["# Add parameter", "", `Status: ${result.status.replaceAll("_", " ")}.`, ""];
  for (const blocker of result.blockers) {
    lines.push(`- ${blocker.code}: ${blocker.message}${blocker.path === null ? "" : ` (${blocker.path}${blocker.start === null ? "" : `, byte ${blocker.start}`})`}`);
  }
  for (const diagnostic of result.diagnostics) lines.push(`- ${diagnostic.message}`);
  for (const change of result.edit?.documentChanges ?? []) {
    lines.push("", `## ${vscode.workspace.asRelativePath(vscode.Uri.parse(change.textDocument.uri), false)}`, "");
    for (const edit of change.edits) {
      const fence = "`".repeat(Math.max(3, ...Array.from(edit.newText.matchAll(/`+/gu), match => match[0].length + 1)));
      lines.push(`Insert at ${edit.range.start.line + 1}:${edit.range.start.character + 1} (UTF-16):`, "", `${fence}fpas`, edit.newText, fence, "");
    }
  }
  if (result.limitations.length) lines.push("", ...result.limitations.map(value => `- ${value}`));
  lines.push("", "This preview records the prepared snapshot. Application rechecks current inputs and leaves buffers unsaved.", "");
  return lines.join("\n");
}
