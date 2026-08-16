/** Real Extension Host coverage for the Functional Pascal debugger. */

import * as vscode from "vscode";

import { verifyDebuggerLifecycle } from "./debugger_host/lifecycle";
import { verifyBreakpointPolicies } from "./debugger_host/breakpoint_policies";
import { verifyDebuggerEvaluation } from "./debugger_host/evaluation";
import { verifyExpressionMutation } from "./debugger_host/expression_mutation";
import { verifyDictionaryMutation } from "./debugger_host/dictionary_mutation";
import { verifySequenceMutation } from "./debugger_host/sequence_mutation";
import { verifyPayloadMutation } from "./debugger_host/payload_mutation";
import { verifyVariantReplacement } from "./debugger_host/variant_replacement";
import { verifyVariantTransition } from "./debugger_host/variant_transition";
import { verifyUninitializedAssignment } from "./debugger_host/uninitialized_assignment";
import { verifyFunctionValueAssignment } from "./debugger_host/function_value_assignment";
import { verifyFunctionBreakpoints } from "./debugger_host/function_breakpoints";
import { verifyCapturingRoutineAssignment } from "./debugger_host/capturing_routine_assignment";
import { verifyCellCapturingRoutineAssignment } from "./debugger_host/cell_capturing_routine_assignment";
import { verifyTaskHandleAssignment } from "./debugger_host/task_handle_assignment";
import { verifyForcedReturn } from "./debugger_host/forced_return";
import { verifyFrameRestart } from "./debugger_host/frame_restart";
import { verifyTaskResultReplacement } from "./debugger_host/task_result_replacement";
import { verifyVariantConstruction } from "./debugger_host/variant_construction";
import { verifyEmptyStorageConstruction } from "./debugger_host/empty_storage_construction";
import { verifyPauseAndDisconnect } from "./debugger_host/pause";
import { verifyRuntimeFailure } from "./debugger_host/runtime_failure";
import { verifyRuntimeFailureFilters } from "./debugger_host/runtime_failure_filters";
import { verifyTaskDebugging } from "./debugger_host/task_debugging";
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
    await verifyRuntimeFailureFilters(workspaceRoot, received, sent);
    await verifyDebuggerEvaluation(workspaceRoot, received, sent);
    await verifyExpressionMutation(workspaceRoot, received, sent);
    await verifyDictionaryMutation(workspaceRoot, received, sent);
    await verifySequenceMutation(workspaceRoot, received, sent);
    await verifyPayloadMutation(workspaceRoot, received, sent);
    await verifyVariantReplacement(workspaceRoot, received, sent);
    await verifyVariantTransition(workspaceRoot, received, sent);
    await verifyUninitializedAssignment(workspaceRoot, received, sent);
    await verifyFunctionValueAssignment(workspaceRoot, received, sent);
    await verifyFunctionBreakpoints(workspaceRoot, received, sent);
    await verifyCapturingRoutineAssignment(workspaceRoot, received, sent);
    await verifyCellCapturingRoutineAssignment(workspaceRoot, received, sent);
    await verifyTaskHandleAssignment(workspaceRoot, received, sent);
    await verifyForcedReturn(workspaceRoot, received, sent);
    await verifyFrameRestart(workspaceRoot, received, sent);
    await verifyTaskResultReplacement(workspaceRoot, received, sent);
    await verifyVariantConstruction(workspaceRoot, received, sent);
    await verifyEmptyStorageConstruction(workspaceRoot, received, sent);
    await verifyVariableMutation(workspaceRoot, received, sent);
    await verifyBreakpointPolicies(workspaceRoot, received, sent);
    await verifyTaskDebugging(workspaceRoot, received, sent);
  } finally {
    await vscode.debug.stopDebugging();
    tracker.dispose();
  }
}
