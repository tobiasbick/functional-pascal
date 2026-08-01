import * as vscode from "vscode";

import { LanguageClientController } from "./languageClient";
import { WorkflowController, WORKFLOW_COMMANDS } from "./workflow/controller";
import type { WorkflowTestStatus } from "./workflow/model";
import type { ParsedWorkflowDiagnostic } from "./workflow/model";

/** Command that reveals the Functional Pascal output channel. */
export const SHOW_OUTPUT_COMMAND = "functionalPascal.showOutput";

/** Command that stops and starts the bundled language server. */
export const RESTART_LANGUAGE_SERVER_COMMAND =
  "functionalPascal.restartLanguageServer";

/** Name of the extension's dedicated output channel. */
export const OUTPUT_CHANNEL_NAME = "Functional Pascal";

/** First line emitted when the extension activates. */
export const ACTIVATION_MESSAGE =
  "Functional Pascal extension activated.";

/** Public API exposed to extension-host regression tests. */
export interface FunctionalPascalExtensionApi {
  /** Exact message written to the output channel during activation. */
  readonly activationMessage: string;
  /** Whether the development language server completed its LSP handshake. */
  readonly languageServerStarted: boolean;
  /** Exact executable used for the running language server, when available. */
  readonly languageServerPath?: string;
  /** Actionable startup failure when the development server did not start. */
  readonly languageServerError?: string;
  /** Project-workflow surface used by real Extension Host regression tests. */
  readonly workflow: {
    readonly commands: readonly string[];
    cliPath(): string;
    selectProject(uri: vscode.Uri): Promise<void>;
    discoverTests(): Promise<string[]>;
    runTests(files?: readonly string[]): Promise<Record<string, WorkflowTestStatus>>;
    lastOperation():
      | {
          readonly exitCode: number | null;
          readonly stderr: string;
          readonly diagnostics: readonly ParsedWorkflowDiagnostic[];
        }
      | undefined;
  };
}

let languageClient: LanguageClientController | undefined;

/** Activates the Functional Pascal extension. */
export async function activate(
  context: vscode.ExtensionContext
): Promise<FunctionalPascalExtensionApi> {
  const outputChannel = vscode.window.createOutputChannel(OUTPUT_CHANNEL_NAME, {
    log: true
  });
  outputChannel.appendLine(ACTIVATION_MESSAGE);
  languageClient = new LanguageClientController(context, outputChannel);
  const workflow = new WorkflowController(context, outputChannel);
  const workflowApi = {
    commands: Object.values(WORKFLOW_COMMANDS),
    cliPath: () => workflow.cliPath(),
    selectProject: (uri: vscode.Uri) => workflow.selectProject(uri),
    discoverTests: () => workflow.discoverTests(),
    runTests: (files?: readonly string[]) => workflow.runTests(files),
    lastOperation: () => workflow.lastOperation()
  };

  const showOutput = vscode.commands.registerCommand(SHOW_OUTPUT_COMMAND, () => {
    outputChannel.show(true);
  });
  const restartLanguageServer = vscode.commands.registerCommand(
    RESTART_LANGUAGE_SERVER_COMMAND,
    async () => {
      try {
        const serverPath = await languageClient?.restart();
        if (serverPath !== undefined) {
          outputChannel.appendLine(
            `Functional Pascal language server restarted: ${serverPath}`
          );
        }
      } catch (error) {
        outputChannel.appendLine(
          `Functional Pascal language server restart failed: ${errorMessage(error)}`
        );
        outputChannel.show(true);
      }
    }
  );

  context.subscriptions.push(outputChannel, showOutput, restartLanguageServer);

  try {
    const languageServerPath = await languageClient.start();
    return {
      activationMessage: ACTIVATION_MESSAGE,
      languageServerStarted: true,
      languageServerPath,
      workflow: workflowApi
    };
  } catch (error) {
    const message = errorMessage(error);
    outputChannel.appendLine(
      `Functional Pascal language server did not start: ${message}`
    );
    return {
      activationMessage: ACTIVATION_MESSAGE,
      languageServerStarted: false,
      languageServerError: message,
      workflow: workflowApi
    };
  }
}

/** Stops the language server during extension-host shutdown. */
export async function deactivate(): Promise<void> {
  const current = languageClient;
  languageClient = undefined;
  await current?.stop();
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
