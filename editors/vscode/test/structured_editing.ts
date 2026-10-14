import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import * as vscode from "vscode";
import { PUBLIC_SIGNATURES_COMMAND } from "../src/publicSignatures";

/** Applies real rename edits and checks the live read-only signature view in VS Code. */
export async function verifyStructuredEditing(): Promise<void> {
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  assert.ok(workspaceRoot);
  const root = await fs.mkdtemp(path.join(workspaceRoot, ".structured-editing-"));
  const unitSource = "unit Structured.Api;\n\nconst Hidden: integer := 42;\n\npublic function Answer(): integer;\nbegin\n  return Hidden;\nend function;\nend unit;\n";
  const mainSource = "program StructuredMain;\n\nuses Structured.Api as Api;\n\nbegin\n  // Answer stays in comments and strings.\n  const Text: string := 'Answer';\n  const Value: integer := Api.Answer();\n  discard Text;\n  discard Value;\nend.\n";
  const documents: vscode.TextDocument[] = [];
  try {
    await fs.writeFile(path.join(root, "demo.fpasprj"), '[project]\nname = "structured-editing"\nkind = "program"\nmain = "main.fpas"\n\n[sources]\ninclude = ["*.fpas"]\n');
    await fs.writeFile(path.join(root, "api.fpas"), unitSource);
    await fs.writeFile(path.join(root, "main.fpas"), mainSource);
    const unit = await vscode.workspace.openTextDocument(path.join(root, "api.fpas"));
    const main = await vscode.workspace.openTextDocument(path.join(root, "main.fpas"));
    documents.push(unit, main);
    await vscode.window.showTextDocument(main);
    const originalUse = main.positionAt(main.getText().indexOf("Api.Answer") + 4);
    await waitFor(async () => {
      const locations = await vscode.commands.executeCommand<Array<vscode.Location | vscode.LocationLink>>(
        "vscode.executeDefinitionProvider", main.uri, originalUse
      );
      return (locations?.length ?? 0) === 1;
    });
    const rename = await vscode.commands.executeCommand<vscode.WorkspaceEdit>(
      "vscode.executeDocumentRenameProvider", main.uri, originalUse, "ReadValue"
    );
    assert.ok(rename);
    assert.equal(await vscode.workspace.applyEdit(rename), true);
    assert.equal(unit.getText(), unitSource.replace("function Answer()", "function ReadValue()"));
    assert.equal(main.getText(), mainSource.replace("Api.Answer()", "Api.ReadValue()"));
    const renamedUse = main.positionAt(main.getText().indexOf("Api.ReadValue") + 4);
    await waitFor(async () => {
      const locations = await vscode.commands.executeCommand<Array<vscode.Location | vscode.LocationLink>>(
        "vscode.executeDefinitionProvider", main.uri, renamedUse
      );
      const location = locations?.[0];
      return locations?.length === 1 && location !== undefined &&
        ("uri" in location ? location.uri : location.targetUri).toString() === unit.uri.toString();
    });
    for (const document of [unit, main]) {
      const canonical = document.getText();
      const unformatted = canonical.replace(": integer", ":   integer");
      assert.notEqual(unformatted, canonical);
      const perturb = new vscode.WorkspaceEdit();
      perturb.replace(document.uri, new vscode.Range(document.positionAt(0), document.positionAt(canonical.length)), unformatted);
      assert.equal(await vscode.workspace.applyEdit(perturb), true);
      const edits = await vscode.commands.executeCommand<vscode.TextEdit[]>(
        "vscode.executeFormatDocumentProvider", document.uri, { tabSize: 2, insertSpaces: true }
      );
      assert.ok(edits?.length, "formatter returns edits for the noncanonical control");
      const formatting = new vscode.WorkspaceEdit();
      formatting.set(document.uri, edits);
      assert.equal(await vscode.workspace.applyEdit(formatting), true);
      assert.equal(document.getText(), canonical, "canonical formatting retains the renamed source and unrelated text");
      const stable = await vscode.commands.executeCommand<vscode.TextEdit[]>(
        "vscode.executeFormatDocumentProvider", document.uri, { tabSize: 2, insertSpaces: true }
      );
      assert.equal(stable?.length ?? 0, 0, "canonical source needs no further edits");
    }

    await vscode.window.showTextDocument(unit);
    const view = await vscode.commands.executeCommand<vscode.TextDocument>(PUBLIC_SIGNATURES_COMMAND);
    assert.ok(view);
    documents.push(view);
    assert.equal(view.uri.scheme, "fpas-public-signatures");
    assert.equal(view.getText(), "unit Structured.Api;\n\npublic function ReadValue(): integer;\n\nend unit;\n");
    assert.equal(vscode.window.activeTextEditor?.document.uri.toString(), view.uri.toString());
    const beforeTyping = view.getText();
    await vscode.commands.executeCommand("type", { text: "changed" });
    assert.equal(view.getText(), beforeTyping, "signature view rejects normal editor input");
    assert.equal(view.isDirty, false);

    const addExport = new vscode.WorkspaceEdit();
    addExport.insert(unit.uri, unit.positionAt(unit.getText().indexOf("const Hidden")), "public const Added: integer := 2;\n\n");
    assert.equal(await vscode.workspace.applyEdit(addExport), true);
    await waitFor(async () => view.getText().includes("public const Added: integer;"));
    assert.ok(!view.getText().includes("Hidden"));
    assert.equal(await fs.readFile(path.join(root, "api.fpas"), "utf8"), unitSource);
    assert.equal(await fs.readFile(path.join(root, "main.fpas"), "utf8"), mainSource);

    const breakSource = new vscode.WorkspaceEdit();
    breakSource.delete(unit.uri, new vscode.Range(unit.positionAt(unit.getText().indexOf("end unit;")), unit.positionAt(unit.getText().length)));
    assert.equal(await vscode.workspace.applyEdit(breakSource), true);
    await waitFor(async () => view.getText().includes("Public signatures unavailable"));
    assert.ok(!view.getText().includes("public function ReadValue"), "incomplete syntax does not retain a stale signature view");
  } finally {
    for (const document of documents.reverse()) {
      if (!document.isClosed) {
        await vscode.window.showTextDocument(document);
        await vscode.commands.executeCommand("workbench.action.revertAndCloseActiveEditor");
      }
    }
    assert.ok(root.startsWith(`${workspaceRoot}${path.sep}.structured-editing-`));
    // The host retains directory handles; the runner removes this fixture after shutdown.
  }
}

async function waitFor(predicate: () => Promise<boolean>): Promise<void> {
  const deadline = Date.now() + 10_000;
  while (Date.now() < deadline) {
    if (await predicate()) return;
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  assert.fail("structured-editing result did not arrive");
}
