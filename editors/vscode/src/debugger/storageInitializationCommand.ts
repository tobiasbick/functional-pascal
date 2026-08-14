/** Interactive seeded initialization of a descendant below empty debugger storage. */

import * as vscode from "vscode";

/** Stable command identifier contributed by the Functional Pascal extension. */
export const INITIALIZE_STORAGE_COMMAND = "functionalPascal.debug.initializeStorage";

/** Optional arguments used by command links and Extension Host tests. */
export interface StorageInitializationInput {
  readonly frameId?: number;
  readonly target?: string;
  readonly initializer?: string;
  readonly expression?: string;
  readonly [key: string]: unknown;
}

/** Register the editor command that seeds one empty mutable local or global descendant. */
export function registerStorageInitializationCommand(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand(
      INITIALIZE_STORAGE_COMMAND,
      async (input?: StorageInitializationInput) => {
        const selection = await activeSelection(input?.frameId);
        if (selection === undefined) return;
        try {
          const target = input?.target ?? await prompt("Empty mutable descendant target", "State.Count");
          if (target === undefined) return;
          const initializer = input?.initializer
            ?? await prompt("Complete root initializer", "MakeInitialState()");
          if (initializer === undefined) return;
          const expression = input?.expression ?? await prompt("Descendant replacement expression", "0");
          if (expression === undefined) return;
          const result = await selection.session.customRequest("fpas/initializeStorage", {
            ...input,
            frameId: selection.frameId,
            target,
            initializer,
            expression
          }) as { value?: string; target?: string };
          void vscode.window.showInformationMessage(
            `Functional Pascal initialized ${result.target ?? target}: ${result.value ?? "committed"}`
          );
        } catch (error) {
          void vscode.window.showErrorMessage(
            `Functional Pascal empty-storage initialization failed: ${errorMessage(error)}`
          );
          if (input !== undefined) throw error;
        }
      }
    )
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
      "Start and stop a Functional Pascal debug session before initializing empty storage."
    );
    return undefined;
  }
  if (frameId !== undefined) return { session, frameId };
  const selection = vscode.debug.activeStackItem;
  if (selection instanceof vscode.DebugStackFrame && selection.session === session) {
    return { session, frameId: selection.frameId };
  }
  void vscode.window.showWarningMessage(
    "Select a stopped Functional Pascal stack frame before initializing empty storage."
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

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
