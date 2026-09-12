/** Validation and caching for one installed Functional Pascal toolchain. */

import { execFile } from "node:child_process";
import { statSync } from "node:fs";
import path from "node:path";
import { promisify } from "node:util";

import { resolveCliPath } from "./cliPath";

const execFileAsync = promisify(execFile);

/** CLI version understood by this extension build. */
export const SUPPORTED_CLI_VERSION = "0.0.1";

/** Installed paths reported by `fpas env --json`. */
export interface ToolchainInfo {
  readonly executable: string;
  readonly standardLibrary: string;
  readonly version: string;
}

/** An actionable installed-toolchain discovery or compatibility failure. */
export class ToolchainError extends Error {
  public constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = "ToolchainError";
  }
}

/** Resolve and validate one toolchain once per configured executable. */
export class ToolchainResolver {
  private cached: ToolchainInfo | undefined;
  private generation = 0;
  private pending: Promise<ToolchainInfo> | undefined;

  /** Return the configured or PATH-resolved executable without starting it. */
  public executablePath(): string {
    try {
      return resolveCliPath();
    } catch (error) {
      throw toolchainError(error);
    }
  }

  /** Query and validate the selected toolchain. */
  public async resolve(): Promise<ToolchainInfo> {
    const executable = this.executablePath();
    if (this.cached?.executable === executable) return this.cached;
    if (this.pending === undefined) {
      const generation = this.generation;
      this.pending = queryToolchain(executable).then(
        (toolchain) => {
          if (generation === this.generation) {
            this.cached = toolchain;
            this.pending = undefined;
          }
          return toolchain;
        },
        (error: unknown) => {
          if (generation === this.generation) this.pending = undefined;
          throw toolchainError(error);
        }
      );
    }
    return this.pending;
  }

  /** Forget a cached environment after the executable setting changes. */
  public invalidate(): void {
    this.generation += 1;
    this.cached = undefined;
    this.pending = undefined;
  }
}

/** Parse and validate the stable `fpas env --json` output contract. */
export function parseToolchainEnvironment(
  executable: string,
  stdout: string,
  isManifest: (candidate: string) => boolean = isFile
): ToolchainInfo {
  let value: unknown;
  try {
    value = JSON.parse(stdout);
  } catch (error) {
    throw new ToolchainError("`fpas env --json` returned invalid JSON.", { cause: error });
  }
  if (!isEnvironment(value)) {
    throw new ToolchainError("`fpas env --json` returned an unsupported environment schema.");
  }
  if (value.version !== SUPPORTED_CLI_VERSION) {
    throw new ToolchainError(
      `Functional Pascal toolchain is incompatible. Expected version ${SUPPORTED_CLI_VERSION}, received ${value.version}. Install a matching toolchain or select another executable.`
    );
  }
  const standardLibrary = path.normalize(value.standardLibrary);
  if (!path.isAbsolute(standardLibrary) || !isManifest(path.join(standardLibrary, "stdlib.fpasprj"))) {
    throw new ToolchainError(
      `Functional Pascal standard library is incomplete: ${standardLibrary}. Expected \`stdlib.fpasprj\` in that directory.`
    );
  }
  return { executable, standardLibrary, version: value.version };
}

async function queryToolchain(executable: string): Promise<ToolchainInfo> {
  let stdout: string;
  try {
    const result = await execFileAsync(executable, ["env", "--json"], {
      cwd: path.dirname(executable),
      encoding: "utf8",
      timeout: 5_000,
      windowsHide: true
    });
    stdout = result.stdout;
  } catch (error) {
    throw new ToolchainError(
      `Cannot query Functional Pascal toolchain at ${executable}. Run \`fpas env --json\` in a terminal to inspect the installation.`,
      { cause: error }
    );
  }
  return parseToolchainEnvironment(executable, stdout);
}

function isEnvironment(value: unknown): value is {
  schemaVersion: 1;
  version: string;
  executable: string;
  standardLibrary: string;
} {
  if (typeof value !== "object" || value === null) return false;
  const record = value as Record<string, unknown>;
  return (
    record.schemaVersion === 1 &&
    typeof record.version === "string" &&
    typeof record.executable === "string" &&
    path.isAbsolute(record.executable) &&
    typeof record.standardLibrary === "string"
  );
}

function isFile(candidate: string): boolean {
  try {
    return statSync(candidate).isFile();
  } catch {
    return false;
  }
}

function toolchainError(error: unknown): ToolchainError {
  return error instanceof ToolchainError
    ? error
    : new ToolchainError(error instanceof Error ? error.message : String(error), {
        cause: error
      });
}
