/** Non-interactive Functional Pascal project operation. */
export type WorkflowOperation =
  | "check"
  | "build"
  | "test"
  | "format"
  | "formatCheck";

/** Complete result of one native CLI process. */
export interface WorkflowProcessResult {
  readonly exitCode: number | null;
  readonly stdout: string;
  readonly stderr: string;
  readonly cancelled: boolean;
}

/** Parsed compiler diagnostic from the documented CLI text contract. */
export interface ParsedWorkflowDiagnostic {
  readonly path: string;
  readonly line: number;
  readonly column: number;
  readonly severity: "error" | "warning";
  readonly code: string;
  readonly message: string;
  readonly help?: string;
}

/** Supported machine-readable test outcome. */
export type WorkflowTestStatus =
  | "pass"
  | "skipped"
  | "not_run"
  | "assert_failed"
  | "compile_error"
  | "runtime_error"
  | "timed_out";

/** One case from `fpas test --report json`. */
export interface WorkflowTestCase {
  readonly file: string;
  readonly status: WorkflowTestStatus;
}

/** Versioned machine-readable FPAS test report. */
export interface WorkflowTestReport {
  readonly version: 1;
  readonly tests: readonly WorkflowTestCase[];
}
