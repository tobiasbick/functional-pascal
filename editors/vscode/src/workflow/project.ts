import path from "node:path";

import * as vscode from "vscode";
import { pathIdentity } from "../projects/pathIdentity";
import { ProjectIndex, projectIndex } from "../projects/index";

const SELECTION_KEY = "functionalPascal.selectedProject";

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
    (candidate) => pathIdentity(candidate) === pathIdentity(normalized)
  );
}

/** Owns explicit, workspace-persisted project/workspace selection. */
export class ProjectSelector implements vscode.Disposable {
  private readonly changed = new vscode.EventEmitter<vscode.Uri | undefined>();

  public constructor(
    private readonly state: vscode.Memento,
    private readonly index: Pick<ProjectIndex, "candidates"> = projectIndex,
    private readonly pick: (
      items: readonly { label: string; description: string; uri: vscode.Uri }[],
      options: { placeHolder: string }
    ) => Thenable<{ uri: vscode.Uri } | undefined> = (items, options) =>
      vscode.window.showQuickPick(items, options)
  ) {}

  /** Fires after the selected manifest changes. */
  public readonly onDidChange = this.changed.event;

  /** Finds bounded project and workspace manifest candidates. */
  public async candidates(): Promise<vscode.Uri[]> {
    return [...await this.index.candidates()];
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

  /** Resolves ordinary operations without reopening an existing selection. */
  public async resolve(explicit?: vscode.Uri): Promise<vscode.Uri | undefined> {
    return explicit === undefined ? (await this.current()) ?? this.select() : this.select(explicit);
  }

  /** Selects an explicit target or prompts when multiple manifests exist. */
  public async select(explicit?: vscode.Uri): Promise<vscode.Uri | undefined> {
    const candidates = await this.candidates();
    let selected = explicit;
    if (selected !== undefined) {
      selected = candidates.find(
        (candidate) =>
          pathIdentity(candidate.fsPath) === pathIdentity(selected!.fsPath)
      );
      if (selected === undefined) {
        throw new Error("The selected FPAS project or workspace is outside the opened folder.");
      }
    }
    if (selected === undefined && candidates.length === 1) {
      [selected] = candidates;
    } else if (selected === undefined && candidates.length > 1) {
      const picked = await this.pick(
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
