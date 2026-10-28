import assert from "node:assert/strict";

import * as vscode from "vscode";
import type { LanguageClient } from "vscode-languageclient/node";

import { LanguageClientController } from "../src/languageClient";
import type { ToolchainResolver } from "../src/toolchain";

/** Checks concurrent starts, queued stops/restarts, and recovery after startup failure. */
export async function verifyLanguageClientLifecycle(): Promise<void> {
  let created = 0;
  let stopped = 0;
  let disposed = 0;
  let release!: () => void;
  const startGate = new Promise<void>(resolve => { release = resolve; });
  const toolchain = { resolve: async () => ({
    executable: "fpas", languageServer: "fpas-lsp", standardLibrary: "stdlib", version: "0.0.1"
  }) } as Pick<ToolchainResolver, "resolve">;
  const output = { appendLine: () => undefined } as unknown as vscode.LogOutputChannel;
  const controller = new LanguageClientController(toolchain, output, (server, options) => {
    assert.deepEqual(server, {
      command: "fpas-lsp", args: [], options: {
        cwd: vscode.workspace.workspaceFolders?.find(folder => folder.uri.scheme === "file")?.uri.fsPath ?? "."
      }
    });
    assert.equal(options.initializationOptions.standardLibraryUri, vscode.Uri.file("stdlib").toString());
    created += 1;
    return {
      start: async () => { if (created === 1) await startGate; },
      stop: async () => { stopped += 1; },
      dispose: async () => { disposed += 1; },
      sendRequest: async () => { throw new Error("unexpected request in lifecycle test"); }
    } as Pick<LanguageClient, "start" | "stop" | "dispose" | "sendRequest">;
  });
  const first = controller.start();
  const second = controller.start();
  const stop = controller.stop();
  release();
  assert.deepEqual(await Promise.all([first, second]), ["fpas-lsp", "fpas-lsp"]);
  await stop;
  assert.equal(created, 1);
  assert.equal(stopped, 1);
  assert.equal(disposed, 1);

  await Promise.all([controller.start(), controller.restart(), controller.stop()]);
  assert.equal(created, 3);
  assert.equal(stopped, 3);
  assert.equal(disposed, 3);

  let attempts = 0;
  const recovery = new LanguageClientController(toolchain, output, () => ({
    start: async () => { if (++attempts === 1) throw new Error("startup failed"); },
    stop: async () => undefined,
    dispose: async () => undefined,
    sendRequest: async () => { throw new Error("unexpected request in lifecycle test"); }
  }) as Pick<LanguageClient, "start" | "stop" | "dispose" | "sendRequest">);
  await assert.rejects(recovery.start(), /startup failed/u);
  assert.equal(await recovery.start(), "fpas-lsp");
  await recovery.stop();
  assert.equal(attempts, 2);
}
