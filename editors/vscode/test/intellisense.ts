import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import * as vscode from "vscode";

/** Verifies semantic completion, signature help, snippets, and safe auto-imports. */
export async function verifyIntelliSense(extensionPath: string): Promise<void> {
  verifySnippetContribution(extensionPath);

  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  assert.ok(workspaceRoot, "extension test workspace is open");
  const fixtureRoot = path.join(
    workspaceRoot,
    `.intellisense-${process.pid}-${Date.now()}`
  );
  const sourcePath = path.join(fixtureRoot, "src", "main.fpas");
  const corePath = path.join(fixtureRoot, "src", "Intellisense", "Core.fpas");
  const importablePath = path.join(
    fixtureRoot,
    "src",
    "Intellisense",
    "Importable.fpas"
  );

  const source = [
    "program IntelliSenseHost;",
    "",
    "uses Intellisense.Core;",
    "",
    "begin",
    "  var CounterValue: Counter := record Amount := 1; end;",
    "  var Member: integer := CounterValue.Amount;",
    "  var Sum: integer := Add(1, 2);",
    "  var Imported: integer := UniqueHostValue()",
    "end.",
    ""
  ].join("\n");

  try {
    await fs.mkdir(path.dirname(corePath), { recursive: true });
    await fs.writeFile(
      path.join(fixtureRoot, "intellisense.fpasprj"),
      '[project]\nname = "intellisense-host"\nkind = "program"\nmain = "src/main.fpas"\n\n[sources]\ninclude = ["src/**/*.fpas"]\n'
    );
    await fs.writeFile(
      corePath,
      [
        "unit Intellisense.Core;",
        "",
        "public type",
        "  Counter = record",
        "    public Amount: integer;",
        "  end;",
        "",
        "public function Add(Left: integer; Right: integer): integer;",
        "begin",
        "  return Left + Right",
        "end;",
        ""
      ].join("\n")
    );
    await fs.writeFile(
      importablePath,
      [
        "unit Intellisense.Importable;",
        "",
        "// Returns the stable IntelliSense fixture value.",
        "public function UniqueHostValue(): integer;",
        "begin",
        "  return 42",
        "end;",
        ""
      ].join("\n")
    );
    await fs.writeFile(sourcePath, source);

    const uri = vscode.Uri.file(sourcePath);
    const document = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(document);

    const memberPosition = document.positionAt(
      source.indexOf("CounterValue.Amount") + "CounterValue.Am".length
    );
    const member = await waitForCompletion(
      uri,
      memberPosition,
      "record member Amount",
      (item) => completionLabel(item) === "Amount"
    );
    assert.equal(member.kind, vscode.CompletionItemKind.Field);
    assert.match(member.detail ?? "", /integer/i);
    assert.ok(member.range, "member completion has a replacement range");

    const signaturePosition = document.positionAt(
      source.indexOf("Add(1, 2)") + "Add(1, ".length
    );
    const signature = await waitForSignature(uri, signaturePosition);
    assert.match(signature.signatures[0]?.label ?? "", /Add\(/);
    assert.equal(signature.activeParameter, 1);

    const autoImportPosition = document.positionAt(
      source.indexOf("UniqueHostValue") + "UniqueHostValue".length
    );
    const autoImport = await waitForCompletion(
      uri,
      autoImportPosition,
      "safe auto-import UniqueHostValue",
      (item) => completionLabel(item) === "UniqueHostValue"
    );
    assert.ok(autoImport.additionalTextEdits?.length);
    assert.ok(
      autoImport.additionalTextEdits.some((edit) =>
        edit.newText.includes("Intellisense.Importable")
      )
    );

    await vscode.commands.executeCommand(
      "workbench.action.revertAndCloseActiveEditor"
    );
    await verifySnippetCompletion(fixtureRoot);
  } finally {
    await vscode.commands.executeCommand(
      "workbench.action.revertAndCloseActiveEditor"
    );
    await new Promise((resolve) => setTimeout(resolve, 50));
    await fs.rm(fixtureRoot, { recursive: true, force: true });
  }
}

function verifySnippetContribution(extensionPath: string): void {
  const extension = vscode.extensions.getExtension(
    "functional-pascal.functional-pascal"
  );
  assert.ok(extension);
  assert.equal(extension.extensionPath, extensionPath);
  assert.deepEqual(extension.packageJSON.contributes?.snippets, [
    { language: "fpas", path: "./snippets/fpas.json" }
  ]);
}

async function verifySnippetCompletion(fixtureRoot: string): Promise<void> {
  const snippetPath = path.join(fixtureRoot, "src", "snippet.fpas");
  const source =
    "program SnippetHost;\n\nbegin\n  if true then\n  begin\n  end\nend.\n";
  await fs.writeFile(snippetPath, source);
  const document = await vscode.workspace.openTextDocument(snippetPath);
  await vscode.window.showTextDocument(document);
  const position = document.positionAt(source.indexOf("if") + 2);
  const item = await waitForCompletion(
    document.uri,
    position,
    "If statement snippet",
    (candidate) =>
      candidate.kind === vscode.CompletionItemKind.Snippet &&
      completionLabel(candidate) === "if"
  );
  assert.equal(item.insertText instanceof vscode.SnippetString, true);
  await vscode.commands.executeCommand("workbench.action.closeActiveEditor");
}

async function waitForCompletion(
  uri: vscode.Uri,
  position: vscode.Position,
  description: string,
  predicate: (item: vscode.CompletionItem) => boolean
): Promise<vscode.CompletionItem> {
  const deadline = Date.now() + 10_000;
  let lastItems: readonly vscode.CompletionItem[] = [];
  while (Date.now() < deadline) {
    const completions = await vscode.commands.executeCommand<vscode.CompletionList>(
      "vscode.executeCompletionItemProvider",
      uri,
      position
    );
    lastItems = completions?.items ?? [];
    const item = completions?.items.find(predicate);
    if (item) {
      return item;
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  const labels = lastItems
    .slice(0, 50)
    .map((item) => `${completionLabel(item)}:${item.kind}`)
    .join(", ");
  assert.fail(`timed out waiting for ${description}; received [${labels}]`);
}

async function waitForSignature(
  uri: vscode.Uri,
  position: vscode.Position
): Promise<vscode.SignatureHelp> {
  const deadline = Date.now() + 10_000;
  while (Date.now() < deadline) {
    const signature = await vscode.commands.executeCommand<vscode.SignatureHelp>(
      "vscode.executeSignatureHelpProvider",
      uri,
      position
    );
    if (signature?.signatures.length) {
      return signature;
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  assert.fail("timed out waiting for signature help");
}

function completionLabel(item: vscode.CompletionItem): string {
  return typeof item.label === "string" ? item.label : item.label.label;
}
