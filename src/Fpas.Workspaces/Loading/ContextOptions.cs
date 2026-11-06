namespace Fpas.Workspaces.Loading;

public enum ContextPreparation { BuildDependencies, ExistingInputs, SourceSnapshots }

public sealed record ContextOptions(string Context, string Configuration = "Debug", bool NoRestore = false,
    ContextPreparation Preparation = ContextPreparation.BuildDependencies);

public sealed class ContextFailure(string code, string message, bool outputTruncated = false, IReadOnlyList<string>? candidates = null) : Exception(message)
{
    public string Code { get; } = code;
    public bool OutputTruncated { get; } = outputTruncated;
    public IReadOnlyList<string>? Candidates { get; } = candidates is null ? null : Array.AsReadOnly(candidates.ToArray());
}
