import * as vscode from "vscode";
import type { DebugSelection } from "./selection";

/** Sends a collection mutation against the captured frame and reports its result. */
export async function requestCollectionMutation(
  selection: DebugSelection,
  command: string,
  args: Record<string, string>,
  kind: "dictionary" | "array" | "string"
): Promise<void> {
  try {
    const result = await selection.session.customRequest(command, { frameId: selection.frameId, ...args }) as { value?: string };
    void vscode.window.showInformationMessage(`Functional Pascal ${kind} updated: ${result.value ?? "committed"}`);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    void vscode.window.showErrorMessage(`Functional Pascal ${kind} update failed: ${message}`);
  }
}
