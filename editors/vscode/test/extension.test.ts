import assert from "node:assert/strict";
import * as vscode from "vscode";

import type { FunctionalPascalExtensionApi } from "../src/extension";

const EXTENSION_ID = "functional-pascal.functional-pascal";
const SHOW_OUTPUT_COMMAND = "functionalPascal.showOutput";
const ACTIVATION_MESSAGE =
  "Functional Pascal extension activated (Hello World).";

/** Runs the bootstrap extension-host regression test. */
export async function run(): Promise<void> {
  const extension =
    vscode.extensions.getExtension<FunctionalPascalExtensionApi>(EXTENSION_ID);

  assert.ok(extension, `extension ${EXTENSION_ID} is available`);

  const api = await extension.activate();
  assert.equal(extension.isActive, true);
  assert.equal(api.activationMessage, ACTIVATION_MESSAGE);

  const commands = await vscode.commands.getCommands(true);
  assert.ok(commands.includes(SHOW_OUTPUT_COMMAND));

  await vscode.commands.executeCommand(SHOW_OUTPUT_COMMAND);
  console.log("Functional Pascal bootstrap extension test passed.");
}
