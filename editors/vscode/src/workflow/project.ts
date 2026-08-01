import path from "node:path";

import * as vscode from "vscode";

const SELECTION_KEY = "functionalPascal.selectedProject";
const EXCLUDE_GLOB =
  "**/{.git,target,node_modules,.vscode-test,dist,out}/**";

/** Returns a remembered candidate only when it still exists. */
export function rememberedProject(
  candidates: readonly string[],
  remembered: string | undefined
): string | undefined {
  if (remembered === undefined) {
    return undefined;
  }
  const normalized = path.normalize(remembered);
  return candidates.find(
    (candidate) => path.normalize(candidate).toLocaleLowerCase() === normalized.toLocaleLowerCase()
  );
}

/** Owns explicit, workspace-persisted project/workspace selection. */
export class ProjectSelector implements vscode.Disposable {
  private readonly changed = new vscode.EventEmitter<vscode.Uri | undefined>();

  public constructor(private readonly state: vscode.Memento) {}

  /** Fires after the selected manifest changes. */
  public readonly onDidChange = this.changed.event;

  /** Finds bounded project and workspace manifest candidates. */
  public async candidates(): Promise<vscode.Uri[]> {
    const manifests = await vscode.workspace.findFiles(
      "**/*.{fpasprj,fpasworkspace}",
      EXCLUDE_GLOB
    );
    return manifests.sort((left, right) => left.fsPath.localeCompare(right.fsPath));
  }

  /** Returns the valid remembered target, or the sole unambiguous candidate. */
  public async current(): Promise<vscode.Uri | undefined> {
    const candidates = await this.candidates();
    const selected = rememberedProject(
      candidates.map((candidate) => candidate.fsPath),
      this.state.get<string>(SELECTION_KEY)
    );
    if (selected !== undefined) {
      return vscode.Uri.file(selected);
    }
    if (candidates.length === 1) {
      await this.remember(candidates[0]);
      return candidates[0];
    }
    return undefined;
  }

  /** Selects an explicit target or prompts when multiple manifests exist. */
  public async select(explicit?: vscode.Uri): Promise<vscode.Uri | undefined> {
    const candidates = await this.candidates();
    let selected = explicit;
    if (selected !== undefined) {
      selected = candidates.find(
        (candidate) =>
          path.normalize(candidate.fsPath).toLocaleLowerCase() ===
          path.normalize(selected!.fsPath).toLocaleLowerCase()
      );
      if (selected === undefined) {
        throw new Error("The selected FPAS project or workspace is outside the opened folder.");
      }
    } else {
      const remembered = rememberedProject(
        candidates.map((candidate) => candidate.fsPath),
        this.state.get<string>(SELECTION_KEY)
      );
      if (remembered !== undefined) {
        return vscode.Uri.file(remembered);
      }
    }
    if (selected === undefined && candidates.length === 1) {
      [selected] = candidates;
    } else if (selected === undefined && candidates.length > 1) {
      const picked = await vscode.window.showQuickPick(
        candidates.map((uri) => ({
          label: path.basename(uri.fsPath),
          description: vscode.workspace.asRelativePath(uri, false),
          uri
        })),
        { placeHolder: "Select the Functional Pascal project or workspace" }
      );
      selected = picked?.uri;
    }
    if (selected !== undefined) {
      await this.remember(selected);
    }
    return selected;
  }

  public dispose(): void {
    this.changed.dispose();
  }

  private async remember(uri: vscode.Uri): Promise<void> {
    await this.state.update(SELECTION_KEY, uri.fsPath);
    this.changed.fire(uri);
  }
}
