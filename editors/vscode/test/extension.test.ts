import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import * as vscode from "vscode";

import type { FunctionalPascalExtensionApi } from "../src/extension";
import { verifyIntelliSense } from "./intellisense";
import { verifySemanticTools } from "./semantic_tools";
import { verifyWorkflowHost } from "./workflow/host";
import { verifyWorkflowUnits } from "./workflow/unit";
import { verifyDebuggerHost } from "./debugger_host";

const EXTENSION_ID = "functional-pascal.functional-pascal";
const SHOW_OUTPUT_COMMAND = "functionalPascal.showOutput";
const RESTART_LANGUAGE_SERVER_COMMAND =
  "functionalPascal.restartLanguageServer";
const ACTIVATION_MESSAGE =
  "Functional Pascal extension activated.";

/** Runs the extension-shell regression test in a real Extension Host. */
export async function run(): Promise<void> {
  await verifyWorkflowUnits();
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
    "program Corrected;\n\nbegin\n  // kept\n  var Value: integer := 1\nend.\n"
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
  const greetingCompletion = await waitForNamedCompletion(
    navigationDocument.uri,
    greetingPosition,
    "GreetingFor"
  );
  assert.equal(completionLabel(greetingCompletion), "GreetingFor");

  await vscode.commands.executeCommand("workbench.action.closeActiveEditor");

  await verifyExternalProjectChanges();
  await verifyWorkspaceNavigation();
  await verifyIntelliSense(extension.extensionPath);
  await verifySemanticTools(extension);
  await verifyWorkflowHost(api);
  await verifyDebuggerHost();

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
    "Functional Pascal extension diagnostics, formatting, navigation, IntelliSense, semantic tools, project workflows, debugger, and lifecycle test passed."
  );
}

async function verifyExternalProjectChanges(): Promise<void> {
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  assert.ok(workspaceRoot, "extension test workspace is open");
  const fixtureRoot = path.join(
    workspaceRoot,
    `.project-index-${process.pid}-${Date.now()}`
  );
  const coreSource = path.join(fixtureRoot, "core", "src", "core.fpas");
  const appSource = path.join(fixtureRoot, "app", "src", "main.fpas");
  const appManifest = path.join(fixtureRoot, "app", "app.fpasprj");
  try {
    await fs.mkdir(path.dirname(coreSource), { recursive: true });
    await fs.mkdir(path.dirname(appSource), { recursive: true });
    await fs.writeFile(
      path.join(fixtureRoot, "core", "core.fpasprj"),
      '[project]\nname = "watch-core"\nkind = "library"\n\n[sources]\ninclude = ["src/**/*.fpas"]\n'
    );
    const declarationSource =
      "unit Watch.Core;\n\npublic function WatchedValue(): integer;\nbegin return 42 end;\n";
    await fs.writeFile(coreSource, declarationSource);
    await fs.writeFile(
      appManifest,
      '[project]\nname = "watch-app"\nkind = "program"\nmain = "src/main.fpas"\n\n[sources]\ninclude = ["src/**/*.fpas"]\n'
    );
    await fs.writeFile(
      appSource,
      "program WatchApp;\n\nuses Watch.Core;\n\nbegin var First: integer := WatchedValue() end.\n"
    );

    const declarationUri = vscode.Uri.file(coreSource);
    const document = await vscode.workspace.openTextDocument(declarationUri);
    await vscode.window.showTextDocument(document);
    const position = document.positionAt(document.getText().indexOf("WatchedValue"));
    await waitForReferences(declarationUri, position, 1);

    await fs.writeFile(
      appManifest,
      '[project]\nname = "watch-app"\nkind = "program"\nmain = "src/main.fpas"\n\n[dependencies]\nprojects = ["../core/core.fpasprj"]\n\n[sources]\ninclude = ["src/**/*.fpas"]\n'
    );
    await waitForReferences(declarationUri, position, 2);

    await fs.writeFile(
      appSource,
      "program WatchApp;\n\nuses Watch.Core;\n\nbegin\n  var First: integer := WatchedValue();\n  var Second: integer := WatchedValue()\nend.\n"
    );
    await waitForReferences(declarationUri, position, 3);
  } finally {
    await vscode.commands.executeCommand("workbench.action.closeActiveEditor");
  }
}

async function verifyWorkspaceNavigation(): Promise<void> {
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  assert.ok(workspaceRoot, "extension test workspace is open");
  const fixtureRoot = path.join(
    workspaceRoot,
    `.workspace-navigation-${process.pid}-${Date.now()}`
  );
  const coreSource = path.join(fixtureRoot, "core", "src", "core.fpas");
  const appSource = path.join(fixtureRoot, "app", "src", "main.fpas");
  try {
    await fs.mkdir(path.dirname(coreSource), { recursive: true });
    await fs.mkdir(path.dirname(appSource), { recursive: true });
    await fs.writeFile(
      path.join(fixtureRoot, "core", "core.fpasprj"),
      '[project]\nname = "navigation-core"\nkind = "library"\n\n[sources]\ninclude = ["src/**/*.fpas"]\n'
    );
    await fs.writeFile(
      coreSource,
      "unit Navigation.Core;\n\npublic type HostPoint = record\n  public X: integer;\nend;\n"
    );
    await fs.writeFile(
      path.join(fixtureRoot, "app", "app.fpasprj"),
      '[project]\nname = "navigation-app"\nkind = "program"\nmain = "src/main.fpas"\n\n[dependencies]\nprojects = ["../core/core.fpasprj"]\n\n[sources]\ninclude = ["src/**/*.fpas"]\n'
    );
    const source =
      "program NavigationApp;\n\nuses Navigation.Core;\n\nmutable var Counter: integer := 0;\n\nbegin\n  Counter := Counter + 1;\n  var Value: HostPoint := record X := Counter; end\nend.\n";
    await fs.writeFile(appSource, source);

    const symbols = await waitForWorkspaceSymbols("HostPoint");
    assert.ok(
      symbols.some(
        (symbol) =>
          symbol.name === "HostPoint" &&
          symbol.containerName === "Navigation.Core"
      )
    );

    const uri = vscode.Uri.file(appSource);
    const document = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(document);
    const counterPosition = document.positionAt(source.indexOf("Counter :="));
    const highlights = await vscode.commands.executeCommand<
      vscode.DocumentHighlight[]
    >("vscode.executeDocumentHighlights", uri, counterPosition);
    assert.equal(highlights?.length, 4);
    assert.ok(
      highlights.some(
        (highlight) => highlight.kind === vscode.DocumentHighlightKind.Write
      )
    );

    const valuePosition = document.positionAt(source.indexOf("Value:"));
    const typeDefinitions = await vscode.commands.executeCommand<
      Array<vscode.Location | vscode.LocationLink>
    >("vscode.executeTypeDefinitionProvider", uri, valuePosition);
    assert.ok(typeDefinitions?.length);
    assert.ok(
      typeDefinitions.some((definition) =>
        (definition instanceof vscode.Location
          ? definition.uri
          : definition.targetUri
        ).fsPath.endsWith(path.join("core", "src", "core.fpas"))
      )
    );
  } finally {
    await vscode.commands.executeCommand("workbench.action.closeActiveEditor");
  }
}

async function waitForWorkspaceSymbols(
  query: string
): Promise<vscode.SymbolInformation[]> {
  const deadline = Date.now() + 10_000;
  while (Date.now() < deadline) {
    const symbols =
      (await vscode.commands.executeCommand<vscode.SymbolInformation[]>(
        "vscode.executeWorkspaceSymbolProvider",
        query
      )) ?? [];
    if (symbols.some((symbol) => symbol.name === query)) {
      return symbols;
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  assert.fail(`timed out waiting for workspace symbol ${query}`);
}

async function waitForReferences(
  uri: vscode.Uri,
  position: vscode.Position,
  expected: number
): Promise<vscode.Location[]> {
  const deadline = Date.now() + 10_000;
  while (Date.now() < deadline) {
    const references =
      (await vscode.commands.executeCommand<vscode.Location[]>(
        "vscode.executeReferenceProvider",
        uri,
        position
      )) ?? [];
    if (references.length === expected) {
      return references;
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  assert.fail(`timed out waiting for ${expected} references`);
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

async function waitForNamedCompletion(
  uri: vscode.Uri,
  position: vscode.Position,
  name: string
): Promise<vscode.CompletionItem> {
  const deadline = Date.now() + 10_000;
  while (Date.now() < deadline) {
    const completions = await vscode.commands.executeCommand<vscode.CompletionList>(
      "vscode.executeCompletionItemProvider",
      uri,
      position
    );
    const item = completions?.items.find(
      (candidate) => completionLabel(candidate) === name
    );
    if (item) {
      return item;
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  assert.fail(`timed out waiting for completion ${name}`);
}

function completionLabel(item: vscode.CompletionItem): string {
  return typeof item.label === "string" ? item.label : item.label.label;
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
