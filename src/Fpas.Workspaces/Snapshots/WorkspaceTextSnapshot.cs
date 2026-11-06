using Fpas.Compiler;
using Fpas.Workspaces.Loading;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Text;

namespace Fpas.Workspaces.Snapshots;

/// <summary>Captured UTF-16 source text, independent of its current filesystem contents.</summary>
public sealed class TextDocumentSnapshot
{
    internal static readonly UTF8Encoding Utf8 = new(false, true);
    public string Path { get; }
    public string Text { get; }
    public int? Version { get; }
    public string ContentHash { get; }

    public TextDocumentSnapshot(string path, string text, int? version = null)
    {
        ArgumentNullException.ThrowIfNull(text);
        Path = System.IO.Path.GetFullPath(path); Text = text; Version = version;
        ContentHash = Convert.ToHexString(SHA256.HashData(MemoryMarshal.AsBytes(text.AsSpan())));
    }

    public byte[] EncodeUtf8() => Utf8.GetBytes(Text);
}

/// <summary>One immutable physical text per path, shared by all owning compilations.</summary>
public sealed class WorkspaceTextSnapshot
{
    internal static StringComparer Paths => OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
    private readonly IReadOnlyDictionary<string, TextDocumentSnapshot> byPath;
    public IReadOnlyList<TextDocumentSnapshot> Documents { get; }

    public WorkspaceTextSnapshot(IEnumerable<TextDocumentSnapshot> documents)
    {
        var captured = new Dictionary<string, TextDocumentSnapshot>(Paths);
        foreach (var document in documents)
        {
            if (captured.TryGetValue(document.Path, out var previous) && (previous.Text != document.Text || previous.Version != document.Version))
                throw new ContextFailure("edit_conflict", "Linked source owners have different captured contents or versions: " + document.Path);
            captured[document.Path] = document;
        }
        Documents = Array.AsReadOnly(captured.Values.OrderBy(document => document.Path, StringComparer.Ordinal).ToArray());
        byPath = captured;
    }

    public static WorkspaceTextSnapshot Capture(IEnumerable<CompilationSource> sources)
        => new(sources.Select(source => new TextDocumentSnapshot(source.Path, source.Text)));

    public TextDocumentSnapshot Document(string path) => byPath.TryGetValue(System.IO.Path.GetFullPath(path), out var document) ? document
        : throw new ContextFailure("invalid_fix", "An edit is outside the captured FPAS sources: " + path);
}
