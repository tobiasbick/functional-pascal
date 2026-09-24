import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";

import * as vscode from "vscode";

import { ProjectIndex, projectIndex } from "../../src/projects/index";
import { debugTargetForDocument } from "../../src/debugger/projectTarget";
import { programMainPath } from "../../src/projects/manifest";
import { pathIdentity } from "../../src/projects/pathIdentity";
import { ProjectSelector } from "../../src/workflow/project";

/** Covers TOML project ownership, cached scans, invalidation, and explicit switching. */
export async function verifyProjectIndex(): Promise<void> {
  const root = path.join(path.parse(process.cwd()).root, "workspace");
  const manifest = path.join(root, "one.fpasprj");
  const main = path.join(root, "src", "main.fpas");
  for (const value of [
    '"src/main.fpas"', "'src/main.fpas'", '"""src/main.fpas"""',
    "'''src/main.fpas'''", '"src/\\u006dain.fpas"'
  ]) {
    assert.equal(programMainPath(manifest, `[project]\nkind = "program"\nmain = ${value}\n`), main);
  }
  assert.throws(() => programMainPath(manifest, '[project]\nkind = "program"\nmain = "oops\n'));
  assert.equal(programMainPath(manifest, '[project]\nkind = "library"\nmain = "src/main.fpas"\n'), undefined);
  assert.notEqual(pathIdentity("/work/Main.fpas", "linux"), pathIdentity("/work/main.fpas", "linux"));
  assert.equal(pathIdentity("C:\\Work\\Main.fpas", "win32"), pathIdentity("c:\\work\\main.fpas", "win32"));

  const first = vscode.Uri.file(manifest);
  const second = vscode.Uri.file(path.join(root, "two.fpasprj"));
  const bad = vscode.Uri.file(path.join(root, "bad.fpasprj"));
  let scanCount = 0;
  let readCount = 0;
  const files = [first, second, bad];
  const contents = new Map([
    [first.fsPath, '[project]\nkind = "program"\nmain = "src/main.fpas"\n'],
    [second.fsPath, '[project]\nkind = "program"\nmain = "src/main.fpas"\n'],
    [bad.fsPath, 'invalid = "\n']
  ]);
  const index = new ProjectIndex(
    async () => { scanCount += 1; return files; },
    async uri => { readCount += 1; return new TextEncoder().encode(contents.get(uri.fsPath)!); },
    false
  );
  const [owners1, owners2] = await Promise.all([index.programOwners(main), index.programOwners(main)]);
  assert.deepEqual(owners1, [first.fsPath, second.fsPath]);
  assert.deepEqual(owners2, owners1);
  assert.equal(scanCount, 1);
  assert.equal(readCount, 3);
  await index.candidates();
  assert.equal(scanCount, 1);

  contents.set(second.fsPath, '[project]\nkind = "library"\n');
  files.splice(files.indexOf(bad), 1);
  index.invalidate();
  assert.deepEqual(await index.programOwners(main), [first.fsPath]);
  assert.equal(scanCount, 2);
  assert.equal(readCount, 5);

  let release!: (uris: vscode.Uri[]) => void;
  const staleScan = new Promise<vscode.Uri[]>(resolve => { release = resolve; });
  let scans = 0;
  const racing = new ProjectIndex(
    () => { scans += 1; return scans === 1 ? staleScan : Promise.resolve([second]); },
    async () => new TextEncoder().encode('[project]\nkind = "program"\nmain = "src/main.fpas"\n'),
    false
  );
  const pending = racing.candidates();
  racing.invalidate();
  release([first]);
  assert.deepEqual((await pending).map(uri => uri.fsPath), [second.fsPath]);
  assert.equal(scans, 2);
  racing.dispose();

  let releaseRead!: (contents: Uint8Array) => void;
  let beganRead!: () => void;
  const oldRead = new Promise<Uint8Array>(resolve => { releaseRead = resolve; });
  const reading = new Promise<void>(resolve => { beganRead = resolve; });
  let readAttempts = 0;
  const racingOwners = new ProjectIndex(async () => [first], () => {
    readAttempts += 1;
    if (readAttempts === 1) {
      beganRead();
      return oldRead;
    }
    return Promise.resolve(new TextEncoder().encode('[project]\nkind = "library"\n'));
  }, false);
  const staleOwnerQuery = racingOwners.programOwners(main);
  await reading;
  racingOwners.invalidate();
  const currentOwnerQuery = racingOwners.programOwners(main);
  releaseRead(new TextEncoder().encode('[project]\nkind = "program"\nmain = "src/main.fpas"\n'));
  assert.deepEqual(await Promise.all([staleOwnerQuery, currentOwnerQuery]), [[], []]);
  assert.equal(readAttempts, 2);
  racingOwners.dispose();

  const values = new Map<string, string>();
  const state = {
    get: (key: string) => values.get(key),
    update: async (key: string, value: string) => { values.set(key, value); },
    keys: () => [...values.keys()]
  } as vscode.Memento;
  let picks = 0;
  const selector = new ProjectSelector(state, index, async items => {
    picks += 1;
    return items[1];
  });
  await selector.select(first);
  assert.equal((await selector.select())?.fsPath, second.fsPath);
  assert.equal(picks, 1);
  assert.equal((await selector.resolve())?.fsPath, second.fsPath);
  assert.equal(picks, 1);
  selector.dispose();
  index.dispose();

  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  assert.ok(workspaceRoot);
  const fixture = await fs.mkdtemp(path.join(workspaceRoot, "f5-ownership-"));
  const source = path.join(fixture, "src", "main.fpas");
  const firstManifest = path.join(fixture, "first.fpasprj");
  const secondManifest = path.join(fixture, "second.fpasprj");
  try {
    await fs.mkdir(path.dirname(source));
    await fs.writeFile(source, "program Main; begin end.");
    await fs.writeFile(firstManifest,
      '[project]\nname = "first"\nkind = "program"\nmain = """src/main.fpas"""\n[sources]\ninclude = ["src/*.fpas"]\n');
    await fs.writeFile(path.join(fixture, "bad.fpasprj"), "invalid = '\n");
    projectIndex.invalidate();
    const document = { uri: vscode.Uri.file(source) } as vscode.TextDocument;
    assert.equal(await debugTargetForDocument(document), vscode.Uri.file(firstManifest).fsPath);
    await fs.writeFile(secondManifest,
      '[project]\nname = "second"\nkind = "program"\nmain = \'src/main.fpas\'\n[sources]\ninclude = ["src/*.fpas"]\n');
    projectIndex.invalidate();
    await assert.rejects(debugTargetForDocument(document), /multiple projects/u);
    await fs.writeFile(secondManifest,
      '[project]\nname = "second"\nkind = "library"\n[sources]\ninclude = ["src/*.fpas"]\n');
    projectIndex.invalidate();
    assert.equal(await debugTargetForDocument(document), vscode.Uri.file(firstManifest).fsPath);
  } finally {
    await fs.rm(fixture, { recursive: true, force: true });
    projectIndex.invalidate();
  }
}
