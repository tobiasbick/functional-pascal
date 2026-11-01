import type * as vscode from "vscode";
import type { LanguageClientController } from "./languageClient";
import { registerInspect } from "./sourceTools/inspect";
import { registerParameter } from "./sourceTools/refactor";

/** Registers source tools against the existing language-client and dirty-buffer session. */
export function registerSourceTools(context: vscode.ExtensionContext, client: LanguageClientController): void {
  registerInspect(context, client);
  registerParameter(context, client);
}
