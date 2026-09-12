/** Project-aware zero-configuration debugger target discovery. */

import path from "node:path";

import * as vscode from "vscode";

const MANIFEST_EXCLUDE_GLOB =
  "**/{.git,target,node_modules,.vscode-test,dist,out}/**";

/** Resolve an active source or manifest to the executable target F5 should debug. */
export async function debugTargetForDocument(
  document: vscode.TextDocument
): Promise<string | undefined> {
  const extension = path.extname(document.uri.fsPath).toLocaleLowerCase();
  if (extension === ".fpasprj" || extension === ".fpasworkspace") {
    return document.uri.fsPath;
  }
  if (extension !== ".fpas") {
    return undefined;
  }

  const manifests = await vscode.workspace.findFiles(
    "**/*.fpasprj",
    MANIFEST_EXCLUDE_GLOB
  );
  const owners: string[] = [];
  for (const manifest of manifests) {
    const contents = new TextDecoder().decode(
      await vscode.workspace.fs.readFile(manifest)
    );
    const main = programMainPath(manifest.fsPath, contents);
    if (main !== undefined && samePath(main, document.uri.fsPath)) {
      owners.push(manifest.fsPath);
    }
  }
  if (owners.length > 1) {
    throw new Error(
      `Functional Pascal source \`${document.uri.fsPath}\` is the main file of multiple projects: ${owners.join(", ")}.`
    );
  }
  return owners[0] ?? document.uri.fsPath;
}

/** Return the absolute program main declared by one project manifest. */
export function programMainPath(
  manifestPath: string,
  contents: string
): string | undefined {
  let section = "";
  let kind: string | undefined;
  let main: string | undefined;
  for (const rawLine of contents.split(/\r?\n/u)) {
    const line = stripTomlComment(rawLine).trim();
    const sectionMatch = /^\[([^\]]+)\]$/u.exec(line);
    if (sectionMatch !== null) {
      section = sectionMatch[1].trim().toLocaleLowerCase();
      continue;
    }
    if (section !== "project") {
      continue;
    }
    const assignment = /^([A-Za-z0-9_-]+)\s*=\s*(.+)$/u.exec(line);
    if (assignment === null) {
      continue;
    }
    const value = tomlString(assignment[2].trim());
    if (assignment[1].toLocaleLowerCase() === "kind") {
      kind = value?.toLocaleLowerCase();
    } else if (assignment[1].toLocaleLowerCase() === "main") {
      main = value;
    }
  }
  if (kind !== "program" || main === undefined) {
    return undefined;
  }
  return path.resolve(path.dirname(manifestPath), main);
}

function samePath(left: string, right: string): boolean {
  const normalizedLeft = path.normalize(left);
  const normalizedRight = path.normalize(right);
  return process.platform === "win32"
    ? normalizedLeft.toLocaleLowerCase() === normalizedRight.toLocaleLowerCase()
    : normalizedLeft === normalizedRight;
}

function stripTomlComment(line: string): string {
  let quote: "\"" | "'" | undefined;
  let escaped = false;
  for (let index = 0; index < line.length; index += 1) {
    const character = line[index];
    if (quote === "\"" && character === "\\" && !escaped) {
      escaped = true;
      continue;
    }
    if ((character === "\"" || character === "'") && !escaped) {
      quote = quote === character ? undefined : quote ?? character;
    } else if (character === "#" && quote === undefined) {
      return line.slice(0, index);
    }
    escaped = false;
  }
  return line;
}

function tomlString(value: string): string | undefined {
  if (value.startsWith("'") && value.endsWith("'")) {
    return value.slice(1, -1);
  }
  if (!value.startsWith("\"") || !value.endsWith("\"")) {
    return undefined;
  }
  try {
    return JSON.parse(value) as string;
  } catch {
    return undefined;
  }
}
