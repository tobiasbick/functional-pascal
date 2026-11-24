// [Documentation](../../../docs/pascal/program-structure/project-commands.md#save-the-original-failure)
namespace Fpas.Debugging.Failures;

/// <summary>Options for a managed run that saves its first unhandled exception before resuming.</summary>
/// <param name="Program">Absolute executable DLL path, with matching portable symbols and sources.</param>
/// <param name="Cwd">Absolute target working directory.</param>
/// <param name="Arguments">Verbatim target arguments.</param>
/// <param name="ReportPath">Absolute destination for the debug JSON report; must not already exist.</param>
/// <param name="CaptureOutput">Capture bounded UTF-8 stdout/stderr instead of inheriting their handles.</param>
/// <param name="Timeout">Caller-selected overall deadline in seconds; zero denotes no deadline.</param>
/// <param name="Stdin">Optional input bytes followed by EOF. With captured output, missing input supplies EOF.</param>
public sealed record FailureRunOptions(string Program, string Cwd, string[] Arguments, string ReportPath,
    bool CaptureOutput = false, int Timeout = 0, ReadOnlyMemory<byte>? Stdin = null);

/// <summary>The natural target exit and any detached failure report saved during that run.</summary>
/// <param name="ExitCode">Actual target exit code.</param>
/// <param name="Output">Captured standard output, or empty when inherited.</param>
/// <param name="Error">Captured standard error, or empty when inherited.</param>
/// <param name="Truncated">Whether captured output exceeded its byte budget.</param>
/// <param name="ReportPath">Saved report path, or null if no unhandled exception was observed.</param>
public sealed record FailureRunResult(int ExitCode, string Output, string Error, bool Truncated, string? ReportPath);
