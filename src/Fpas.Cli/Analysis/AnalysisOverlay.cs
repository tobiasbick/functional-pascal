// [Documentation](../../../docs/pascal/program-structure/cli.md#check-a-draft-in-memory)
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Live;
using Fpas.Workspaces.Loading;
using System.CommandLine;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Cli.Analysis;

internal static class AnalysisOverlay
{
    internal const int FileLimit = 64;
    private const int InputLimit = 16 * 1024 * 1024;
    private const int SourceLimit = 4 * 1024 * 1024;
    private static StringComparer Paths => OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;

    internal static Option<string> FileOption() => new("--overlay")
        { Description = "Analyze replacement and new source texts from this JSON file in memory." };

    internal static async Task<T> RunAsync<T>(ContextOptions input, string manifest, TimeSpan timeout,
        Func<WorkspaceAnalysis, IReadOnlyList<string>, ContextFailure?, T> use, CancellationToken token)
    {
        var path = Path.GetFullPath(manifest);
        var (files, creations, fingerprint) = await ReadAsync(path, token);
        if (creations.Count != 0)
        {
            var captured = await FileWorkspace.OpenDraftAsync(input, token, creations);
            var analysis = captured.AnalyzeDraft(files, creations, token);
            var paths = files.Keys.Concat(creations.Select(file => file.Path)).Order(StringComparer.Ordinal).ToArray();
            var checkedDraft = use(analysis, paths, null);
            await captured.VerifyAsync(token);
            await VerifyAsync(path, fingerprint, token);
            return checkedDraft;
        }
        await using var workspace = new LiveWorkspace(input, new(DocumentBytes: SourceLimit, OpenBytes: InputLimit, WorkTimeout: timeout));
        foreach (var (file, text) in files)
            workspace.Open(file, text, version: 1);

        var snapshot = await workspace.AnalyzeAsync(token);
        var names = files.Keys.Order(StringComparer.Ordinal).ToArray();
        if (snapshot.Unowned.Count != 0)
        {
            var failed = snapshot.Unowned.Select(item => item.Document.Path).ToArray();
            var blocked = snapshot.Analysis.Context.Issues.Count > 0;
            var error = new ContextFailure(blocked ? "analysis_unavailable" : "invalid_argument",
                blocked ? "Cannot verify draft file ownership until project preparation succeeds." : "Draft files must belong to the evaluated FPAS context.",
                candidates: failed);
            return use(snapshot.Analysis, names, error);
        }
        var result = use(snapshot.Analysis, names, null);
        await workspace.VerifyAsync(snapshot, token);
        await VerifyAsync(path, fingerprint, token);
        return result;
    }

    internal static async Task VerifyAsync(string path, byte[] fingerprint, CancellationToken token)
    {
        var verified = await BoundedFile.ReadAsync(path, InputLimit, token);
        if (!SHA256.HashData(verified).AsSpan().SequenceEqual(fingerprint))
            throw new ContextFailure("inputs_changed", "The overlay manifest changed during analysis.", inputPath: path);
    }

    internal static async Task<OverlayInput> ReadAsync(string path, CancellationToken token)
    {
        var original = await BoundedFile.ReadAsync(path, InputLimit, token);
        OverlayEntry[]? entries;
        try { entries = JsonSerializer.Deserialize(WithoutBom(original), OverlayJson.Default.OverlayEntryArray); }
        catch (Exception error) when (error is JsonException or InvalidOperationException) { throw Invalid(path, error.Message); }
        if (entries is not { Length: > 0 and <= FileLimit })
            throw Invalid(path, $"Supply an array of 1 to {FileLimit} draft source files.");
        var files = new Dictionary<string, string>(Paths);
        var creations = new List<DraftCreation>();
        var seen = new HashSet<string>(Paths);
        foreach (var entry in entries)
        {
            if (entry is null || string.IsNullOrWhiteSpace(entry.File) || entry.Text is null)
                throw Invalid(path, "Each draft entry requires file and text.");
            string file;
            try { file = Path.GetFullPath(entry.File); }
            catch (ArgumentException error) { throw Invalid(path, error.Message); }
            if (Encoding.UTF8.GetByteCount(entry.Text) > SourceLimit)
                throw Invalid(file, "Draft source exceeds 4 MiB.");
            if (!seen.Add(file))
                throw Invalid(file, "Supply one draft entry per source path.");
            if (entry.Project is null) files.Add(file, entry.Text);
            else
            {
                if (string.IsNullOrWhiteSpace(entry.Project) || !Path.GetExtension(file).Equals(".fpas", StringComparison.OrdinalIgnoreCase) ||
                    !Path.GetExtension(entry.Project).Equals(".fpasproj", StringComparison.OrdinalIgnoreCase))
                    throw Invalid(file, "A new .fpas source requires an explicit owning .fpasproj path.");
                creations.Add(new(file, Path.GetFullPath(entry.Project), entry.Text));
            }
        }
        return new(files, creations.AsReadOnly(), SHA256.HashData(original));
    }

    private static ReadOnlySpan<byte> WithoutBom(byte[] bytes) => bytes.AsSpan().StartsWith<byte>([0xef, 0xbb, 0xbf]) ? bytes.AsSpan(3) : bytes;
    private static ContextFailure Invalid(string input, string message) => new("invalid_argument", message, inputPath: input);
}

[JsonUnmappedMemberHandling(JsonUnmappedMemberHandling.Disallow)]
internal sealed record OverlayEntry(string File, string Text, string? Project = null);
internal sealed record OverlayInput(Dictionary<string, string> Files, IReadOnlyList<DraftCreation> Creations, byte[] Fingerprint);

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase)]
[JsonSerializable(typeof(OverlayEntry[]))]
internal partial class OverlayJson : JsonSerializerContext;
