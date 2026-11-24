// [Documentation](../../../docs/architecture/workspaces.md#context-preparation)
using Fpas.Workspaces.Loading;
using System.CommandLine;

namespace Fpas.Cli.Analysis;

internal sealed class ContextCommandOptions
{
    internal Argument<string> Context { get; } = new("context") { Description = "Explicit .fpasproj or .slnx path." };
    internal Option<string> Configuration { get; } = new("--configuration") { Description = "Evaluated build configuration.", DefaultValueFactory = _ => "Debug" };
    internal Option<bool> NoRestore { get; } = new("--no-restore") { Description = "Skip restore; dependency builds remain enabled." };
    internal Option<bool> ExistingInputs { get; } = new("--existing-inputs") { Description = "Use existing sources and dependency outputs without restore, builds or preparation hooks." };
    internal Option<int> Timeout { get; } = new("--timeout") { Description = "Overall deadline in seconds (1..3600).", DefaultValueFactory = _ => 120 };

    internal void AddTo(Command command, bool preparationOptions = true)
    {
        Range(Timeout, 1, 3600);
        command.Arguments.Add(Context);
        command.Options.Add(Configuration);
        if (preparationOptions)
        {
            command.Options.Add(NoRestore);
            command.Options.Add(ExistingInputs);
        }
        command.Options.Add(Timeout);
    }

    internal ContextOptions Read(ParseResult result)
        => new(Path.GetFullPath(result.GetValue(Context)!), result.GetValue(Configuration)!, result.GetValue(NoRestore),
            result.GetValue(ExistingInputs) ? ContextPreparation.ExistingInputs : ContextPreparation.BuildDependencies);

    internal static void Range(Option<int> option, int minimum, int maximum)
        => option.Range(minimum, maximum);
}
