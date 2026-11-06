namespace Fpas.Debugging.Symbols;

internal sealed record SymbolProbe(ModuleSymbols? Symbols, Core.DebugProblem? Error = null);

internal sealed class SymbolCatalog(ModuleSymbols primary) : IDisposable
{
    private readonly Dictionary<string, ModuleSymbols?> modules = new(OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal)
    { [primary.Path] = primary };

    internal ModuleSymbols? Open(string path, Guid loadedIdentity)
    {
        if (!System.IO.Path.IsPathFullyQualified(path) || !File.Exists(path)) return null;
        path = System.IO.Path.GetFullPath(path);
        if (modules.TryGetValue(path, out var existing))
        {
            if (existing is not null) Verify(existing, loadedIdentity);
            return existing;
        }
        if (modules.Count >= 1024) throw new IOException("The debugger module catalog limit was reached.");
        var symbols = File.Exists(System.IO.Path.ChangeExtension(path, ".pdb")) ? new ModuleSymbols(path) : null;
        try { if (symbols is not null) Verify(symbols, loadedIdentity); }
        catch { symbols?.Dispose(); throw; }
        modules.Add(path, symbols);
        return symbols;
    }

    internal SymbolProbe Probe(string path, Guid loadedIdentity)
    {
        try { return new(Open(path, loadedIdentity)); }
        catch (Exception exception) when (exception is IOException or BadImageFormatException or InvalidOperationException or ArgumentException or UnauthorizedAccessException or Core.DebugException)
        {
            return new(null, new(exception is Core.DebugException failure ? failure.Code : "invalid_symbols", exception.Message));
        }
    }

    private static void Verify(ModuleSymbols symbols, Guid loadedIdentity)
    {
        if (symbols.Identity != loadedIdentity)
            throw new Core.DebugException("module_mismatch", "The assembly on disk does not match the module loaded by CoreCLR.");
    }

    internal void Remove(string path)
    {
        if (System.IO.Path.IsPathFullyQualified(path) && modules.Remove(System.IO.Path.GetFullPath(path), out var symbols)) symbols?.Dispose();
    }

    public void Dispose() { foreach (var symbols in modules.Values) symbols?.Dispose(); modules.Clear(); }
}
