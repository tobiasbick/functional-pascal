/** Interactive dictionary structure commands over the FPAS DAP adapter. */

import * as vscode from "vscode";
import { activeSelection, type DebugSelection } from "./commands/selection";
import { requestCollectionMutation as request } from "./commands/collectionMutation";
import { promptExpression as prompt } from "./commands/expressionPrompt";

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
      const selection = await activeSelection(input?.frameId, "changing a dictionary");
      if (selection === undefined) return;
      const target = input?.target ?? await prompt("Mutable dictionary target", "Scores");
      if (target === undefined) return;
      const key = input?.key ?? await prompt("Missing key expression", "'NewKey'");
      if (key === undefined) return;
      const value = input?.value ?? await prompt("Value expression", "0");
      if (value === undefined) return;
      await request(selection, "fpas/dictionaryInsert", { target, key, value }, "dictionary");
    }),
    vscode.commands.registerCommand(DICTIONARY_COMMANDS.remove, async (input?: DictionaryCommandInput) => {
      const selection = await activeSelection(input?.frameId, "changing a dictionary");
      if (selection === undefined) return;
      const target = input?.target ?? await prompt("Mutable dictionary target", "Scores");
      if (target === undefined) return;
      const key = input?.key ?? await prompt("Existing key expression", "'Key'");
      if (key === undefined) return;
      await request(selection, "fpas/dictionaryRemove", { target, key }, "dictionary");
    }),
    vscode.commands.registerCommand(DICTIONARY_COMMANDS.replaceKey, async (input?: DictionaryCommandInput) => {
      const selection = await activeSelection(input?.frameId, "changing a dictionary");
      if (selection === undefined) return;
      const target = input?.target ?? await prompt("Mutable dictionary target", "Scores");
      if (target === undefined) return;
      const key = input?.key ?? await prompt("Existing key expression", "'OldKey'");
      if (key === undefined) return;
      const newKey = input?.newKey ?? await prompt("Missing replacement key expression", "'NewKey'");
      if (newKey === undefined) return;
      await request(selection, "fpas/dictionaryReplaceKey", { target, key, newKey }, "dictionary");
    })
  );
}
