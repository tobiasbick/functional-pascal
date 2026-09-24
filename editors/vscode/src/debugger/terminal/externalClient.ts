/** Native terminal frontend for one externally hosted FPAS debug session. */

import net from "node:net";
import process from "node:process";
import { StringDecoder } from "node:string_decoder";

import { TimedTerminalInput } from "./timedInput";

interface HostMessage {
  readonly output?: unknown;
  readonly exitCode?: unknown;
  readonly finished?: unknown;
}

const MAX_WIRE_LINE_BYTES = 2 * 1024 * 1024;
const MAX_INPUT_BUFFER_BYTES = 4 * 1024 * 1024;

let exitCode = 0;
let rawMode = false;
let finished = false;
const options = parseOptions(process.argv.slice(2));
const [host, portText] = options.connect.split(":");
const port = Number.parseInt(portText ?? "", 10);
if (host.length === 0 || !Number.isInteger(port) || port < 1 || port > 65_535) {
  throw new Error("External FPAS terminal received an invalid --connect address.");
}

const socket = net.createConnection({ host, port });
const input = new TimedTerminalInput(
  events => { if (events.length > 0) send({ events }); },
  error => fail(`External terminal input failed: ${String(error)}`)
);
const utf8 = new StringDecoder("utf8");
let pending = "";

socket.setEncoding("utf8");
socket.once("connect", () => {
  send({ token: options.token });
  if (process.stdin.isTTY) {
    process.stdin.setRawMode(true);
    rawMode = true;
  }
  process.stdin.resume();
  sendResize();
});
socket.on("data", (chunk: string) => {
  pending += chunk;
  let newline = pending.indexOf("\n");
  while (newline >= 0) {
    const line = pending.slice(0, newline);
    pending = pending.slice(newline + 1);
    if (Buffer.byteLength(line, "utf8") > MAX_WIRE_LINE_BYTES) {
      fail("External FPAS terminal message exceeds the 2 MiB limit.");
      return;
    }
    receive(line);
    newline = pending.indexOf("\n");
  }
  if (Buffer.byteLength(pending, "utf8") > MAX_WIRE_LINE_BYTES) {
    fail("External FPAS terminal message exceeds the 2 MiB limit.");
  }
});
socket.once("error", (error) => fail(`External FPAS terminal failed: ${error.message}`));
socket.once("close", () => finish());

process.stdin.on("data", (chunk: Buffer) => {
  input.feed(utf8.write(chunk));
});
process.stdin.once("end", () => send({ closed: true }));
process.stdout.on("resize", sendResize);
process.once("exit", restoreTerminal);
process.once("SIGTERM", () => {
  send({ closed: true });
  finish();
});

function receive(line: string): void {
  let message: HostMessage;
  try {
    message = JSON.parse(line) as HostMessage;
  } catch {
    fail("External FPAS terminal received invalid JSON.");
    return;
  }
  if (typeof message.output === "string" && !process.stdout.write(message.output)) {
    socket.pause();
    process.stdout.once("drain", () => socket.resume());
  }
  if (typeof message.exitCode === "number") exitCode = message.exitCode;
  if (message.finished === true) finish();
}

function sendResize(): void {
  const width = process.stdout.columns;
  const height = process.stdout.rows;
  if (width && height) send({ events: [{ kind: "resize", width, height }] });
}

function send(message: Record<string, unknown>): void {
  if (finished || socket.destroyed) return;
  const line = `${JSON.stringify(message)}\n`;
  if (Buffer.byteLength(line, "utf8") + socket.writableLength > MAX_INPUT_BUFFER_BYTES) {
    fail("External FPAS terminal input exceeds the 4 MiB buffer limit.");
    return;
  }
  if (!socket.write(line)) {
    process.stdin.pause();
    socket.once("drain", () => { if (!finished) process.stdin.resume(); });
  }
}

function finish(): void {
  if (finished) return;
  finished = true;
  input.dispose();
  restoreTerminal();
  process.stdin.pause();
  socket.end();
  process.exitCode = exitCode;
}

function restoreTerminal(): void {
  if (rawMode && process.stdin.isTTY) process.stdin.setRawMode(false);
  rawMode = false;
}

function parseOptions(args: readonly string[]): { connect: string; token: string } {
  const connectIndex = args.indexOf("--connect");
  const tokenIndex = args.indexOf("--token");
  const connect = connectIndex >= 0 ? args[connectIndex + 1] : undefined;
  const token = tokenIndex >= 0 ? args[tokenIndex + 1] : undefined;
  if (!connect || !token) throw new Error("External FPAS terminal requires --connect and --token.");
  return { connect, token };
}

function fail(message: string): void {
  process.stderr.write(`${message}\n`);
  exitCode = 1;
  finish();
}
