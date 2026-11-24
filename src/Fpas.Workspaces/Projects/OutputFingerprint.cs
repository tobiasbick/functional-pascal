// [Documentation](../../../docs/pascal/program-structure/project-commands.md#resolve-a-runnable-target)
using System.Security.Cryptography;
using System.Text;

namespace Fpas.Workspaces.Projects;

/// <summary>Identifies a managed output together with its immediate companion files.</summary>
public static class OutputFingerprint
{
    /// <summary>Hashes the assembly, portable symbols, dependency manifest and runtime configuration.</summary>
    /// <param name="assembly">Absolute path to the managed output.</param>
    /// <param name="token">Cancels capture between files.</param>
    /// <returns>A SHA-256 hexadecimal identity including the presence or absence of each file.</returns>
    public static string Compute(string assembly, CancellationToken token)
    {
        var paths = new[] { assembly, Path.ChangeExtension(assembly, ".pdb"),
            Path.ChangeExtension(assembly, ".deps.json"), Path.ChangeExtension(assembly, ".runtimeconfig.json") };
        return Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(string.Join(':', paths.Select(path =>
        {
            token.ThrowIfCancellationRequested();
            if (!File.Exists(path)) return "-";
            using var content = File.OpenRead(path);
            return Convert.ToHexString(SHA256.HashData(content));
        })))));
    }
}
