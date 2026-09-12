import { type ChildProcess, spawn } from "node:child_process";
import path from "node:path";
import process from "node:process";

import * as vscode from "vscode";

interface ExternalTerminalInvocation {
  readonly command: string;
  readonly args: readonly string[];
}

/** Start one interactive program in a native terminal window. */
export async function launchExternalProgram(
  executable: string,
  args: readonly string[],
  cwd: string
): Promise<ChildProcess> {
  const invocation = externalTerminalInvocation(
    process.platform,
    executable,
    args,
    cwd,
    configuredExternalTerminal(process.platform)
  );
  const child = spawn(invocation.command, invocation.args, {
    cwd,
    detached: true,
    stdio: "ignore",
    windowsHide: process.platform === "win32"
  });
  await new Promise<void>((resolve, reject) => {
    child.once("spawn", resolve);
    child.once("error", reject);
  });
  child.unref();
  return child;
}

/** Build a platform-native terminal invocation without joining process arguments. */
export function externalTerminalInvocation(
  platform: NodeJS.Platform,
  executable: string,
  args: readonly string[],
  cwd: string,
  configuredTerminal?: string
): ExternalTerminalInvocation {
  if (platform === "win32") {
    const payload = Buffer.from(
      JSON.stringify({ executable, arguments: args, cwd }),
      "utf8"
    ).toString("base64");
    const script = [
      `$fpasInvocation = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('${payload}')) | ConvertFrom-Json`,
      "$fpasArguments = @($fpasInvocation.arguments)",
      "Set-Location -LiteralPath $fpasInvocation.cwd",
      "& $fpasInvocation.executable @fpasArguments"
    ].join("; ");
    const encodedScript = Buffer.from(script, "utf16le").toString("base64");
    return {
      command: process.env.ComSpec || "cmd.exe",
      args: [
        "/d",
        "/s",
        "/c",
        "start",
        "",
        "powershell.exe",
        "-NoLogo",
        "-NoProfile",
        "-NoExit",
        "-EncodedCommand",
        encodedScript
      ]
    };
  }
  if (platform === "linux") {
    const terminal = configuredTerminal?.trim() || "xterm";
    const separator = path.basename(terminal).startsWith("gnome-terminal") ? "--" : "-e";
    return { command: terminal, args: [separator, executable, ...args] };
  }
  if (platform === "darwin") {
    const application = configuredTerminal?.trim() || "Terminal.app";
    const command = `cd ${shellQuote(cwd)} && exec ${[executable, ...args]
      .map(shellQuote)
      .join(" ")}`;
    const script = `tell application ${appleScriptString(application)} to do script ${appleScriptString(command)}`;
    return { command: "/usr/bin/osascript", args: ["-e", script] };
  }
  throw new Error(`External FPAS terminals are unsupported on ${platform}.`);
}

function configuredExternalTerminal(platform: NodeJS.Platform): string | undefined {
  const setting = platform === "win32" ? "windowsExec" : platform === "darwin" ? "osxExec" : "linuxExec";
  return vscode.workspace.getConfiguration("terminal.external").get<string>(setting);
}

function shellQuote(value: string): string {
  return `'${value.replaceAll("'", `'"'"'`)}'`;
}

function appleScriptString(value: string): string {
  return `"${value.replaceAll("\\", "\\\\").replaceAll('"', '\\"')}"`;
}
