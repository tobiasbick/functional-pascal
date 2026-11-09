namespace Fpas.Compiler.Analysis;

internal sealed class EditorSourceProjection
{
    private readonly CompilationSource original;
    private readonly SourceDocument document;
    private readonly int prefix;
    private readonly int editedEnd;
    private readonly int delta;

    internal EditorSourceProjection(CompilationSource original, string edited, CancellationToken token)
    {
        this.original = original;
        document = new(original.Path, original.Text, token);
        var shared = 0;
        while (shared < original.Text.Length && shared < edited.Length && original.Text[shared] == edited[shared])
        {
            if (shared % 1024 == 0) token.ThrowIfCancellationRequested();
            shared++;
        }
        prefix = shared;
        var suffix = 0;
        while (suffix < original.Text.Length - prefix && suffix < edited.Length - prefix && original.Text[^(suffix + 1)] == edited[^(suffix + 1)])
        {
            if (suffix % 1024 == 0) token.ThrowIfCancellationRequested();
            suffix++;
        }
        editedEnd = edited.Length - suffix;
        delta = original.Text.Length - edited.Length;
    }

    internal SourceSpan? Restore(SourceSpan? declaration)
    {
        if (declaration is null || !string.Equals(declaration.Path, original.Path, OperatingSystem.IsWindows() ? StringComparison.OrdinalIgnoreCase : StringComparison.Ordinal))
            return declaration;
        if (declaration.Offset + declaration.Length <= prefix) return document.Span(declaration.Offset, declaration.Length);
        return declaration.Offset >= editedEnd ? document.Span(declaration.Offset + delta, declaration.Length) : null;
    }
}
