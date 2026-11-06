using System.Reflection;

namespace Fpas.Debugging.Hosting;

internal sealed record WorkerLauncher(string Executable, string[] PrefixArguments)
{
    internal const string WorkerArgument = "--internal-debug-worker";
    internal const string BrokerArgument = "--internal-debug-target";

    internal static WorkerLauncher ForAssembly(string cli, string dotnet)
        => cli.EndsWith(".dll", StringComparison.OrdinalIgnoreCase) ? new(dotnet, [cli]) : new(cli, []);

    internal static WorkerLauncher Current()
    {
        var executable = Environment.ProcessPath ?? throw new IOException("The debugger launcher path is unavailable.");
        if (Path.GetFileNameWithoutExtension(executable).Equals("dotnet", StringComparison.OrdinalIgnoreCase))
            return new(executable, [Assembly.GetEntryAssembly()?.Location ?? throw new IOException("The CLI assembly path is unavailable.")]);
        return new(executable, []);
    }

    internal string[] Arguments(string pipe) => [.. PrefixArguments, WorkerArgument, pipe];
    internal string[] BrokerArguments(string pipe) => [.. PrefixArguments, BrokerArgument, pipe];
}
