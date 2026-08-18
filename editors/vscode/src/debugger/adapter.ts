/** VS Code launch configuration and executable DAP adapter wiring. */

import path from "node:path";

import * as vscode from "vscode";

import { resolveCliPath } from "../cliPath";
import { resolveStandardLibraryPath } from "../standardLibraryPath";
import { registerDictionaryCommands } from "./dictionaryCommands";
import { registerForcedReturnCommand } from "./forcedReturnCommand";
import { registerLiveReloadCommands } from "./liveReloadCommand";
import { registerSequenceCommands } from "./sequenceCommands";
import { registerStorageInitializationCommand } from "./storageInitializationCommand";
import { registerDebuggeeInputCommands } from "./inputCommand";
import { registerTaskControlCommands } from "./taskControlCommand";
import { registerTaskResultCommand } from "./taskResultCommand";
import { registerVariantConstructionCommand } from "./variantConstructionCommand";

/** Register the Functional Pascal debug type without changing LSP ownership. */
export function registerDebugger(context: vscode.ExtensionContext): void {
  const provider = vscode.debug.registerDebugConfigurationProvider(
    "fpas",
    new FunctionalPascalDebugConfigurationProvider()
  );
  const factory = vscode.debug.registerDebugAdapterDescriptorFactory(
    "fpas",
    new FunctionalPascalDebugAdapterFactory(context)
  );
  registerDictionaryCommands(context);
  registerForcedReturnCommand(context);
  registerLiveReloadCommands(context);
  registerSequenceCommands(context);
  registerVariantConstructionCommand(context);
  registerStorageInitializationCommand(context);
  registerTaskResultCommand(context);
  registerTaskControlCommands(context);
  registerDebuggeeInputCommands(context);
  context.subscriptions.push(provider, factory);
}

class FunctionalPascalDebugConfigurationProvider
  implements vscode.DebugConfigurationProvider
{
  resolveDebugConfiguration(
    folder: vscode.WorkspaceFolder | undefined,
    configuration: vscode.DebugConfiguration
  ): vscode.ProviderResult<vscode.DebugConfiguration> {
    if (!configuration.type && !configuration.request && !configuration.name) {
      const editor = vscode.window.activeTextEditor;
      if (editor?.document.languageId !== "fpas") {
        void vscode.window.showErrorMessage(
          "Open a Functional Pascal program or provide `program` in launch.json."
        );
        return undefined;
      }
      configuration.type = "fpas";
      configuration.request = "launch";
      configuration.name = "Debug Functional Pascal";
      configuration.program = editor.document.uri.fsPath;
      configuration.stopOnEntry = false;
    }
    const unsupported = unsupportedDebugRequestReason(configuration.request);
    if (unsupported) {
      void vscode.window.showErrorMessage(unsupported);
      return undefined;
    }
    if (typeof configuration.program !== "string" || configuration.program.length === 0) {
      void vscode.window.showErrorMessage(
        "Functional Pascal debugging requires a source, project, workspace, or compiled image in `program`."
      );
      return undefined;
    }
    configuration.cwd ??= folder?.uri.fsPath ?? path.dirname(configuration.program);
    configuration.args ??= [];
    return configuration;
  }
}

class FunctionalPascalDebugAdapterFactory
  implements vscode.DebugAdapterDescriptorFactory
{
  constructor(private readonly context: vscode.ExtensionContext) {}

  createDebugAdapterDescriptor(
    session: vscode.DebugSession
  ): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
    const executable = resolveCliPath(this.context);
    const standardLibrary = resolveStandardLibraryPath(this.context);
    return new vscode.DebugAdapterExecutable(
      executable,
      debugAdapterArguments(session.configuration, standardLibrary),
      { cwd: session.configuration.cwd }
    );
  }
}

/** Why a debug request cannot start; `undefined` means the request is allowed. */
export function unsupportedDebugRequestReason(request: unknown): string | undefined {
  if (request === "attach") {
    return "Functional Pascal debugging does not support attach; use a launch configuration.";
  }
  return undefined;
}

/** Build deterministic CLI arguments for one VS Code launch configuration. */
export function debugAdapterArguments(
  configuration: vscode.DebugConfiguration,
  standardLibrary: string
): string[] {
  const args = [
    "debug",
    "--std-lib",
    standardLibrary,
    String(configuration.program),
    "--protocol",
    "dap"
  ];
  if (typeof configuration.sourceRoot === "string" && configuration.sourceRoot.length > 0) {
    args.push("--source-root", configuration.sourceRoot);
  }
  const programArgs = Array.isArray(configuration.args)
    ? configuration.args.map(String)
    : [];
  if (programArgs.length > 0) {
    args.push("--", ...programArgs);
  }
  return args;
}
