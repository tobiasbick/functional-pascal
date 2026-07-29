import path from "node:path";

import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  RevealOutputChannelOn,
  ServerOptions
} from "vscode-languageclient/node";

import { resolveServerPath } from "./serverPath";

/** Owns exactly one Functional Pascal language-client process. */
export class LanguageClientController {
  private client: LanguageClient | undefined;
  private serverPath: string | undefined;

  public constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly outputChannel: vscode.LogOutputChannel
  ) {}

  /** Starts the language client unless it is already running. */
  public async start(): Promise<string> {
    if (this.client !== undefined && this.serverPath !== undefined) {
      return this.serverPath;
    }

    const serverPath = resolveServerPath(this.context);
    const workspaceDirectory = vscode.workspace.workspaceFolders?.find(
      (folder) => folder.uri.scheme === "file"
    )?.uri.fsPath;
    const serverOptions: ServerOptions = {
      command: serverPath,
      options: {
        cwd: workspaceDirectory ?? path.dirname(serverPath)
      }
    };
    const clientOptions: LanguageClientOptions = {
      documentSelector: [{ scheme: "file", language: "fpas" }],
      outputChannel: this.outputChannel,
      revealOutputChannelOn: RevealOutputChannelOn.Never
    };
    const client = new LanguageClient(
      "functionalPascal",
      "Functional Pascal Language Server",
      serverOptions,
      clientOptions
    );

    try {
      await client.start();
    } catch (error) {
      await client.dispose();
      throw error;
    }
    this.client = client;
    this.serverPath = serverPath;
    this.outputChannel.appendLine(
      `Functional Pascal language server started: ${serverPath}`
    );
    return serverPath;
  }

  /** Stops the current language client and waits for its child process to exit. */
  public async stop(): Promise<void> {
    const client = this.client;
    this.client = undefined;
    this.serverPath = undefined;
    if (client === undefined) {
      return;
    }
    await client.stop();
    this.outputChannel.appendLine("Functional Pascal language server stopped.");
  }

  /** Restarts the current client with a newly resolved server executable. */
  public async restart(): Promise<string> {
    await this.stop();
    return this.start();
  }
}
