namespace Fpas.Debugging.Sessions;

public sealed record SourcePosition(string Path, int Line, int Column);
public sealed record FailureInfo(string Code, string Message);
public sealed record ExceptionInfo(string Id, string? Description, string? BreakMode, string? Type, string? StackTrace);
public sealed record SessionSnapshot(string SessionId, string State, long Event, long? StopId,
    string? Reason, int? ThreadId, SourcePosition? Location, int? ExitCode, FailureInfo? Error,
    IReadOnlyList<BreakpointInfo> Breakpoints, ExceptionInfo? Exception);
public sealed record Reply<T>(SessionSnapshot Session, T Data);
public sealed record BreakpointInfo(int? Id, bool Verified, int? Line, string? Source, string? Message);
public sealed record ThreadInfo(int Id, string Name);
public sealed record FrameInfo(string Handle, string Name, SourcePosition? Location);
public sealed record StackPage(IReadOnlyList<FrameInfo> Frames, int Start, int? Total, bool Truncated);
public sealed record ScopeInfo(string Name, string? Handle, bool Expensive);
public sealed record VariableInfo(string Name, string Value, string? Type, string? Handle, string? EvaluateName);
public sealed record VariablesPage(IReadOnlyList<ScopeInfo> Scopes, IReadOnlyList<VariableInfo> Variables, int Start, bool Truncated);
public sealed record Evaluation(string Value, string? Type, string? Handle);
public sealed record OutputChunk(long Sequence, string Category, string Text);
public sealed record OutputPage(IReadOnlyList<OutputChunk> Items, long NextSequence, bool Truncated, bool More);
