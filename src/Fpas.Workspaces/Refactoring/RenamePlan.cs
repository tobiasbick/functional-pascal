using Fpas.Compiler;
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Queries;
using System.Security.Cryptography;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Workspaces.Refactoring;

public sealed record RenameTarget(string Project, string Kind, string Name, SourceSpan Declaration, bool PublicApi);
public sealed record RenameFile(string Path, string OriginalHash, string ReplacementHash, IReadOnlyList<DiagnosticEdit> Edits);
public sealed record RenamePlan(int Version, string Kind, string Context, string Configuration, string InputSnapshotId,
    SourceSelection Selection, string NewName, IReadOnlyList<RenameTarget> Targets, IReadOnlyList<RenameFile> Files,
    IReadOnlyList<string> Projects, IReadOnlyList<string> BinaryBoundaries, string Id = "")
{
    public bool ChangesPublicApi => Targets.Any(target => target.PublicApi);
    public int EditCount => Files.Sum(file => file.Edits.Count);
}

public static class RenamePlanJson
{
    public const int MaximumBytes = 4_194_304;
    public static byte[] Serialize(RenamePlan plan) => JsonSerializer.SerializeToUtf8Bytes(plan, RenameJson.Default.RenamePlan);
    public static string Identity(RenamePlan plan) => Convert.ToHexString(SHA256.HashData(Serialize(plan with { Id = "" })));
    public static RenamePlan Read(byte[] bytes)
    {
        if (bytes.Length > MaximumBytes) throw new ArgumentException("Rename plan exceeds 4 MiB.");
        var plan = JsonSerializer.Deserialize(bytes, RenameJson.Default.RenamePlan);
        if (plan is null || plan.Version != 1 || plan.Kind != "rename" || plan.Selection is null || string.IsNullOrEmpty(plan.Selection.File) ||
            string.IsNullOrEmpty(plan.Context) || string.IsNullOrEmpty(plan.Configuration) || string.IsNullOrEmpty(plan.NewName) ||
            string.IsNullOrEmpty(plan.Id) || string.IsNullOrEmpty(plan.InputSnapshotId) || plan.Targets is null || plan.Files is null ||
            plan.Projects is null || plan.BinaryBoundaries is null || plan.Targets.Any(target => target is null || target.Declaration is null ||
                string.IsNullOrEmpty(target.Project) || string.IsNullOrEmpty(target.Kind) || string.IsNullOrEmpty(target.Name)) ||
            plan.Files.Any(file => file is null || string.IsNullOrEmpty(file.Path) || string.IsNullOrEmpty(file.OriginalHash) ||
                string.IsNullOrEmpty(file.ReplacementHash) || file.Edits is null || file.Edits.Any(edit => edit is null || edit.Location is null ||
                    edit.ExpectedText is null || edit.NewText is null)) || Identity(plan) != plan.Id)
            throw new ArgumentException("Invalid or modified rename plan.");
        using var document = JsonDocument.Parse(bytes);
        var root = document.RootElement;
        if (root.TryGetProperty("editCount", out var count) && (count.ValueKind != JsonValueKind.Number || !count.TryGetInt32(out var edits) || edits != plan.EditCount) ||
            root.TryGetProperty("changesPublicApi", out var api) && (api.ValueKind is not (JsonValueKind.True or JsonValueKind.False) || api.GetBoolean() != plan.ChangesPublicApi))
            throw new ArgumentException("Rename plan summaries do not match its edits and declarations.");
        return plan;
    }
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase, UnmappedMemberHandling = JsonUnmappedMemberHandling.Disallow, WriteIndented = true)]
[JsonSerializable(typeof(RenamePlan))]
internal partial class RenameJson : JsonSerializerContext;
