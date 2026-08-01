import path from "node:path";

import { WorkflowProcessRunner } from "./processes";

/** CLI version implemented by the repository workspace packaged with this extension. */
export const SUPPORTED_CLI_VERSION = "0.0.1";

/** Parses the stable `fpas --version` output contract. */
export function parseCliVersion(stdout: string): string | undefined {
  return /^fpas (\d+\.\d+\.\d+)\s*$/u.exec(stdout)?.[1];
}

/** Resolves and validates one CLI once per Extension Host session. */
export class CliCompatibility {
  private validated: string | undefined;

  public constructor(
    private readonly resolvePath: () => string,
    private readonly runner: WorkflowProcessRunner
  ) {}

  /** Returns a compatible executable or one actionable recovery error. */
  public async resolve(): Promise<string> {
    const executable = this.resolvePath();
    if (this.validated === executable) {
      return executable;
    }
    const result = await this.runner.run(
      executable,
      ["--version"],
      path.dirname(executable)
    );
    const actual = parseCliVersion(result.stdout);
    if (result.exitCode !== 0 || actual !== SUPPORTED_CLI_VERSION) {
      throw new Error(
        `Functional Pascal CLI is incompatible. Expected \`fpas ${SUPPORTED_CLI_VERSION}\`, received ${actual === undefined ? "invalid version output" : `\`fpas ${actual}\``}. Rebuild the host-native VSIX and reinstall it.`
      );
    }
    this.validated = executable;
    return executable;
  }
}
