namespace Fpas.Cli.Toolchain;

internal static class InstalledSdkInspection
{
    internal static async Task<(SdkCache Cache, bool Truncated)> Read(string? packages, CompilerInstallation? compiler,
        List<EnvironmentIssue> issues, CancellationToken token)
    {
        if (packages is null || compiler is null) return (new("unavailable", packages, []), false);
        var root = Path.Combine(packages, "fpas.net.sdk");
        var entries = new List<CachedSdk>(); var truncated = false;
        try
        {
            var expectedPath = Path.Combine(root, compiler.ExpectedSdkVersion.ToLowerInvariant());
            var expectedPresent = Directory.Exists(expectedPath);
            var others = Directory.Exists(root) ? Directory.EnumerateDirectories(root)
                .Where(path => !Path.GetFileName(path).Equals(compiler.ExpectedSdkVersion, StringComparison.OrdinalIgnoreCase)).Take(129).Order(StringComparer.Ordinal).ToArray() : [];
            var capacity = expectedPresent ? 127 : 128;
            if (others.Length > capacity) { truncated = true; issues.Add(new("warning", "inventory_limit", "Only 128 cached FPAS SDK versions are reported; the expected version is always inspected.", root)); }
            var paths = (expectedPresent ? new[] { expectedPath } : []).Concat(others.Take(capacity));
            foreach (var path in paths)
            {
                token.ThrowIfCancellationRequested();
                var version = Path.GetFileName(path);
                var expected = version.Equals(compiler.ExpectedSdkVersion, StringComparison.OrdinalIgnoreCase);
                string? compilerHash = null; string? coreHash = null;
                try
                {
                    var binary = Path.Combine(path, "tools", "compiler", "Fpas.Compiler.dll");
                    var core = Path.Combine(path, "tools", "compiler", "Fpas.Core.dll");
                    if (File.Exists(binary)) compilerHash = await EnvironmentInspection.Hash(binary, token);
                    if (File.Exists(core)) coreHash = await EnvironmentInspection.Hash(core, token);
                }
                catch (Exception exception) when (EnvironmentInspection.Recoverable(exception))
                { issues.Add(new(expected ? "error" : "warning", "sdk_unreadable", exception.Message, path)); }
                var compatibility = compilerHash is null || coreHash is null ? "incomplete" :
                    compilerHash == compiler.Compiler.Sha256 && coreHash == compiler.Core.Sha256 ? "matching" : "different";
                entries.Add(new(version, path, expected, compatibility, compilerHash, coreHash));
                if (expected && compatibility != "matching") issues.Add(new("error", "sdk_mismatch",
                    "The cached expected FPAS SDK has missing or different compiler/Core files. Deploy and resolve the matching SDK again.", path));
            }
            if (!entries.Any(entry => entry.Expected)) issues.Add(new("warning", "sdk_not_cached",
                "The expected FPAS SDK is not cached. Resolve it from the matching distribution before building projects.", root));
        }
        catch (Exception exception) when (EnvironmentInspection.Recoverable(exception))
        { issues.Add(new("error", "sdk_cache_unavailable", exception.Message, root)); return (new("unavailable", root, entries.AsReadOnly()), truncated); }
        return (new(truncated ? "partial" : entries.Count == 0 ? "empty" : "available", root, entries.AsReadOnly()), truncated);
    }
}
