/** Interactive dictionary structure commands over the FPAS DAP adapter. */

import * as vscode from "vscode";

/** Stable command identifiers contributed by the Functional Pascal extension. */
export const DICTIONARY_COMMANDS = {
  insert: "functionalPascal.debug.insertDictionaryEntry",
  remove: "functionalPascal.debug.removeDictionaryEntry",
  replaceKey: "functionalPascal.debug.replaceDictionaryKey"
} as const;

/** Optional arguments used by command links and Extension Host tests. */
export interface DictionaryCommandInput {
  readonly frameId?: number;
  readonly target?: string;
  readonly key?: string;
  readonly value?: string;
  readonly newKey?: string;
}

/** Register editor commands for all supported dictionary structure operations. */
export function registerDictionaryCommands(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand(DICTIONARY_COMMANDS.insert, async (input?: DictionaryCommandInput) => {
      const selection = await activeSelection(input?.frameId);
      if (selection === undefined) return;
      const target = input?.target ?? await prompt("Mutable dictionary target", "Scores");
      if (target === undefined) return;
      const key = input?.key ?? await prompt("Missing key expression", "'NewKey'");
      if (key === undefined) return;
      const value = input?.value ?? await prompt("Value expression", "0");
      if (value === undefined) return;
      await request(selection, "fpas/dictionaryInsert", { target, key, value });
    }),
    vscode.commands.registerCommand(DICTIONARY_COMMANDS.remove, async (input?: DictionaryCommandInput) => {
      const selection = await activeSelection(input?.frameId);
      if (selection === undefined) return;
      const target = input?.target ?? await prompt("Mutable dictionary target", "Scores");
      if (target === undefined) return;
      const key = input?.key ?? await prompt("Existing key expression", "'Key'");
      if (key === undefined) return;
      await request(selection, "fpas/dictionaryRemove", { target, key });
    }),
    vscode.commands.registerCommand(DICTIONARY_COMMANDS.replaceKey, async (input?: DictionaryCommandInput) => {
      const selection = await activeSelection(input?.frameId);
      if (selection === undefined) return;
      const target = input?.target ?? await prompt("Mutable dictionary target", "Scores");
      if (target === undefined) return;
      const key = input?.key ?? await prompt("Existing key expression", "'OldKey'");
      if (key === undefined) return;
      const newKey = input?.newKey ?? await prompt("Missing replacement key expression", "'NewKey'");
      if (newKey === undefined) return;
      await request(selection, "fpas/dictionaryReplaceKey", { target, key, newKey });
    })
  );
}

interface DebugSelection {
  readonly session: vscode.DebugSession;
  readonly frameId: number;
}

async function activeSelection(frameId?: number): Promise<DebugSelection | undefined> {
  const session = vscode.debug.activeDebugSession;
  if (session?.type !== "fpas") {
    void vscode.window.showWarningMessage(
      "Start and stop a Functional Pascal debug session before changing a dictionary."
    );
    return undefined;
  }
  if (frameId !== undefined) return { session, frameId };
  const selection = vscode.debug.activeStackItem;
  if (selection instanceof vscode.DebugStackFrame && selection.session === session) {
    return { session, frameId: selection.frameId };
  }
  void vscode.window.showWarningMessage(
    "Select a stopped Functional Pascal stack frame before changing a dictionary."
  );
  return undefined;
}

async function prompt(label: string, value: string): Promise<string | undefined> {
  return vscode.window.showInputBox({
    prompt: label,
    value,
    ignoreFocusOut: true,
    validateInput: (input) => input.trim().length === 0 ? "Enter one FPAS expression." : undefined
  });
}

async function request(
  selection: DebugSelection,
  command: string,
  requestArguments: Record<string, string>
): Promise<void> {
  try {
    const result = await selection.session.customRequest(command, {
      frameId: selection.frameId,
      ...requestArguments
    }) as { value?: string };
    void vscode.window.showInformationMessage(
      `Functional Pascal dictionary updated: ${result.value ?? "committed"}`
    );
  } catch (error) {
    void vscode.window.showErrorMessage(
      `Functional Pascal dictionary update failed: ${errorMessage(error)}`
    );
  }
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
