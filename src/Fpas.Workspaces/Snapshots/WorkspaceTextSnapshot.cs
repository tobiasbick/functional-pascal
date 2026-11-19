// [Documentation](../../../docs/architecture/workspaces.md#input-consistency-and-ownership)
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
    /// <summary>Gets the normalized absolute source path.</summary>
    public string Path { get; }
    /// <summary>Gets the captured UTF-16 text, including any byte-order-mark character.</summary>
    public string Text { get; }
    /// <summary>Gets the open-buffer version, or null for a disk capture.</summary>
    public int? Version { get; }
    /// <summary>Gets a SHA-256 identity preserving individual UTF-16 code units.</summary>
    public string ContentHash { get; }

    /// <summary>Captures text without reading its path from disk.</summary>
    /// <param name="path">Logical source path, normalized to an absolute path.</param>
    /// <param name="text">Source text to retain.</param>
    /// <param name="version">Open-buffer version, or null for disk text.</param>
    public TextDocumentSnapshot(string path, string text, int? version = null)
    {
        ArgumentNullException.ThrowIfNull(text);
        Path = System.IO.Path.GetFullPath(path); Text = text; Version = version;
        ContentHash = Convert.ToHexString(SHA256.HashData(MemoryMarshal.AsBytes(text.AsSpan())));
    }

    /// <summary>Encodes a new strict UTF-8 byte buffer without adding a byte-order mark.</summary>
    /// <returns>Independent bytes; unpaired surrogates cause an encoding failure.</returns>
    public byte[] EncodeUtf8() => Utf8.GetBytes(Text);
}

/// <summary>One immutable physical text per path, shared by all owning compilations.</summary>
public sealed class WorkspaceTextSnapshot
{
    internal static StringComparer Paths => OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
    private readonly IReadOnlyDictionary<string, TextDocumentSnapshot> byPath;
    /// <summary>Gets unique physical documents sorted by ordinal path.</summary>
    public IReadOnlyList<TextDocumentSnapshot> Documents { get; }

    /// <summary>Captures documents and rejects linked owners with different text or versions.</summary>
    /// <param name="documents">Documents to share by normalized path.</param>
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

    /// <summary>Captures compiler sources as unversioned text documents.</summary>
    /// <param name="sources">Already captured compiler inputs.</param>
    /// <returns>One consistent physical text per path.</returns>
    public static WorkspaceTextSnapshot Capture(IEnumerable<CompilationSource> sources)
        => new(sources.Select(source => new TextDocumentSnapshot(source.Path, source.Text)));

    /// <summary>Finds a captured document, rejecting paths outside this snapshot.</summary>
    /// <param name="path">Source path to normalize and look up.</param>
    /// <returns>The captured document.</returns>
    public TextDocumentSnapshot Document(string path) => byPath.TryGetValue(System.IO.Path.GetFullPath(path), out var document) ? document
        : throw new ContextFailure("invalid_fix", "An edit is outside the captured FPAS sources: " + path);
}
