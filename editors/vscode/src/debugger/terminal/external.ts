/** External debug terminal lifecycle and authenticated local event bridge. */

import { randomBytes } from "node:crypto";
import net from "node:net";
import process from "node:process";

import * as vscode from "vscode";

import type { TerminalInputEvent } from "./input";

const MAX_WIRE_LINE_BYTES = 2 * 1024 * 1024;

interface DapMessage {
  readonly type?: string;
  readonly event?: string;
  readonly body?: { readonly output?: unknown; readonly exitCode?: unknown };
}

interface ClientMessage {
  readonly token?: unknown;
  readonly events?: unknown;
  readonly closed?: unknown;
}

/** Own external terminal bridges and provide launch metadata to the DAP adapter. */
export class ExternalDebugTerminalManager implements vscode.Disposable {
  private readonly sessions = new Map<string, ExternalDebugTerminalSession>();
  private readonly tracker: vscode.Disposable;

  public constructor(private readonly clientPath: string) {
    this.tracker = vscode.debug.registerDebugAdapterTrackerFactory("fpas", {
      createDebugAdapterTracker: (session) => ({
        onDidSendMessage: (message: DapMessage) => this.receive(session, message),
        onWillStopSession: () => this.stop(session.id)
      })
    });
  }

  /** Prepare the local bridge and inject its one-use DAP launch request. */
  public async prepare(session: vscode.DebugSession): Promise<void> {
    this.stop(session.id);
    const terminal = new ExternalDebugTerminalSession(session, this.clientPath);
    const launch = await terminal.listen();
    this.sessions.set(session.id, terminal);
    session.configuration.__fpasExternalTerminal = launch;
  }

  public dispose(): void {
    this.tracker.dispose();
    for (const terminal of this.sessions.values()) terminal.finish();
    this.sessions.clear();
  }

  private receive(session: vscode.DebugSession, message: DapMessage): void {
    const terminal = this.sessions.get(session.id);
    if (terminal === undefined || message.type !== "event") return;
    if (message.event === "initialized") terminal.ready();
    if (message.event === "fpas/terminalOutput" && typeof message.body?.output === "string") {
      terminal.write(message.body.output);
    }
    if (message.event === "exited" && typeof message.body?.exitCode === "number") {
      terminal.setExitCode(message.body.exitCode);
    }
    if (message.event === "terminated") this.stop(session.id);
  }

  private stop(sessionId: string): void {
    const terminal = this.sessions.get(sessionId);
    if (terminal === undefined) return;
    this.sessions.delete(sessionId);
    terminal.finish();
  }
}

class ExternalDebugTerminalSession {
  private readonly server = net.createServer((socket) => this.accept(socket));
  private readonly token = randomBytes(32).toString("hex");
  private readonly pendingEvents: TerminalInputEvent[] = [];
  private readonly pendingOutput: string[] = [];
  private socket: net.Socket | undefined;
  private readyForInput = false;
  private authenticated = false;
  private finished = false;
  private exitCode = 0;

  public constructor(
    private readonly session: vscode.DebugSession,
    private readonly clientPath: string
  ) {}

  public async listen(): Promise<Record<string, unknown>> {
    await new Promise<void>((resolve, reject) => {
      this.server.once("error", reject);
      this.server.listen(0, "127.0.0.1", () => {
        this.server.off("error", reject);
        resolve();
      });
    });
    const address = this.server.address();
    if (address === null || typeof address === "string") {
      this.finish();
      throw new Error("Cannot allocate the external FPAS debug terminal bridge.");
    }
    return {
      kind: "external",
      title: `FPAS: ${this.session.name}`,
      cwd: String(this.session.configuration.cwd),
      args: [
        process.execPath,
        this.clientPath,
        "--connect",
        `127.0.0.1:${address.port}`,
        "--token",
        this.token
      ],
      env: { ELECTRON_RUN_AS_NODE: "1" }
    };
  }

  public ready(): void {
    this.readyForInput = true;
    this.flushEvents();
  }

  public write(output: string): void {
    if (this.finished) return;
    if (!this.authenticated || this.socket === undefined) {
      this.pendingOutput.push(output);
      return;
    }
    this.send({ output });
  }

  public setExitCode(exitCode: number): void {
    this.exitCode = exitCode;
    this.send({ exitCode });
  }

  public finish(): void {
    if (this.finished) return;
    this.finished = true;
    if (this.server.listening) this.server.close();
    this.send({ exitCode: this.exitCode, finished: true });
    this.socket?.end();
    this.socket = undefined;
    this.pendingEvents.length = 0;
    this.pendingOutput.length = 0;
  }

  private accept(socket: net.Socket): void {
    if (this.finished || this.socket !== undefined) {
      socket.destroy();
      return;
    }
    this.socket = socket;
    socket.setEncoding("utf8");
    let pending = "";
    socket.on("data", (chunk: string) => {
      pending += chunk;
      if (Buffer.byteLength(pending, "utf8") > MAX_WIRE_LINE_BYTES) {
        socket.destroy(new Error("External terminal message exceeds the 2 MiB limit."));
        return;
      }
      let newline = pending.indexOf("\n");
      while (newline >= 0) {
        const line = pending.slice(0, newline);
        pending = pending.slice(newline + 1);
        this.receiveLine(socket, line);
        newline = pending.indexOf("\n");
      }
    });
    socket.once("close", () => {
      const wasAuthenticated = this.authenticated && this.socket === socket;
      if (this.socket === socket) this.socket = undefined;
      if (wasAuthenticated && !this.finished) void vscode.debug.stopDebugging(this.session);
    });
    socket.once("error", () => undefined);
  }

  private receiveLine(socket: net.Socket, line: string): void {
    let message: ClientMessage;
    try {
      message = JSON.parse(line) as ClientMessage;
    } catch {
      socket.destroy(new Error("External terminal sent invalid JSON."));
      return;
    }
    if (!this.authenticated) {
      if (message.token !== this.token) {
        this.socket = undefined;
        socket.destroy(new Error("External terminal authentication failed."));
        return;
      }
      this.authenticated = true;
      this.server.close();
      for (const output of this.pendingOutput.splice(0)) this.send({ output });
      this.flushEvents();
      return;
    }
    if (message.closed === true) {
      void vscode.debug.stopDebugging(this.session);
      return;
    }
    if (!Array.isArray(message.events)) return;
    const events = message.events as TerminalInputEvent[];
    if (!this.readyForInput) {
      this.pendingEvents.push(...events);
      return;
    }
    this.sendEvents(events);
  }

  private flushEvents(): void {
    if (!this.readyForInput || !this.authenticated || this.pendingEvents.length === 0) return;
    this.sendEvents(this.pendingEvents.splice(0));
  }

  private sendEvents(events: readonly TerminalInputEvent[]): void {
    if (events.length === 0) return;
    void this.session.customRequest("fpas/terminalInput", { events }).then(
      () => undefined,
      (error: unknown) => {
        this.send({ output: `\r\nTerminal input failed: ${String(error)}\r\n` });
      }
    );
  }

  private send(message: Record<string, unknown>): void {
    if (!this.authenticated || this.socket === undefined || this.socket.destroyed) return;
    this.socket.write(`${JSON.stringify(message)}\n`);
  }
}
