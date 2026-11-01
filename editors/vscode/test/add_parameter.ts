import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import * as vscode from "vscode";
import { ADD_PARAMETER_COMMAND, type RefactorResponse } from "../src/sourceTools/refactor";

/** Real command-to-LSP-to-buffer application retains dirty changes, versions and undo. */
export async function verifyAddParameter(): Promise<void> {
  assert.ok((await vscode.commands.getCommands(true)).includes(ADD_PARAMETER_COMMAND));
  const workspace = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  assert.ok(workspace);
  const root = await fs.mkdtemp(path.join(workspace, ".parameter-tools-"));
  const savedUnit = "unit Parameter.Api; public function Answer(): integer; begin return 42; end function; end unit;";
  const savedMain = "program Main; uses Parameter.Api as Api; begin discard '😀'; discard Api.Answer(); end.";
  const documents: vscode.TextDocument[] = [];
  try {
    const manifest = path.join(root, "app.fpasprj");
    await fs.writeFile(manifest, "[project]\nname='parameter'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n");
    await fs.writeFile(path.join(root, "api.fpas"), savedUnit);
    await fs.writeFile(path.join(root, "main.fpas"), savedMain);
    const unit = await vscode.workspace.openTextDocument(path.join(root, "api.fpas"));
    const main = await vscode.workspace.openTextDocument(path.join(root, "main.fpas"));
    documents.push(unit, main);
    const dirtyUnit = savedUnit.replace("42", "73");
    const dirtyMain = savedMain.replace("discard Api.Answer();", "discard Api.Answer(); discard Api.Answer();");
    const edit = new vscode.WorkspaceEdit();
    edit.replace(unit.uri, new vscode.Range(unit.positionAt(0), unit.positionAt(savedUnit.length)), dirtyUnit);
    edit.replace(main.uri, new vscode.Range(main.positionAt(0), main.positionAt(savedMain.length)), dirtyMain);
    assert.equal(await vscode.workspace.applyEdit(edit), true);
    await vscode.window.showTextDocument(main);
    const args = { uri: main.uri, position: main.positionAt(main.getText().indexOf("Answer")), context: vscode.Uri.file(manifest),
      name: "Extra", type: "integer", mode: "value", argument: "9" };
    const preview = await vscode.commands.executeCommand<RefactorResponse & { previewUri: vscode.Uri }>(ADD_PARAMETER_COMMAND, { ...args, previewOnly: true });
    assert.equal(preview?.status, "prepared", JSON.stringify(preview));
    assert.ok(preview?.previewUri);
    documents.push(await vscode.workspace.openTextDocument(preview.previewUri));
    assert.equal(main.getText(), dirtyMain, "preview does not apply edits");
    const applied = await vscode.commands.executeCommand<RefactorResponse & { previewUri: vscode.Uri }>(ADD_PARAMETER_COMMAND, args);
    assert.equal(applied?.status, "applied", JSON.stringify(applied));
    assert.ok(applied?.previewUri);
    documents.push(await vscode.workspace.openTextDocument(applied.previewUri));
    assert.equal(unit.getText(), dirtyUnit.replace("Answer()", "Answer(Extra: integer)"));
    assert.equal(main.getText(), dirtyMain.replaceAll("Api.Answer()", "Api.Answer(9)"));
    assert.ok(main.isDirty && unit.isDirty, "application retains dirty buffers");
    assert.equal(await fs.readFile(path.join(root, "main.fpas"), "utf8"), savedMain);
    assert.equal(await fs.readFile(path.join(root, "api.fpas"), "utf8"), savedUnit);
    await vscode.window.showTextDocument(main);
    await vscode.commands.executeCommand("undo");
    assert.equal(main.getText(), dirtyMain, "undo restores the dirty caller");
    assert.equal(unit.getText(), dirtyUnit, "workspace edit undo restores the dirty declaration");
    const refused = await vscode.commands.executeCommand<RefactorResponse & { previewUri: vscode.Uri }>(ADD_PARAMETER_COMMAND, { ...args, type: "MissingType" });
    assert.ok(refused && refused.status !== "prepared" && refused.status !== "applied");
    documents.push(await vscode.workspace.openTextDocument(refused.previewUri));
    assert.equal(main.getText(), dirtyMain);
    assert.equal(unit.getText(), dirtyUnit);
  } finally {
    for (const document of documents.reverse()) {
      if (!document.isClosed) {
        await vscode.window.showTextDocument(document);
        await vscode.commands.executeCommand("workbench.action.revertAndCloseActiveEditor");
      }
    }
    await fs.rm(root, { recursive: true, force: true, maxRetries: 20, retryDelay: 100 });
  }
}
