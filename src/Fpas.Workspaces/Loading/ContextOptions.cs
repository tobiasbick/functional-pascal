// [Documentation](../../../docs/architecture/workspaces.md#context-preparation)
namespace Fpas.Workspaces.Loading;

/// <summary>Selects which project preparation effects workspace loading permits.</summary>
public enum ContextPreparation
{
    /// <summary>Allow restore and dependency builds before capturing inputs.</summary>
    BuildDependencies,
    /// <summary>Consume existing generated sources and dependency outputs without preparation.</summary>
    ExistingInputs,
    /// <summary>Use current FPAS sources and private dependency images without building them.</summary>
    SourceSnapshots
}

/// <summary>Specifies an explicit project or solution and its preparation policy.</summary>
/// <param name="Context">Project or solution path.</param>
/// <param name="Configuration">Selected build or solution configuration.</param>
/// <param name="NoRestore">Whether to skip restore; permitted dependency builds remain enabled.</param>
/// <param name="Preparation">Allowed preparation effects.</param>
public sealed record ContextOptions(string Context, string Configuration = "Debug", bool NoRestore = false,
    ContextPreparation Preparation = ContextPreparation.BuildDependencies);

/// <summary>Reports a workspace failure with a stable machine-readable code.</summary>
/// <param name="code">Failure category.</param>
/// <param name="message">Human-readable explanation.</param>
/// <param name="outputTruncated">Whether supporting process output exceeded its budget.</param>
/// <param name="candidates">Explicit project choices for an ambiguous selection, when available.</param>
/// <param name="project">Affected project path, when known.</param>
/// <param name="inputPath">Exact missing or changed input path, when known.</param>
/// <param name="nextAction">Specific recovery action overriding the category default, when available.</param>
/// <param name="expectedSnapshotId">Requested input snapshot, when a snapshot precondition fails.</param>
/// <param name="actualSnapshotId">Captured input snapshot, when a snapshot precondition fails.</param>
public sealed class ContextFailure(string code, string message, bool outputTruncated = false, IReadOnlyList<string>? candidates = null,
    string? project = null, string? inputPath = null, string? nextAction = null,
    string? expectedSnapshotId = null, string? actualSnapshotId = null) : Exception(message)
{
    /// <summary>Gets the failure category.</summary>
    public string Code { get; } = code;
    /// <summary>Gets whether supporting process output was truncated.</summary>
    public bool OutputTruncated { get; } = outputTruncated;
    /// <summary>Gets detached candidate paths, or null when no choices apply.</summary>
    public IReadOnlyList<string>? Candidates { get; } = candidates is null ? null : Array.AsReadOnly(candidates.ToArray());

    /// <summary>Gets the affected project path when known.</summary>
    public string? Project { get; } = project;
    /// <summary>Gets the exact missing or changed input path when known.</summary>
    public string? InputPath { get; } = inputPath;
    /// <summary>Gets a machine-readable recovery action when known.</summary>
    public string? NextAction { get; } = nextAction ?? DefaultAction(code);

    /// <summary>Gets the requested input snapshot when a snapshot precondition fails.</summary>
    public string? ExpectedSnapshotId { get; } = expectedSnapshotId;
    /// <summary>Gets the captured input snapshot when a snapshot precondition fails.</summary>
    public string? ActualSnapshotId { get; } = actualSnapshotId;

    internal static string? DefaultAction(string code) => code switch
    {
        "inputs_changed" => "refresh_snapshot",
        "stale_plan" => "recreate_plan",
        "preparation_required" => "review_project",
        _ => null
    };
}
