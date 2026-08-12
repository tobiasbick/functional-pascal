/** Interactive array and string structure commands over the FPAS DAP adapter. */

import * as vscode from "vscode";

/** Stable command identifiers contributed by the Functional Pascal extension. */
export const SEQUENCE_COMMANDS = {
  insertArray: "functionalPascal.debug.insertArrayElement",
  removeArray: "functionalPascal.debug.removeArrayElement",
  replaceStringCharacter: "functionalPascal.debug.replaceStringCharacter"
} as const;

/** Optional arguments used by command links and Extension Host tests. */
export interface SequenceCommandInput {
  readonly frameId?: number;
  readonly target?: string;
  readonly index?: string;
  readonly value?: string;
}

/** Register editor commands for all supported sequence structure operations. */
export function registerSequenceCommands(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand(SEQUENCE_COMMANDS.insertArray, async (input?: SequenceCommandInput) => {
      const selection = await activeSelection(input?.frameId);
      if (selection === undefined) return;
      const target = input?.target ?? await prompt("Mutable array target", "Values");
      if (target === undefined) return;
      const index = input?.index ?? await prompt("Insertion index expression", "0");
      if (index === undefined) return;
      const value = input?.value ?? await prompt("Element expression", "0");
      if (value === undefined) return;
      await request(selection, "fpas/arrayInsert", { target, index, value }, "array");
    }),
    vscode.commands.registerCommand(SEQUENCE_COMMANDS.removeArray, async (input?: SequenceCommandInput) => {
      const selection = await activeSelection(input?.frameId);
      if (selection === undefined) return;
      const target = input?.target ?? await prompt("Mutable array target", "Values");
      if (target === undefined) return;
      const index = input?.index ?? await prompt("Removal index expression", "0");
      if (index === undefined) return;
      await request(selection, "fpas/arrayRemove", { target, index }, "array");
    }),
    vscode.commands.registerCommand(SEQUENCE_COMMANDS.replaceStringCharacter, async (input?: SequenceCommandInput) => {
      const selection = await activeSelection(input?.frameId);
      if (selection === undefined) return;
      const target = input?.target ?? await prompt("Mutable string target", "Text");
      if (target === undefined) return;
      const index = input?.index ?? await prompt("Unicode character index expression", "0");
      if (index === undefined) return;
      const value = input?.value ?? await prompt("Single-character string expression", "'x'");
      if (value === undefined) return;
      await request(selection, "fpas/stringReplaceCharacter", { target, index, value }, "string");
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
      "Start and stop a Functional Pascal debug session before changing an array or string."
    );
    return undefined;
  }
  if (frameId !== undefined) return { session, frameId };
  const selection = vscode.debug.activeStackItem;
  if (selection instanceof vscode.DebugStackFrame && selection.session === session) {
    return { session, frameId: selection.frameId };
  }
  void vscode.window.showWarningMessage(
    "Select a stopped Functional Pascal stack frame before changing an array or string."
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
  requestArguments: Record<string, string>,
  kind: "array" | "string"
): Promise<void> {
  try {
    const result = await selection.session.customRequest(command, {
      frameId: selection.frameId,
      ...requestArguments
    }) as { value?: string };
    void vscode.window.showInformationMessage(
      `Functional Pascal ${kind} updated: ${result.value ?? "committed"}`
    );
  } catch (error) {
    void vscode.window.showErrorMessage(
      `Functional Pascal ${kind} update failed: ${errorMessage(error)}`
    );
  }
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
