namespace Fpas.Debugging.Core;

internal sealed record SourceBreakpoint(string Source, int Line, int? Column = null);
internal sealed record LaunchTarget(string Dotnet, string Program, string Cwd, SourceBreakpoint? InitialBreakpoint = null, string[]? Arguments = null,
    TerminalLaunch? Terminal = null);
internal sealed record TerminalLaunch(string Pipe, int Columns, int Rows, string? TuiPipe = null);
internal sealed record TerminalSize(int Columns, int Rows);
internal sealed record BreakpointPosition(int Line, int? Column = null);
internal sealed record BreakpointsRequest(string Source, BreakpointPosition[] Positions, long? StopId = null);
internal sealed record BreakpointInfo(long Id, SourceBreakpoint Requested, string State, SourceLocation? Actual = null, DebugProblem? Error = null, long Hits = 0);
internal sealed record ResumeRequest(long StopId, string Kind);
internal sealed record DebugProblem(string Code, string Message);
internal sealed record DebugModuleIssue(string Module, DebugProblem Error);
internal sealed record ExceptionPolicy(bool Raised = false, bool Unhandled = true);
internal sealed record PassiveText(string? Text, bool Truncated = false, string? RawUtf16 = null, string Status = "available");
internal sealed record ExceptionFrame(string Name, string Module, uint MethodToken, SourceLocation? Source, bool Approximate, string Status = "available");
internal sealed record DebugExceptionInfo(string Type, PassiveText Message, string Policy, SourceLocation? Source,
    ExceptionFrame[] ThrowStack, DebugFrame[] CurrentStack, bool Truncated = false, string Status = "available");
internal sealed record DebugSnapshot(string SessionId, string State, long Revision = 0, long? StopId = null, int? ProcessId = null,
    string? Reason = null, int? ExitCode = null, DebugProblem? Error = null, int? ExitSignal = null, long[]? HitBreakpoints = null,
    BreakpointInfo[]? Breakpoints = null, DebugModuleIssue[]? ModuleIssues = null, AsyncStepInfo? AsyncStep = null);
internal sealed record DebugBinding(string Name, string Type, string Kind, string? Scalar, string Status = "available",
    bool Truncated = false, string? RawUtf16 = null, DebugProblem? Error = null, long? Reference = null, int? Children = null,
    string? ObjectId = null, bool Cycle = false, DebugTypeIdentity? RuntimeType = null, int[]? Lengths = null, int[]? LowerBounds = null,
    int? SourceChildren = null, DebugProblem? SourceError = null, bool Constant = false);
internal sealed record DebugTypeIdentity(string Name, uint ElementKind, long? ModuleId = null, Guid? ModuleIdentity = null,
    uint? Token = null, DebugTypeIdentity[]? Arguments = null, int? Rank = null, bool Exact = true);
internal sealed record BindingPage(DebugBinding[] Items, int Total, bool Truncated = false);
internal sealed record SourceLocation(string Path, int Line, int Column, int EndLine, int EndColumn, string Status = "available");
internal sealed record DebugThread(uint Id, bool IsStopThread, string Name);
internal sealed record AsyncFrame(string? Id, string Method, string State, string IdentityStatus = "available");
internal sealed record AsyncStepInfo(string Id, string Method, string Phase, SourceLocation? Source);
internal sealed record DebugFrame(long Id, uint ThreadId, string Name, string Module, uint MethodToken, int? IlOffset,
    SourceLocation? Source, string Status, bool Approximate, bool Generated, AsyncFrame? Async = null, string? RuntimeName = null);
internal sealed record StackPage(DebugFrame[] Items, int? Total, bool Truncated, int OmittedNativeChains);
internal sealed record StackRequest(long StopId, uint ThreadId, int Start = 0, int Count = 32);
internal sealed record FrameRequest(long StopId, long FrameId, int Start = 0, int Count = 32, string View = "source");
internal sealed record StaticRequest(long StopId, long FrameId, string? TypeName = null, int Start = 0, int Count = 32);
internal sealed record ValueRequest(long StopId, long Reference, int Start = 0, int Count = 32, string View = "raw");
internal sealed record EvaluationRequest(long StopId, long FrameId, string Expression);
internal sealed record EvaluationResult(string Expression, DebugBinding? Value = null, DebugProblem? Error = null);
internal sealed record StopRequest(long StopId);
internal sealed record CapturedOutput(long Sequence, string Stream, string Text);
internal sealed record OutputSnapshot(CapturedOutput[] Items, long NextSequence, bool Truncated, long TotalBytes = 0, int RetainedBytes = 0, bool LimitReached = false,
    bool EncodingLoss = false);

internal sealed class DebugException(string code, string message) : Exception(message)
{
    internal string Code { get; } = code;
}
