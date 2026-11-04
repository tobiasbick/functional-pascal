using Fpas.Processes;

namespace Fpas.Debugging.Processes;

internal static class OwnedBackend
{
    internal static void Validate(string path)
    {
        if (!Path.IsPathFullyQualified(path) || !File.Exists(path))
            throw new ArgumentException("--netcoredbg requires an absolute path to an existing executable.");
        OwnedProcess.Validate(path);
    }

    internal static OwnedProcess Start(string path, string cwd)
    {
        Validate(path);
        return OwnedProcess.Start(path, ["--interpreter=vscode"], cwd);
    }
}
