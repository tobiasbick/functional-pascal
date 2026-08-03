import assert from "node:assert/strict";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";
import { pathToFileURL } from "node:url";

import AdmZip from "adm-zip";

function frame(message) {
  const body = Buffer.from(JSON.stringify(message), "utf8");
  return Buffer.concat([
    Buffer.from(`Content-Length: ${body.length}\r\n\r\n`, "ascii"),
    body
  ]);
}

function positionAt(source, offset) {
  const before = source.slice(0, offset);
  const lines = before.split("\n");
  return {
    line: lines.length - 1,
    character: lines.at(-1).length
  };
}

function responseReader(stdout, child) {
  let buffer = Buffer.alloc(0);
  const received = new Map();
  const waiters = new Map();
  const receivedNotifications = new Map();
  const notificationWaiters = new Map();

  function rejectWaiters(error) {
    for (const waiter of waiters.values()) {
      clearTimeout(waiter.timeout);
      waiter.reject(error);
    }
    waiters.clear();
  }

  function dispatch(message) {
    if (message.id === undefined) {
      const waiting = notificationWaiters.get(message.method) ?? [];
      const waiterIndex = waiting.findIndex((waiter) => waiter.predicate(message));
      if (waiterIndex >= 0) {
        const [waiter] = waiting.splice(waiterIndex, 1);
        clearTimeout(waiter.timeout);
        waiter.resolve(message);
        return;
      }
      const queued = receivedNotifications.get(message.method) ?? [];
      queued.push(message);
      receivedNotifications.set(message.method, queued);
      return;
    }
    const waiter = waiters.get(message.id);
    if (waiter === undefined) {
      received.set(message.id, message);
      return;
    }
    clearTimeout(waiter.timeout);
    waiters.delete(message.id);
    waiter.resolve(message);
  }

  function parseAvailableFrames() {
    while (buffer.length > 0) {
      const headerEnd = buffer.indexOf("\r\n\r\n", 0, "ascii");
      if (headerEnd === -1) {
        return;
      }
      const header = buffer.subarray(0, headerEnd).toString("ascii");
      const match = /^Content-Length:\s*(\d+)$/imu.exec(header);
      assert.ok(match, `server stdout contains an invalid LSP header: ${header}`);
      const bodyStart = headerEnd + 4;
      const bodyEnd = bodyStart + Number.parseInt(match[1], 10);
      if (bodyEnd > buffer.length) {
        return;
      }
      dispatch(JSON.parse(buffer.subarray(bodyStart, bodyEnd).toString("utf8")));
      buffer = buffer.subarray(bodyEnd);
    }
  }

  stdout.on("data", (chunk) => {
    buffer = Buffer.concat([buffer, chunk]);
    try {
      parseAvailableFrames();
    } catch (error) {
      rejectWaiters(error);
    }
  });
  stdout.on("end", () => {
    if (buffer.length > 0) {
      rejectWaiters(
        new Error("server stdout ended with an incomplete LSP frame")
      );
    }
  });
  child.on("error", rejectWaiters);

  return {
    response(id) {
      const existing = received.get(id);
      if (existing !== undefined) {
        received.delete(id);
        return Promise.resolve(existing);
      }
      return new Promise((resolve, reject) => {
        const timeout = setTimeout(() => {
          waiters.delete(id);
          reject(new Error(`timed out waiting for LSP response ${id}`));
        }, 15_000);
        waiters.set(id, { resolve, reject, timeout });
      });
    },
    notification(method, predicate) {
      const queued = receivedNotifications.get(method) ?? [];
      const notificationIndex = queued.findIndex(predicate);
      if (notificationIndex >= 0) {
        const [notification] = queued.splice(notificationIndex, 1);
        return Promise.resolve(notification);
      }
      return new Promise((resolve, reject) => {
        const timeout = setTimeout(() => {
          const waiting = notificationWaiters.get(method) ?? [];
          notificationWaiters.set(
            method,
            waiting.filter((waiter) => waiter.resolve !== resolve)
          );
          reject(new Error(`timed out waiting for LSP notification ${method}`));
        }, 15_000);
        const waiting = notificationWaiters.get(method) ?? [];
        waiting.push({ predicate, resolve, reject, timeout });
        notificationWaiters.set(method, waiting);
      });
    }
  };
}

/** Extracts and starts the packaged server for an initialize/shutdown smoke test. */
export async function smokePackagedServer({
  vsixPath,
  hostTarget,
  executableName,
  platform
}) {
  const temporaryRoot = mkdtempSync(path.join(os.tmpdir(), "fpas-vsix-smoke-"));
  let child;
  let exited;
  try {
    new AdmZip(vsixPath).extractAllTo(temporaryRoot, true);
    const executable = path.join(
      temporaryRoot,
      "extension",
      "server",
      hostTarget,
      executableName
    );
    const standardLibrary = path.join(
      temporaryRoot,
      "extension",
      "standard-library"
    );
    const workspace = path.join(temporaryRoot, "external-project");
    const source = path.join(workspace, "main.fpas");
    mkdirSync(workspace, { recursive: true });
    writeFileSync(
      path.join(workspace, "external.fpasprj"),
      `[project]\nname = "external"\nkind = "program"\nmain = "main.fpas"\n\n[sources]\ninclude = ["main.fpas"]\n`
    );
    const sourceText =
      "program External;\n\nuses Std.Tui;\n\nbegin\n  var Palette: TuiPalette := TuiPalette.Default();\n  var Selected: TuiPalette := Palette\nend.\n";
    writeFileSync(source, sourceText);
    if (platform !== "win32") {
      chmodSync(executable, 0o755);
    }

    child = spawn(executable, [], {
      stdio: ["pipe", "pipe", "pipe"],
      windowsHide: true
    });
    let stderr = "";
    child.stderr.setEncoding("utf8");
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    exited = new Promise((resolve) => {
      child.once("close", (code) => resolve(code));
    });
    const reader = responseReader(child.stdout, child);

    child.stdin.write(frame({
      jsonrpc: "2.0",
      id: 1,
      method: "initialize",
      params: {
        processId: null,
        capabilities: {
          workspace: {
            workspaceEdit: { documentChanges: true }
          }
        },
        workspaceFolders: [
          { uri: pathToFileURL(workspace).href, name: "external-project" }
        ],
        initializationOptions: {
          standardLibraryUri: pathToFileURL(standardLibrary).href
        }
      }
    }));
    const initialize = await reader.response(1);
    assert.ok(
      initialize?.result?.capabilities,
      `packaged server initialized: ${JSON.stringify(initialize)}`
    );
    assert.equal(initialize.result.capabilities.referencesProvider, true);
    assert.equal(
      initialize.result.capabilities.renameProvider?.prepareProvider,
      true
    );
    child.stdin.write(frame({
      jsonrpc: "2.0",
      method: "initialized",
      params: {}
    }));
    const sourceUri = pathToFileURL(source).href;
    const diagnostics = reader.notification(
      "textDocument/publishDiagnostics",
      (message) => message.params?.uri === sourceUri
    );
    child.stdin.write(frame({
      jsonrpc: "2.0",
      method: "textDocument/didOpen",
      params: {
        textDocument: {
          uri: sourceUri,
          languageId: "fpas",
          version: 1,
          text: sourceText
        }
      }
    }));
    const published = await diagnostics;
    assert.deepEqual(
      published.params.diagnostics,
      [],
      `packaged server resolves bundled Std.Tui: ${JSON.stringify(published)}`
    );
    const paletteUse = sourceText.lastIndexOf("Palette");
    child.stdin.write(frame({
      jsonrpc: "2.0",
      id: 2,
      method: "textDocument/references",
      params: {
        textDocument: { uri: sourceUri },
        position: positionAt(sourceText, paletteUse),
        context: { includeDeclaration: true }
      }
    }));
    const references = await reader.response(2);
    assert.equal(
      references?.result?.length,
      2,
      `packaged server finds local references: ${JSON.stringify(references)}`
    );
    child.stdin.write(frame({
      jsonrpc: "2.0",
      id: 3,
      method: "textDocument/rename",
      params: {
        textDocument: { uri: sourceUri },
        position: positionAt(sourceText, paletteUse),
        newName: "ThemePalette"
      }
    }));
    const rename = await reader.response(3);
    const renameChanges = rename?.result?.documentChanges ?? [];
    assert.deepEqual(
      renameChanges.map((change) => change.edits.length),
      [2],
      `packaged server returns rename edits: ${JSON.stringify(rename)}`
    );
    child.stdin.write(frame({
      jsonrpc: "2.0",
      id: 4,
      method: "shutdown",
      params: null
    }));
    const shutdown = await reader.response(4);
    assert.equal(
      shutdown?.result,
      null,
      `packaged server shut down cleanly: ${JSON.stringify(shutdown)}`
    );
    child.stdin.end(frame({
      jsonrpc: "2.0",
      method: "exit",
      params: null
    }));
    const exitCode = await exited;
    assert.equal(exitCode, 0, `packaged server exited with ${exitCode}: ${stderr}`);
    console.log("Packaged LSP diagnostics, references, rename, and lifecycle verification passed.");
  } finally {
    if (child !== undefined && child.exitCode === null) {
      child.kill();
      await exited;
    }
    rmSync(temporaryRoot, { recursive: true, force: true });
  }
}
