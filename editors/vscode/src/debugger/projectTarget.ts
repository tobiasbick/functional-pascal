/** Project-aware zero-configuration debugger target discovery. */

import path from "node:path";

import * as vscode from "vscode";

import { projectIndex } from "../projects/index";

/** Resolve an active source or manifest to the executable target F5 should debug. */
export async function debugTargetForDocument(
  document: vscode.TextDocument
): Promise<string | undefined> {
  const extension = path.extname(document.uri.fsPath).toLowerCase();
  if (extension === ".fpasprj" || extension === ".fpasworkspace") {
    return document.uri.fsPath;
  }
  if (extension !== ".fpas") return undefined;

  const owners = await projectIndex.programOwners(document.uri.fsPath);
  if (owners.length > 1) {
    throw new Error(
      `Functional Pascal source \`${document.uri.fsPath}\` is the main file of multiple projects: ${owners.join(", ")}.`
    );
  }
  return owners[0] ?? document.uri.fsPath;
}
