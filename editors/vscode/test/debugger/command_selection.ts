import assert from "node:assert/strict";

import * as vscode from "vscode";

import { selectionFor } from "../../src/debugger/commands/selection";
import { requestCollectionMutation } from "../../src/debugger/commands/collectionMutation";

/** Checks command context capture and request routing across an awaited prompt. */
export async function verifyCommandSelection(): Promise<void> {
  const sent: { command: string; args: Record<string, unknown> }[] = [];
  const session = {
    type: "fpas",
    customRequest: async (command: string, args: Record<string, unknown>) => {
      sent.push({ command, args });
      return { value: "done" };
    }
  } as unknown as vscode.DebugSession;
  assert.equal(selectionFor(undefined, undefined, 4, "editing"), undefined);
  assert.equal(selectionFor({ type: "other" } as vscode.DebugSession, undefined, 4, "editing"), undefined);
  assert.equal(selectionFor(session, undefined, undefined, "editing"), undefined);
  const frame = Object.assign(Object.create(vscode.DebugStackFrame.prototype) as vscode.DebugStackFrame,
    { session, frameId: 7 });
  const selected = selectionFor(session, frame, undefined, "editing");
  assert.equal(selected?.frameId, 7);
  const explicit = selectionFor(session, undefined, 19, "editing");
  assert.equal(explicit?.frameId, 19);
  assert.ok(selected);
  const prompt = Promise.resolve("Scores");
  await prompt;
  Object.assign(frame, { frameId: 99 });
  await requestCollectionMutation(selected, "fpas/dictionaryRemove", { target: "Scores", key: "'A'" }, "dictionary");
  assert.deepEqual(sent, [{ command: "fpas/dictionaryRemove",
    args: { frameId: 7, target: "Scores", key: "'A'" } }]);
}
