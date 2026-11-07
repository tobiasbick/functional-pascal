using System.Text.Json;
using System.Text.Json.Serialization;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Batch;

internal sealed class BatchReport
{
    public int SchemaVersion => 1;
    public string Outcome { get; set; } = "error";
    public int ExitCode { get; set; } = 1;
    public int? TargetExitCode { get; set; }
    public string Termination { get; set; } = "not_started";
    public ReportStop? Stop { get; set; }
    public BreakpointInfo[] Breakpoints { get; set; } = [];
    public List<ReportThread> Threads { get; } = [];
    public List<ReportFrame> Frames { get; } = [];
    public List<ReportBinding> Bindings { get; } = [];
    public List<ReportExpression> Expressions { get; } = [];
    public ReportException? Exception { get; set; }
    public OutputSnapshot Output { get; set; } = new([], 0, false);
    public List<ReportIssue> Issues { get; } = [];
    public ReportCapture Capture { get; } = new();
    public ReportLimits Limits { get; set; } = new(30, 1_048_576);
    public ReportToolchain? Toolchain { get; set; }
    public List<BatchReport> Tracepoints { get; } = [];

    internal void Issue(string code, string stage, string message) => Issues.Add(new(code, stage, message));

    internal byte[] Serialize()
    {
        byte[] Encode() => JsonSerializer.SerializeToUtf8Bytes(this, BatchJson.Default.BatchReport);
        var data = Encode();
        if (data.Length < ReportLimits.ReportBytes) return data;
        Capture.Omit("report_bytes");
        while (Tracepoints.Count != 0 && data.Length >= ReportLimits.ReportBytes)
        {
            Tracepoints.RemoveAt(Tracepoints.Count - 1); ExitCode = 4; Outcome = "limit";
            data = Encode();
        }
        if (data.Length < ReportLimits.ReportBytes) return data;
        // Keep requested observations ahead of automatic context and target text.
        Bindings.Clear(); Frames.Clear(); Threads.Clear();
        data = Encode();
        if (data.Length < ReportLimits.ReportBytes) return data;
        Output = Output with { Items = [], RetainedBytes = 0, Truncated = true };
        data = Encode();
        if (data.Length < ReportLimits.ReportBytes) return data;
        for (var index = 0; index < Expressions.Count; index++)
        {
            var item = Expressions[index];
            Expressions[index] = item with { Value = item.Value is { } value ? value with { Children = [], Truncated = true, Omission = "report_bytes" } : null };
        }
        data = Encode();
        if (data.Length < ReportLimits.ReportBytes) return data;
        Expressions.Clear(); Exception = null; Breakpoints = [];
        Issue("report_limit", "report", "Explicit observations could not fit within the report ceiling.");
        if (ExitCode == 0) ExitCode = 3;
        return Encode();
    }
}

internal sealed record ReportStop(string Reason, uint? ThreadId, SourceLocation? Source);
internal sealed record ReportThread(uint Id, string Name, bool Selected, int? TotalFrames, int OmittedNativeChains, bool Truncated);
internal sealed record ReportFrame(int Index, uint ThreadId, string Name, string Module, uint MethodToken, int? IlOffset,
    SourceLocation? Source, string Status, bool Approximate, bool Generated, string? AsyncMethod, string? AsyncState);
internal sealed record ReportBinding(int Frame, string Scope, ReportValue Value);
internal sealed record ReportExpression(string Expression, ReportValue? Value = null, DebugProblem? Error = null);
internal sealed record ReportType(string Name, uint ElementKind, Guid? ModuleIdentity, uint? Token, ReportType[]? Arguments, int? Rank, bool Exact);
internal sealed record ReportValue(string Name, string Type, string Kind, string? Scalar, string Status, bool Truncated, string? RawUtf16,
    DebugProblem? Error, string? ObjectId, bool Cycle, ReportType? RuntimeType, int[]? Lengths, int[]? LowerBounds,
    int? TotalChildren, DebugProblem? SourceError, bool Constant, ReportValue[] Children, string? Omission = null);
internal sealed record ReportException(string Type, PassiveText Message, string Policy, SourceLocation? Source, ExceptionFrame[] ThrowStack, bool Truncated, string Status);
internal sealed record ReportIssue(string Code, string Stage, string Message);
internal sealed record ReportToolchain(string DebuggerVersion, string ShimVersion, string ShimSha256, string HostRuntime, string Architecture,
    string TargetSha256, string SymbolsSha256, Guid TargetModule);
internal sealed record ReportLimits(int TimeoutSeconds, int OutputBytes)
{
    internal const int ReportBytes = 8 * 1024 * 1024;
    public int ReportCeilingBytes => ReportBytes;
    public int Threads => 64;
    public int FramesPerThread => 32;
    public int Frames => 256;
    public int Bindings => 4096;
    public int ValueDepth => 8;
    public int Children => 64;
    public int StringScalars => 4096;
}
internal sealed class ReportCapture
{
    public bool Complete => Omissions.Count == 0;
    public int? TotalThreads { get; set; }
    public List<string> Omissions { get; } = [];
    internal void Omit(string reason) { if (!Omissions.Contains(reason, StringComparer.Ordinal)) Omissions.Add(reason); }
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase)]
[JsonSerializable(typeof(BatchReport))]
internal sealed partial class BatchJson : JsonSerializerContext;
