import * as vscode from "vscode";

import { programMainPath } from "./manifest";
import { pathIdentity } from "./pathIdentity";

const EXCLUDE_GLOB = "**/{.git,target,node_modules,.vscode-test,dist,out}/**";

/** A cached snapshot shared by project selection and F5 ownership lookup. */
export class ProjectIndex implements vscode.Disposable {
  private generation = 0;
  private snapshot: Promise<readonly vscode.Uri[]> | undefined;
  private owners: Promise<Map<string, string[]>> | undefined;
  private readonly subscriptions: vscode.Disposable[] = [];

  public constructor(
    private readonly scan: () => Thenable<vscode.Uri[]> = () =>
      vscode.workspace.findFiles("**/*.{fpasprj,fpasworkspace}", EXCLUDE_GLOB),
    private readonly read: (uri: vscode.Uri) => Thenable<Uint8Array> = uri =>
      vscode.workspace.fs.readFile(uri),
    watch = true
  ) {
    if (!watch) return;
    const projects = vscode.workspace.createFileSystemWatcher("**/*.fpasprj");
    const workspaces = vscode.workspace.createFileSystemWatcher("**/*.fpasworkspace");
    for (const watcher of [projects, workspaces]) {
      this.subscriptions.push(
        watcher,
        watcher.onDidCreate(() => this.invalidate()),
        watcher.onDidChange(() => this.invalidate()),
        watcher.onDidDelete(() => this.invalidate())
      );
    }
    this.subscriptions.push(vscode.workspace.onDidChangeWorkspaceFolders(() => this.invalidate()));
  }

  /** Return the current sorted manifest list, coalescing concurrent scans. */
  public async candidates(): Promise<readonly vscode.Uri[]> {
    for (;;) {
      const generation = this.generation;
      if (this.snapshot === undefined) {
        this.snapshot = Promise.resolve(this.scan())
          .then(files => files.sort((a, b) => a.fsPath.localeCompare(b.fsPath)));
      }
      const pending = this.snapshot;
      try {
        const files = await pending;
        if (generation === this.generation) return files;
      } catch (error) {
        if (generation === this.generation) this.snapshot = undefined;
        throw error;
      }
    }
  }

  /** Find every project that declares the source as its program main. */
  public async programOwners(sourcePath: string): Promise<string[]> {
    for (;;) {
      const generation = this.generation;
      if (this.owners === undefined) this.owners = this.buildOwners(generation);
      const owners = await this.owners;
      if (generation === this.generation) return owners.get(pathIdentity(sourcePath)) ?? [];
    }
  }

  /** Discard snapshots after a manifest or workspace-folder change. */
  public invalidate(): void {
    this.generation += 1;
    this.snapshot = undefined;
    this.owners = undefined;
  }

  public dispose(): void {
    for (const disposable of this.subscriptions) disposable.dispose();
  }

  private async buildOwners(generation: number): Promise<Map<string, string[]>> {
    try {
      const manifests = (await this.candidates()).filter(uri => uri.fsPath.endsWith(".fpasprj"));
      const entries = await Promise.all(manifests.map(async uri => {
        try {
          const text = new TextDecoder().decode(await this.read(uri));
          return { manifest: uri.fsPath, main: programMainPath(uri.fsPath, text) };
        } catch {
          // An invalid unrelated project must not block an open source file.
          return undefined;
        }
      }));
      const owners = new Map<string, string[]>();
      for (const entry of entries) {
        if (entry?.main === undefined) continue;
        const key = pathIdentity(entry.main);
        owners.set(key, [...(owners.get(key) ?? []), entry.manifest]);
      }
      return owners;
    } catch (error) {
      if (this.generation === generation) this.owners = undefined;
      throw error;
    }
  }
}

/** Shared index for extension commands and debugger target discovery. */
export const projectIndex = new ProjectIndex();
