/** Interactive array and string structure commands over the FPAS DAP adapter. */

import * as vscode from "vscode";
import { activeSelection, type DebugSelection } from "./commands/selection";
import { requestCollectionMutation as request } from "./commands/collectionMutation";
import { promptExpression as prompt } from "./commands/expressionPrompt";

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
      const selection = await activeSelection(input?.frameId, "changing an array or string");
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
      const selection = await activeSelection(input?.frameId, "changing an array or string");
      if (selection === undefined) return;
      const target = input?.target ?? await prompt("Mutable array target", "Values");
      if (target === undefined) return;
      const index = input?.index ?? await prompt("Removal index expression", "0");
      if (index === undefined) return;
      await request(selection, "fpas/arrayRemove", { target, index }, "array");
    }),
    vscode.commands.registerCommand(SEQUENCE_COMMANDS.replaceStringCharacter, async (input?: SequenceCommandInput) => {
      const selection = await activeSelection(input?.frameId, "changing an array or string");
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
