import type { WorkflowOperation } from "./model";

/** Builds one non-interactive, shell-free CLI argument vector. */
export function operationArguments(
  operation: WorkflowOperation,
  target: string
): string[] {
  switch (operation) {
    case "check":
    case "build":
      return [operation, target];
    case "test":
      return [
        "test",
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
  target: string
): string[] {
  return ["test", "--list", target];
}

/** Builds a complete or exactly selected machine-readable test invocation. */
export function testRunArguments(
  target: string,
  file?: string,
  timeoutSeconds?: number
): string[] {
  const args = operationArguments("test", target);
  if (timeoutSeconds !== undefined) {
    args.splice(args.length - 1, 0, "--timeout", String(timeoutSeconds));
  }
  if (file !== undefined) {
    args.splice(args.length - 1, 0, "--file", file);
  }
  return args;
}

/** Builds an interactive run invocation with explicit program arguments. */
export function runArguments(
  target: string,
  programArguments: readonly string[]
): string[] {
  const args = ["run", target];
  if (programArguments.length > 0) {
    args.push("--", ...programArguments);
  }
  return args;
}
