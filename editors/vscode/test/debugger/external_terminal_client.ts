import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { once } from "node:events";
import net from "node:net";
import os from "node:os";
import path from "node:path";

import * as vscode from "vscode";

import { ExternalDebugTerminalManager } from "../../src/debugger/terminal/external";

/** Checks that external terminal output drains before its process exits. */
export async function verifyExternalTerminalClient(extensionPath: string): Promise<void> {
  const clientPath = path.join(extensionPath, "out", "src", "debugger", "terminal", "externalClient.js");
  const payload = "z".repeat(64 * 1024);
  const server = net.createServer();
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const address = server.address();
  assert.ok(address && typeof address !== "string");
  let sent!: Promise<void>;
  server.on("connection", socket => {
    let handshake = "";
    socket.on("data", chunk => {
      if (sent !== undefined) return;
      handshake += chunk.toString("utf8");
      const newline = handshake.indexOf("\n");
      if (newline < 0) return;
      const message = JSON.parse(handshake.slice(0, newline)) as { token: string };
      assert.equal(message.token, "review-token");
      sent = (async () => {
        for (let index = 0; index < 32; index += 1) {
          if (!socket.write(`${JSON.stringify({ output: payload })}\n`)) await once(socket, "drain");
        }
        socket.end(`${JSON.stringify({ exitCode: 17, finished: true })}\n`);
      })();
    });
  });
  const child = spawn(process.execPath,
    [clientPath, "--connect", `127.0.0.1:${address.port}`, "--token", "review-token"],
    { env: { ...process.env, ELECTRON_RUN_AS_NODE: "1" }, stdio: ["pipe", "pipe", "pipe"] });
  const output: Buffer[] = [];
  const errors: Buffer[] = [];
  child.stdout.on("data", chunk => output.push(chunk as Buffer));
  child.stderr.on("data", chunk => errors.push(chunk as Buffer));
  const timer = setTimeout(() => child.kill(), 15_000);
  try {
    const [code] = await once(child, "close") as [number | null];
    await sent;
    assert.equal(code, 17, Buffer.concat(errors).toString("utf8"));
    assert.equal(Buffer.concat(output).toString("utf8"), payload.repeat(32));
  } finally {
    clearTimeout(timer);
    server.close();
  }

  const manager = new ExternalDebugTerminalManager(clientPath);
  const session = { id: "rejected-terminal-client", name: "Review",
    configuration: { cwd: os.tmpdir() }, customRequest: async () => undefined
  } as unknown as vscode.DebugSession;
  try {
    await manager.prepare(session);
    const launch = session.configuration.__fpasExternalTerminal as { args: string[] };
    const endpoint = launch.args[launch.args.indexOf("--connect") + 1];
    const rejected = net.createConnection({ port: Number(endpoint.split(":")[1]), host: "127.0.0.1" });
    rejected.on("error", () => undefined);
    await once(rejected, "connect");
    const closed = new Promise<void>(resolve => rejected.once("close", () => resolve()));
    rejected.write(`${JSON.stringify({ token: "wrong-token" })}\n`);
    await closed;
    const state = manager as unknown as { sessions: Map<string, { authenticated: boolean }> };
    assert.equal(state.sessions.get(session.id)?.authenticated, false);
  } finally {
    manager.dispose();
  }

  const malformedServer = net.createServer(socket => {
    socket.once("data", () => socket.end("not JSON\n"));
  });
  malformedServer.listen(0, "127.0.0.1");
  await once(malformedServer, "listening");
  const malformedAddress = malformedServer.address();
  assert.ok(malformedAddress && typeof malformedAddress !== "string");
  const malformedChild = spawn(process.execPath,
    [clientPath, "--connect", `127.0.0.1:${malformedAddress.port}`, "--token", "review-token"],
    { env: { ...process.env, ELECTRON_RUN_AS_NODE: "1" }, stdio: ["pipe", "pipe", "pipe"] });
  const malformedErrors: Buffer[] = [];
  malformedChild.stderr.on("data", chunk => malformedErrors.push(chunk as Buffer));
  const malformedTimer = setTimeout(() => malformedChild.kill(), 15_000);
  try {
    const [code] = await once(malformedChild, "close") as [number | null];
    assert.equal(code, 1);
    assert.match(Buffer.concat(malformedErrors).toString("utf8"), /received invalid JSON/u);
  } finally {
    clearTimeout(malformedTimer);
    malformedServer.close();
  }
}
