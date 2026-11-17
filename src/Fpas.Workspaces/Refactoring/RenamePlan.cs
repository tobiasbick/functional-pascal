using Fpas.Compiler;
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Queries;
using System.Security.Cryptography;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Workspaces.Refactoring;

/// <summary>Describes one declaration grouped into a checked rename.</summary>
/// <param name="Project">Owning project path.</param>
/// <param name="Kind">Declaration kind.</param>
/// <param name="Name">Original unescaped identifier.</param>
/// <param name="Declaration">Exact declaration token span.</param>
/// <param name="PublicApi">Whether external consumers may observe this source API change.</param>
public sealed record RenameTarget(string Project, string Kind, string Name, SourceSpan Declaration, bool PublicApi);
/// <summary>Records exact original-offset edits and byte identities for one source file.</summary>
/// <param name="Path">Source path.</param>
/// <param name="OriginalHash">SHA-256 of original UTF-8 bytes.</param>
/// <param name="ReplacementHash">SHA-256 of replacement UTF-8 bytes.</param>
/// <param name="Edits">Ordered deduplicated edits.</param>
public sealed record RenameFile(string Path, string OriginalHash, string ReplacementHash, IReadOnlyList<DiagnosticEdit> Edits);
/// <summary>Stores a checked rename for explicit review and later regeneration.</summary>
/// <param name="Version">Plan schema version.</param>
/// <param name="Kind">Operation kind, rename.</param>
/// <param name="Context">Original absolute context path.</param>
/// <param name="Configuration">Original context configuration.</param>
/// <param name="InputSnapshotId">Captured input snapshot identity.</param>
/// <param name="Selection">Original checked source selection.</param>
/// <param name="NewName">New unescaped identifier.</param>
/// <param name="Targets">Grouped source declaration targets.</param>
/// <param name="Files">Changed source files and exact edits.</param>
/// <param name="Projects">Analyzed project paths.</param>
/// <param name="BinaryBoundaries">References outside editable source ownership.</param>
/// <param name="Id">Digest of canonical plan serialization with an empty identity field.</param>
public sealed record RenamePlan(int Version, string Kind, string Context, string Configuration, string InputSnapshotId,
    SourceSelection Selection, string NewName, IReadOnlyList<RenameTarget> Targets, IReadOnlyList<RenameFile> Files,
    IReadOnlyList<string> Projects, IReadOnlyList<string> BinaryBoundaries, string Id = "")
{
    /// <summary>Gets whether any grouped target changes a public source API.</summary>
    public bool ChangesPublicApi => Targets.Any(target => target.PublicApi);
    /// <summary>Gets the total number of source edits.</summary>
    public int EditCount => Files.Sum(file => file.Edits.Count);
}

/// <summary>Serializes bounded rename plans and validates their canonical identity.</summary>
public static class RenamePlanJson
{
    /// <summary>Maximum serialized plan size: four MiB.</summary>
    public const int MaximumBytes = 4_194_304;
    /// <summary>Serializes a plan using the canonical versioned JSON contract.</summary>
    /// <param name="plan">Plan to serialize.</param>
    /// <returns>A new UTF-8 JSON buffer.</returns>
    public static byte[] Serialize(RenamePlan plan) => JsonSerializer.SerializeToUtf8Bytes(plan, RenameJson.Default.RenamePlan);
    /// <summary>Hashes canonical serialization with the identity field cleared.</summary>
    /// <param name="plan">Plan whose semantic/input proposal is identified.</param>
    /// <returns>An uppercase SHA-256 digest.</returns>
    public static string Identity(RenamePlan plan) => Convert.ToHexString(SHA256.HashData(Serialize(plan with { Id = "" })));
    /// <summary>Reads a bounded plan and rejects malformed, modified or inconsistent summaries.</summary>
    /// <param name="bytes">UTF-8 JSON bytes, at most four MiB.</param>
    /// <returns>The validated typed plan; applying it still requires semantic regeneration.</returns>
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
