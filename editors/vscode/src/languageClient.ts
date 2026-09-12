import path from "node:path";

import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  RevealOutputChannelOn,
  ServerOptions
} from "vscode-languageclient/node";

import { ToolchainResolver } from "./toolchain";

/** Owns exactly one Functional Pascal language-client process. */
export class LanguageClientController {
  private client: LanguageClient | undefined;
  private executable: string | undefined;

  public constructor(
    private readonly toolchain: ToolchainResolver,
    private readonly outputChannel: vscode.LogOutputChannel
  ) {}

  /** Starts the language client unless it is already running. */
  public async start(): Promise<string> {
    if (this.client !== undefined && this.executable !== undefined) {
      return this.executable;
    }

    const toolchain = await this.toolchain.resolve();
    const workspaceDirectory = vscode.workspace.workspaceFolders?.find(
      (folder) => folder.uri.scheme === "file"
    )?.uri.fsPath;
    const serverOptions: ServerOptions = {
      command: toolchain.executable,
      args: ["lsp"],
      options: {
        cwd: workspaceDirectory ?? path.dirname(toolchain.executable)
      }
    };
    const clientOptions: LanguageClientOptions = {
      documentSelector: [{ scheme: "file", language: "fpas" }],
      initializationOptions: {
        standardLibraryUri: vscode.Uri.file(toolchain.standardLibrary).toString()
      },
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
    this.executable = toolchain.executable;
    this.outputChannel.appendLine(
      `Functional Pascal language server started: ${toolchain.executable} lsp`
    );
    return toolchain.executable;
  }

  /** Stops the current language client and waits for its child process to exit. */
  public async stop(): Promise<void> {
    const client = this.client;
    this.client = undefined;
    this.executable = undefined;
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
