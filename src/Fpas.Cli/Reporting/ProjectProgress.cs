using Fpas.Cli.Diagnostics;
using Fpas.Workspaces.Processes;

namespace Fpas.Cli.Reporting;

internal static class ProjectProgress
{
    internal static ProcessOutput? BuildOutput(bool json, string action, string context)
    {
        Phase(json, action, context);
        return json ? null : new(Console.Error, Console.Error);
    }

    internal static void Phase(bool json, string action, string context)
    {
        if (json || Console.IsErrorRedirected) return;
        Console.Error.WriteLine($"FPAS: {action} {DiagnosticRenderer.Safe(Path.GetFileName(context))}");
    }
}
