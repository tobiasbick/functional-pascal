// [Documentation](../../../docs/pascal/tools/refactoring.md#validation-and-plans)
using Fpas.Compiler;
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Queries;
using System.Security.Cryptography;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Workspaces.Refactoring;

/// <summary>Describes one declaration grouped into a checked refactoring.</summary>
/// <param name="Project">Owning project path.</param>
/// <param name="Kind">Declaration kind.</param>
/// <param name="Name">Original unescaped identifier.</param>
/// <param name="Declaration">Exact declaration token span.</param>
/// <param name="PublicApi">Whether external consumers may observe this source API change.</param>
public sealed record RefactorTarget(string Project, string Kind, string Name, SourceSpan Declaration, bool PublicApi);
/// <summary>Records exact original-offset edits and byte identities for one source file.</summary>
/// <param name="Path">Source path.</param>
/// <param name="OriginalHash">SHA-256 of original UTF-8 bytes.</param>
/// <param name="ReplacementHash">SHA-256 of replacement UTF-8 bytes.</param>
/// <param name="Edits">Ordered deduplicated edits.</param>
public sealed record RefactorPlanFile(string Path, string OriginalHash, string ReplacementHash, IReadOnlyList<DiagnosticEdit> Edits);
/// <summary>Captures the explicit inputs for adding a value parameter.</summary>
/// <param name="Name">New unescaped identifier.</param>
/// <param name="Type">FPAS type written in the declaration.</param>
/// <param name="Argument">FPAS value expression supplied at each call.</param>
public sealed record RefactorParameter(string Name, string Type, string Argument);
/// <summary>Identifies the exclusive end of a selection in its source file.</summary>
/// <param name="Line">One-based source line.</param>
/// <param name="Column">One-based UTF-16 source column.</param>
public sealed record RefactorPosition(int Line, int Column);

/// <summary>Supplies a replacement argument at one direct call to the selected routine.</summary>
/// <param name="Call">One-based UTF-16 coordinates of the called name.</param>
/// <param name="Expression">FPAS argument expression without a label or mode keyword.</param>
/// <param name="File">Caller source file, or null for the routine declaration's file.</param>
public sealed record RefactorArgumentOverride(RefactorPosition Call, string Expression,
    [property: JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)] string? File = null);

/// <summary>Describes a parameter type or mode change, or a new order for a routine.</summary>
/// <param name="Type">New FPAS type, or null to retain the current type.</param>
/// <param name="Mode">New mode (value, in, ref or out), or null to retain it.</param>
/// <param name="Arguments">Explicit replacements at selected direct call sites.</param>
/// <param name="Order">All unescaped parameter names in requested order, or null for a type or mode change.</param>
public sealed record RefactorSignatureChange(string? Type, string? Mode, IReadOnlyList<RefactorArgumentOverride> Arguments,
    [property: JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)] IReadOnlyList<string>? Order = null)
{
    internal bool IsValid => Order is not null
        ? Type is null && Mode is null && Arguments is { Count: 0 } && Order is { Count: > 0 and <= 1000 } &&
            Order.All(name => name is { Length: > 0 and <= 1024 } && !string.IsNullOrWhiteSpace(name)) &&
            Order.Distinct(StringComparer.Ordinal).Count() == Order.Count
        : (Type is not null || Mode is not null) &&
        (Type is null || Type is { Length: <= 1024 } && !string.IsNullOrWhiteSpace(Type)) &&
        Mode is null or "value" or "in" or "ref" or "out" &&
        Arguments is { Count: <= 1000 } && Arguments.All(argument =>
            argument is { Call: { Line: > 0, Column: > 0 }, Expression: { Length: <= 8192 } } &&
            !string.IsNullOrWhiteSpace(argument.Expression) && (argument.File is null || !string.IsNullOrWhiteSpace(argument.File)));
}
/// <summary>Supplies explicit behavior for a required protocol member.</summary>
/// <param name="Key">Member name, with an overload number when required.</param>
/// <param name="Value">FPAS statements or an initializer, or the existing method name for forwarding.</param>
/// <param name="Forward">Whether to forward to an existing record method.</param>
public sealed record RefactorProtocolMember(string Key, string Value, bool Forward = false);

/// <summary>Identifies a protocol and supplies any required implementations.</summary>
/// <param name="Type">The protocol type as written in the destination unit.</param>
/// <param name="Members">Explicit bodies, initializers, and forwarding requests.</param>
public sealed record RefactorProtocol(string Type, IReadOnlyList<RefactorProtocolMember> Members)
{
    internal bool IsValid => Type is { Length: <= 1024 } && !string.IsNullOrWhiteSpace(Type) &&
        Members is { Count: <= 1000 } && Members.All(member => member is { Key.Length: > 0 and <= 1024, Value.Length: <= 8192 } &&
            !string.IsNullOrWhiteSpace(member.Key) && !string.IsNullOrWhiteSpace(member.Value)) &&
        Members.Select(member => member.Key).Distinct(StringComparer.Ordinal).Count() == Members.Count;
}

/// <summary>Stores a checked refactoring for explicit review and later regeneration.</summary>
/// <param name="Version">Plan schema version.</param>
/// <param name="Kind">Refactoring operation name.</param>
/// <param name="Context">Original absolute context path.</param>
/// <param name="Configuration">Original context configuration.</param>
/// <param name="InputSnapshotId">Captured input snapshot identity.</param>
/// <param name="Selection">Original checked source selection.</param>
/// <param name="NewName">New unescaped identifier for rename or extraction, otherwise null.</param>
/// <param name="Targets">Grouped source declaration targets.</param>
/// <param name="Files">Changed source files and exact edits.</param>
/// <param name="Projects">Analyzed project paths.</param>
/// <param name="BinaryBoundaries">References outside editable source ownership.</param>
/// <param name="Id">Digest of canonical plan serialization with an empty identity field.</param>
public sealed record RefactorPlan(int Version, string Kind, string Context, string Configuration, string InputSnapshotId,
    SourceSelection Selection, string? NewName, IReadOnlyList<RefactorTarget> Targets, IReadOnlyList<RefactorPlanFile> Files,
    IReadOnlyList<string> Projects, IReadOnlyList<string> BinaryBoundaries, string Id = "")
{
    /// <summary>Gets the requested value parameter and call argument for an add-parameter plan.</summary>
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public RefactorParameter? AddedParameter { get; init; }

    /// <summary>Gets the exclusive end of the selected expression for extraction.</summary>
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public RefactorPosition? SelectionEnd { get; init; }

    /// <summary>Gets the behavior and limitations reported by the refactoring.</summary>
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public IReadOnlyList<string>? Notes { get; init; }

    /// <summary>Gets the requested parameter change and explicit argument replacements.</summary>
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public RefactorSignatureChange? SignatureChange { get; init; }

    /// <summary>Gets whether the request confirms the selected context contains all public signature consumers.</summary>
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingDefault)]
    public bool ClosedContext { get; init; }

    /// <summary>Gets the requested protocol and any explicit required implementations.</summary>
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public RefactorProtocol? Protocol { get; init; }

    internal bool HasValidOperation => Version == 1 && (Kind == "extract-variable" || SelectionEnd is null) &&
        (!ClosedContext || Kind is "add-parameter" or "remove-parameter" or "change-signature") &&
        (Kind == "change-signature" || SignatureChange is null) && (Kind == "implement-protocol" || Protocol is null) && Kind switch
    {
        "rename" => AddedParameter is null && !string.IsNullOrEmpty(NewName),
        "name-arguments" or "organize-imports" => AddedParameter is null && NewName is null && Targets is { Count: 0 },
        "remove-parameter" => AddedParameter is null && NewName is null && Targets is { Count: > 0 },
        "add-parameter" => NewName is null && Targets is { Count: > 0 } && AddedParameter is { } parameter &&
            !string.IsNullOrWhiteSpace(parameter.Name) && !string.IsNullOrWhiteSpace(parameter.Type) &&
            !string.IsNullOrWhiteSpace(parameter.Argument),
        "inline-variable" => AddedParameter is null && NewName is null && Targets is { Count: > 0 },
        "extract-variable" => AddedParameter is null && !string.IsNullOrWhiteSpace(NewName) && Targets is { Count: 0 } &&
            SelectionEnd is { Line: > 0, Column: > 0 },
        "change-signature" => AddedParameter is null && NewName is null && Targets is { Count: > 0 } &&
            SignatureChange is { IsValid: true },
        "implement-protocol" => AddedParameter is null && NewName is null && Targets is { Count: > 0 } && Protocol is { IsValid: true },
        _ => false
    };

    /// <summary>Gets whether any grouped target changes a public source API.</summary>
    public bool ChangesPublicApi => Targets.Any(target => target.PublicApi);
    /// <summary>Gets the total number of source edits.</summary>
    public int EditCount => Files.Sum(file => file.Edits.Count);
}

/// <summary>Serializes bounded refactoring plans and validates their canonical identity.</summary>
public static class RefactorPlanJson
{
    /// <summary>Maximum serialized plan size: four MiB.</summary>
    public const int MaximumBytes = 4_194_304;
    /// <summary>Serializes a plan using the canonical versioned JSON contract.</summary>
    /// <param name="plan">Plan to serialize.</param>
    /// <returns>A new UTF-8 JSON buffer.</returns>
    public static byte[] Serialize(RefactorPlan plan) => JsonSerializer.SerializeToUtf8Bytes(plan, RefactorPlanContext.Default.RefactorPlan);
    /// <summary>Hashes canonical serialization with the identity field cleared.</summary>
    /// <param name="plan">Plan whose semantic/input proposal is identified.</param>
    /// <returns>An uppercase SHA-256 digest.</returns>
    public static string Identity(RefactorPlan plan) => Convert.ToHexString(SHA256.HashData(Serialize(plan with { Id = "" })));
    /// <summary>Reads a bounded plan and rejects malformed, modified or inconsistent summaries.</summary>
    /// <param name="bytes">UTF-8 JSON bytes, at most four MiB.</param>
    /// <returns>The validated typed plan; applying it still requires semantic regeneration.</returns>
    public static RefactorPlan Read(byte[] bytes)
    {
        if (bytes.Length > MaximumBytes) throw new ArgumentException("Refactoring plan exceeds 4 MiB.");
        var plan = JsonSerializer.Deserialize(bytes, RefactorPlanContext.Default.RefactorPlan);
        if (plan is null || !plan.HasValidOperation || plan.Selection is null || string.IsNullOrEmpty(plan.Selection.File) ||
            string.IsNullOrEmpty(plan.Context) || string.IsNullOrEmpty(plan.Configuration) ||
            string.IsNullOrEmpty(plan.Id) || string.IsNullOrEmpty(plan.InputSnapshotId) || plan.Targets is null || plan.Files is null ||
            plan.Projects is null || plan.BinaryBoundaries is null || plan.Notes?.Any(note => note is null) == true ||
            plan.Targets.Any(target => target is null || target.Declaration is null ||
                string.IsNullOrEmpty(target.Project) || string.IsNullOrEmpty(target.Kind) || string.IsNullOrEmpty(target.Name)) ||
            plan.Files.Any(file => file is null || string.IsNullOrEmpty(file.Path) || string.IsNullOrEmpty(file.OriginalHash) ||
                string.IsNullOrEmpty(file.ReplacementHash) || file.Edits is null || file.Edits.Any(edit => edit is null || edit.Location is null ||
                    edit.ExpectedText is null || edit.NewText is null)) || Identity(plan) != plan.Id)
            throw new ArgumentException("Invalid or modified refactoring plan.");
        using var document = JsonDocument.Parse(bytes);
        var root = document.RootElement;
        if (root.TryGetProperty("editCount", out var count) && (count.ValueKind != JsonValueKind.Number || !count.TryGetInt32(out var edits) || edits != plan.EditCount) ||
            root.TryGetProperty("changesPublicApi", out var api) && (api.ValueKind is not (JsonValueKind.True or JsonValueKind.False) || api.GetBoolean() != plan.ChangesPublicApi))
            throw new ArgumentException("Refactoring plan summaries do not match its edits and declarations.");
        return plan;
    }
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase, UnmappedMemberHandling = JsonUnmappedMemberHandling.Disallow, WriteIndented = true)]
[JsonSerializable(typeof(RefactorPlan))]
internal partial class RefactorPlanContext : JsonSerializerContext;
