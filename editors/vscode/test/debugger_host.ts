/** Real Extension Host coverage for the Functional Pascal debugger. */

import * as vscode from "vscode";

import { verifyDebuggerLifecycle } from "./debugger_host/lifecycle";
import { verifyBreakpointPolicies } from "./debugger_host/breakpoint_policies";
import { verifyDebuggerEvaluation } from "./debugger_host/evaluation";
import { verifyPauseAndDisconnect } from "./debugger_host/pause";
import { verifyRuntimeFailure } from "./debugger_host/runtime_failure";
import { verifyVariableMutation } from "./debugger_host/variable_mutation";
import type { DapMessage } from "./debugger_host/support";

/** Exercise every user-visible VS Code debugger capability promised for V1. */
export async function verifyDebuggerHost(): Promise<void> {
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (!workspaceRoot) throw new Error("extension test workspace is not open");

  const received: DapMessage[] = [];
  const sent: DapMessage[] = [];
  const tracker = vscode.debug.registerDebugAdapterTrackerFactory("fpas", {
    createDebugAdapterTracker: () => ({
      onWillReceiveMessage: (message: DapMessage) => received.push(message),
      onDidSendMessage: (message: DapMessage) => sent.push(message)
    })
  });

  try {
    await verifyDebuggerLifecycle(workspaceRoot, received, sent);
    await verifyPauseAndDisconnect(workspaceRoot, received, sent);
    await verifyRuntimeFailure(workspaceRoot, received, sent);
    await verifyDebuggerEvaluation(workspaceRoot, received, sent);
    await verifyVariableMutation(workspaceRoot, received, sent);
    await verifyBreakpointPolicies(workspaceRoot, received, sent);
  } finally {
    await vscode.debug.stopDebugging();
    tracker.dispose();
  }
}
