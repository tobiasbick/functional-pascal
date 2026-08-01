import type { WorkflowOperation } from "./model";

/** Builds one non-interactive, shell-free CLI argument vector. */
export function operationArguments(
  operation: WorkflowOperation,
  target: string,
  standardLibrary: string
): string[] {
  switch (operation) {
    case "check":
    case "build":
      return [operation, "--std-lib", standardLibrary, target];
    case "test":
      return [
        "test",
        "--std-lib",
        standardLibrary,
        "--report",
        "json",
        target
      ];
    case "format":
      return ["fmt", target];
    case "formatCheck":
      return ["fmt", "--check", target];
  }
}

/** Builds the test-discovery invocation. */
export function testListArguments(
  target: string,
  standardLibrary: string
): string[] {
  return ["test", "--std-lib", standardLibrary, "--list", target];
}

/** Builds a complete or filtered machine-readable test invocation. */
export function testRunArguments(
  target: string,
  standardLibrary: string,
  filter?: string,
  timeoutSeconds?: number
): string[] {
  const args = operationArguments("test", target, standardLibrary);
  if (timeoutSeconds !== undefined) {
    args.splice(args.length - 1, 0, "--timeout", String(timeoutSeconds));
  }
  if (filter !== undefined) {
    args.splice(args.length - 1, 0, "--filter", filter);
  }
  return args;
}

/** Builds an interactive run invocation with explicit program arguments. */
export function runArguments(
  target: string,
  standardLibrary: string,
  programArguments: readonly string[]
): string[] {
  const args = ["run", "--std-lib", standardLibrary, target];
  if (programArguments.length > 0) {
    args.push("--", ...programArguments);
  }
  return args;
}
