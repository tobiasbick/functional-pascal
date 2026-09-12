/** Integrated terminal lifecycle for Functional Pascal debug sessions. */

import path from "node:path";

import * as vscode from "vscode";

import { TerminalInputDecoder, type TerminalInputEvent } from "./input";

interface DapMessage {
  readonly type?: string;
  readonly event?: string;
  readonly body?: { readonly output?: unknown; readonly exitCode?: unknown };
}

/** Own debugger pseudoterminals and route custom DAP terminal events. */
export class DebugTerminalManager implements vscode.Disposable {
  private readonly terminals = new Map<string, DebugTerminalPseudoterminal>();
  private readonly tracker: vscode.Disposable;

  public constructor() {
    this.tracker = vscode.debug.registerDebugAdapterTrackerFactory("fpas", {
      createDebugAdapterTracker: (session) => ({
        onWillStartSession: () => this.start(session),
        onDidSendMessage: (message: DapMessage) => this.receive(session, message),
        onWillStopSession: () => this.stop(session.id)
      })
    });
  }

  public dispose(): void {
    this.tracker.dispose();
    for (const terminal of this.terminals.values()) terminal.finish();
    this.terminals.clear();
  }

  private start(session: vscode.DebugSession): void {
    if (session.configuration.console !== "integratedTerminal") return;
    const pty = new DebugTerminalPseudoterminal(session);
    const target = path.basename(String(session.configuration.program));
    const terminal = vscode.window.createTerminal({ name: `FPAS: ${target}`, pty });
    pty.attach(terminal);
    this.terminals.set(session.id, pty);
    terminal.show(false);
  }

  private receive(session: vscode.DebugSession, message: DapMessage): void {
    const terminal = this.terminals.get(session.id);
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
    const terminal = this.terminals.get(sessionId);
    if (terminal === undefined) return;
    this.terminals.delete(sessionId);
    terminal.finish();
  }
}

class DebugTerminalPseudoterminal implements vscode.Pseudoterminal {
  private readonly writeEmitter = new vscode.EventEmitter<string>();
  private readonly closeEmitter = new vscode.EventEmitter<number | void>();
  private readonly decoder = new TerminalInputDecoder();
  private readonly pending: TerminalInputEvent[] = [];
  private terminal: vscode.Terminal | undefined;
  private connected = false;
  private finished = false;
  private pollPending = false;
  private pollTimer: ReturnType<typeof setInterval> | undefined;
  private exitCode = 0;

  public constructor(private readonly session: vscode.DebugSession) {}

  public readonly onDidWrite = this.writeEmitter.event;
  public readonly onDidClose = this.closeEmitter.event;

  public attach(terminal: vscode.Terminal): void {
    this.terminal = terminal;
  }

  public open(dimensions: vscode.TerminalDimensions | undefined): void {
    if (dimensions !== undefined) this.resize(dimensions);
  }

  public close(): void {
    if (!this.finished) void vscode.debug.stopDebugging(this.session);
  }

  public handleInput(data: string): void {
    this.send(this.decoder.feed(data));
  }

  public setDimensions(dimensions: vscode.TerminalDimensions): void {
    this.resize(dimensions);
  }

  public ready(): void {
    if (this.finished || this.connected) return;
    this.connected = true;
    this.pollTimer = setInterval(() => this.poll(), 30);
    if (this.pending.length > 0) {
      const events = this.pending.splice(0);
      this.send(events);
    }
  }

  public write(output: string): void {
    if (!this.finished) this.writeEmitter.fire(output.replace(/\r?\n/gu, "\r\n"));
  }

  public setExitCode(exitCode: number): void {
    this.exitCode = exitCode;
  }

  public finish(): void {
    if (this.finished) return;
    this.finished = true;
    this.connected = false;
    if (this.pollTimer !== undefined) clearInterval(this.pollTimer);
    this.pollTimer = undefined;
    this.closeEmitter.fire(this.exitCode);
    this.terminal = undefined;
    this.disposeEmitters();
  }

  private resize(dimensions: vscode.TerminalDimensions): void {
    this.send([{ kind: "resize", width: dimensions.columns, height: dimensions.rows }]);
  }

  private send(events: readonly TerminalInputEvent[]): void {
    if (events.length === 0) return;
    if (this.finished) return;
    if (!this.connected) {
      this.pending.push(...events);
      return;
    }
    void this.session.customRequest("fpas/terminalInput", { events }).then(
      () => undefined,
      (error: unknown) => {
        if (this.connected) {
          this.writeEmitter.fire(`\r\nTerminal input failed: ${String(error)}\r\n`);
        }
      }
    );
  }

  private poll(): void {
    if (!this.connected || this.pollPending) return;
    this.pollPending = true;
    void this.session.customRequest("fpas/terminalPoll").then(
      () => {
        this.pollPending = false;
      },
      () => {
        this.pollPending = false;
      }
    );
  }

  private disposeEmitters(): void {
    this.writeEmitter.dispose();
    this.closeEmitter.dispose();
  }
}
