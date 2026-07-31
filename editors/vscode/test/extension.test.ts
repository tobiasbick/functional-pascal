import assert from "node:assert/strict";
import path from "node:path";
import * as vscode from "vscode";

import type { FunctionalPascalExtensionApi } from "../src/extension";

const EXTENSION_ID = "functional-pascal.functional-pascal";
const SHOW_OUTPUT_COMMAND = "functionalPascal.showOutput";
const RESTART_LANGUAGE_SERVER_COMMAND =
  "functionalPascal.restartLanguageServer";
const ACTIVATION_MESSAGE =
  "Functional Pascal extension activated (Hello World).";

/** Runs the extension-shell regression test in a real Extension Host. */
export async function run(): Promise<void> {
  const extension =
    vscode.extensions.getExtension<FunctionalPascalExtensionApi>(EXTENSION_ID);

  assert.ok(extension, `extension ${EXTENSION_ID} is available`);

  const api = await extension.activate();
  assert.equal(extension.isActive, true);
  assert.equal(api.activationMessage, ACTIVATION_MESSAGE);
  assert.equal(api.languageServerStarted, true, api.languageServerError);
  assert.equal(
    path.basename(api.languageServerPath ?? ""),
    process.platform === "win32" ? "fpas-lsp.exe" : "fpas-lsp"
  );

  const commands = await vscode.commands.getCommands(true);
  assert.ok(commands.includes(SHOW_OUTPUT_COMMAND));
  assert.ok(commands.includes(RESTART_LANGUAGE_SERVER_COMMAND));

  const fixture = vscode.Uri.file(
    path.join(
      extension.extensionPath,
      "test",
      "fixtures",
      "standalone",
      "malformed_syntax.fpas"
    )
  );
  const document = await vscode.workspace.openTextDocument(fixture);
  assert.equal(document.languageId, "fpas");
  const editor = await vscode.window.showTextDocument(document);

  const parserDiagnostics = await waitForDiagnostics(
    document.uri,
    (diagnostics) => diagnostics.length > 0
  );
  assert.ok(
    parserDiagnostics.some((diagnostic) => diagnostic.code === "F1001"),
    JSON.stringify(parserDiagnostics)
  );

  const messySource =
    "program Corrected; begin // kept\n var Value:integer:=1 end.";
  await editor.edit((edit) => {
    edit.replace(
      new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)),
      messySource
    );
  });
  await waitForDiagnostics(
    document.uri,
    (diagnostics) => diagnostics.length === 0
  );

  const formattingEdits = await vscode.commands.executeCommand<
    vscode.TextEdit[]
  >("vscode.executeFormatDocumentProvider", document.uri, {
    tabSize: 2,
    insertSpaces: true
  });
  assert.ok(formattingEdits);
  assert.ok(formattingEdits.length > 0);
  const formattingWorkspaceEdit = new vscode.WorkspaceEdit();
  formattingWorkspaceEdit.set(document.uri, formattingEdits);
  assert.equal(await vscode.workspace.applyEdit(formattingWorkspaceEdit), true);
  assert.equal(
    document.getText(),
    "program Corrected;\n\nbegin\n  var Value: integer := 1\nend.\n// kept\n"
  );

  await vscode.commands.executeCommand("workbench.action.revertAndCloseActiveEditor");

  const workspaceMain = vscode.Uri.file(
    path.join(
      extension.extensionPath,
      "test",
      "fixtures",
      "workspace",
      "apps",
      "demo",
      "src",
      "main.fpas"
    )
  );
  const navigationDocument = await vscode.workspace.openTextDocument(workspaceMain);
  await vscode.window.showTextDocument(navigationDocument);
  const greetingOffset = navigationDocument
    .getText()
    .lastIndexOf("GreetingFor");
  assert.ok(greetingOffset >= 0);
  const greetingPosition = navigationDocument.positionAt(greetingOffset);

  const hovers = await vscode.commands.executeCommand<vscode.Hover[]>(
    "vscode.executeHoverProvider",
    navigationDocument.uri,
    greetingPosition
  );
  assert.ok(hovers?.some((hover) => hover.contents.length > 0));
  const definitions = await vscode.commands.executeCommand<
    Array<vscode.Location | vscode.LocationLink>
  >(
    "vscode.executeDefinitionProvider",
    navigationDocument.uri,
    greetingPosition
  );
  assert.ok(definitions?.length);
  const references = await vscode.commands.executeCommand<vscode.Location[]>(
    "vscode.executeReferenceProvider",
    navigationDocument.uri,
    greetingPosition
  );
  assert.ok(references?.length >= 2);
  const renameEdit = await vscode.commands.executeCommand<vscode.WorkspaceEdit>(
    "vscode.executeDocumentRenameProvider",
    navigationDocument.uri,
    greetingPosition,
    "GreetingForEditor"
  );
  assert.ok(renameEdit);
  const renameEntries = renameEdit.entries();
  assert.equal(
    renameEntries.reduce((count, [, edits]) => count + edits.length, 0),
    2
  );
  assert.ok(
    renameEntries.every(([, edits]) =>
      edits.every((edit) => edit.newText === "GreetingForEditor")
    )
  );
  const symbols = await vscode.commands.executeCommand<
    Array<vscode.DocumentSymbol | vscode.SymbolInformation>
  >("vscode.executeDocumentSymbolProvider", navigationDocument.uri);
  assert.ok(symbols?.some((symbol) => symbol.name === "EditorDemo"));
  const completions = await vscode.commands.executeCommand<vscode.CompletionList>(
    "vscode.executeCompletionItemProvider",
    navigationDocument.uri,
    greetingPosition
  );
  assert.ok(completions?.items.some((item) => item.label === "GreetingFor"));

  await vscode.commands.executeCommand("workbench.action.closeActiveEditor");

  const notesTheme = vscode.Uri.file(
    path.resolve(
      extension.extensionPath,
      "..",
      "..",
      "apps",
      "notes",
      "src",
      "Notes",
      "Theme.fpas"
    )
  );
  const notesDocument = await vscode.workspace.openTextDocument(notesTheme);
  await vscode.window.showTextDocument(notesDocument);
  const paletteOffset = notesDocument.getText().indexOf("TuiPalette");
  assert.ok(paletteOffset >= 0);
  const palettePosition = notesDocument.positionAt(paletteOffset);
  const paletteHovers = await waitForHovers(notesDocument.uri, palettePosition);
  assert.ok(paletteHovers.some((hover) => hover.contents.length > 0));
  await waitForDiagnostics(
    notesDocument.uri,
    (diagnostics) => diagnostics.length === 0
  );
  await vscode.commands.executeCommand("workbench.action.closeActiveEditor");

  await vscode.commands.executeCommand(RESTART_LANGUAGE_SERVER_COMMAND);
  await vscode.commands.executeCommand(SHOW_OUTPUT_COMMAND);
  console.log(
    "Functional Pascal extension diagnostics, formatting, navigation, and lifecycle test passed."
  );
}

async function waitForHovers(
  uri: vscode.Uri,
  position: vscode.Position
): Promise<vscode.Hover[]> {
  const deadline = Date.now() + 10_000;
  while (Date.now() < deadline) {
    const hovers = await vscode.commands.executeCommand<vscode.Hover[]>(
      "vscode.executeHoverProvider",
      uri,
      position
    );
    if (hovers?.length) {
      return hovers;
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  assert.fail("timed out waiting for Notes TuiPalette hover");
}

async function waitForDiagnostics(
  uri: vscode.Uri,
  predicate: (diagnostics: readonly vscode.Diagnostic[]) => boolean
): Promise<readonly vscode.Diagnostic[]> {
  const deadline = Date.now() + 5_000;
  while (Date.now() < deadline) {
    const diagnostics = vscode.languages.getDiagnostics(uri);
    if (predicate(diagnostics)) {
      return diagnostics;
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  const diagnostics = vscode.languages.getDiagnostics(uri);
  assert.fail(`timed out waiting for diagnostics: ${JSON.stringify(diagnostics)}`);
}
