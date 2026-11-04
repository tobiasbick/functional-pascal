namespace Fpas.Workspaces.Loading;

public sealed record ContextOptions(string Context, string Configuration = "Debug", bool NoRestore = false);

public sealed class ContextFailure(string code, string message, bool outputTruncated = false, IReadOnlyList<string>? candidates = null) : Exception(message)
{
    public string Code { get; } = code;
    public bool OutputTruncated { get; } = outputTruncated;
    public IReadOnlyList<string>? Candidates { get; } = candidates is null ? null : Array.AsReadOnly(candidates.ToArray());
}
