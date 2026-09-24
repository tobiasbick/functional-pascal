import * as vscode from "vscode";

/** Session and stopped frame captured before any command prompt opens. */
export interface DebugSelection {
  readonly session: vscode.DebugSession;
  readonly frameId: number;
}

/** Captures one FPAS frame, retaining each command's action-specific guidance. */
export async function activeSelection(frameId: number | undefined, action: string): Promise<DebugSelection | undefined> {
  return selectionFor(vscode.debug.activeDebugSession, vscode.debug.activeStackItem, frameId, action);
}

/** Resolve a captured debug context without reading mutable editor selection again. */
export function selectionFor(
  session: vscode.DebugSession | undefined,
  stackItem: typeof vscode.debug.activeStackItem,
  frameId: number | undefined,
  action: string
): DebugSelection | undefined {
  if (session?.type !== "fpas") {
    void vscode.window.showWarningMessage(`Start and stop a Functional Pascal debug session before ${action}.`);
    return undefined;
  }
  if (frameId !== undefined) return { session, frameId };
  if (stackItem instanceof vscode.DebugStackFrame && stackItem.session === session) {
    return { session, frameId: stackItem.frameId };
  }
  void vscode.window.showWarningMessage(`Select a stopped Functional Pascal stack frame before ${action}.`);
  return undefined;
}
