using Fpas.Workspaces.Processes;

namespace Fpas.Cli.Profiling;

internal static class DiagnosticTools
{
    internal const string Version = "10.0.750501";
    internal static string DefaultDirectory => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "fpas", "diagnostics", Version);
    internal static readonly string[] Names = ["dotnet-trace", "dotnet-counters", "dotnet-dump"];

    internal static string Assembly(string directory, string name)
    {
        var package = Path.Combine(Path.GetFullPath(directory), ".store", name, Version);
        var assemblies = Directory.Exists(package) ? Directory.GetFiles(package, name + ".dll", SearchOption.AllDirectories)
            .Where(path => Path.GetFileName(Path.GetDirectoryName(path)) == "any").ToArray() : [];
        if (assemblies.Length != 1) throw new IOException("Prepare the pinned tools first: fpas profile setup --tools-directory \"" + directory + "\".");
        return assemblies[0];
    }

    internal static async Task<BuildResult> Setup(string directory, CancellationToken token)
    {
        directory = Path.GetFullPath(directory); Directory.CreateDirectory(directory);
        await using var lease = await Lock(directory, token);
        var configuration = Path.Combine(directory, "diagnostic-tools.nuget.config");
        const string contents = "<configuration><packageSources><clear/><add key=\"nuget.org\" value=\"https://api.nuget.org/v3/index.json\"/></packageSources></configuration>";
        if (!File.Exists(configuration)) await File.WriteAllTextAsync(configuration, contents, token);
        var output = new System.Text.StringBuilder(); var error = new System.Text.StringBuilder();
        foreach (var name in Names)
        {
            try { _ = Assembly(directory, name); continue; } catch (IOException) { }
            var result = await BuildProcess.Run(Environment.CurrentDirectory, ["tool", "install", name, "--version", Version, "--tool-path", directory, "--configfile", configuration], token);
            output.Append(result.Output); error.Append(result.Error);
            if (result.ExitCode != 0 || result.Truncated) return result with { Output = output.ToString(), Error = error.ToString() };
        }
        return new(0, output.ToString(), error.ToString(), false);
    }

    private static async Task<FileStream> Lock(string directory, CancellationToken token)
    {
        var path = Path.Combine(directory, ".setup.lock");
        while (true)
        {
            token.ThrowIfCancellationRequested();
            try { return new FileStream(path, FileMode.OpenOrCreate, FileAccess.ReadWrite, FileShare.None); }
            catch (IOException) when (File.Exists(path)) { await Task.Delay(50, token); }
        }
    }

    internal static Task<BuildResult> Run(string directory, string name, string cwd, IReadOnlyList<string> arguments, CancellationToken token)
        => BuildProcess.Run(cwd, [Assembly(directory, name), .. arguments], token, buildEnvironment: false);
}
