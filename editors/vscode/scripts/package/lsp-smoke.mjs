import assert from "node:assert/strict";
import { chmodSync, mkdtempSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";

import AdmZip from "adm-zip";

function frame(message) {
  const body = Buffer.from(JSON.stringify(message), "utf8");
  return Buffer.concat([
    Buffer.from(`Content-Length: ${body.length}\r\n\r\n`, "ascii"),
    body
  ]);
}

function responseReader(stdout, child) {
  let buffer = Buffer.alloc(0);
  const received = new Map();
  const waiters = new Map();

  function rejectWaiters(error) {
    for (const waiter of waiters.values()) {
      clearTimeout(waiter.timeout);
      waiter.reject(error);
    }
    waiters.clear();
  }

  function dispatch(message) {
    if (message.id === undefined) {
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
      params: { processId: null, capabilities: {} }
    }));
    const initialize = await reader.response(1);
    assert.ok(
      initialize?.result?.capabilities,
      `packaged server initialized: ${JSON.stringify(initialize)}`
    );
    child.stdin.write(frame({
      jsonrpc: "2.0",
      method: "initialized",
      params: {}
    }));
    child.stdin.write(frame({
      jsonrpc: "2.0",
      id: 2,
      method: "shutdown",
      params: null
    }));
    const shutdown = await reader.response(2);
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
    console.log("Packaged LSP initialize/shutdown verification passed.");
  } finally {
    if (child !== undefined && child.exitCode === null) {
      child.kill();
      await exited;
    }
    rmSync(temporaryRoot, { recursive: true, force: true });
  }
}
