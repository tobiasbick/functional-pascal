using Fpas.Workspaces.Loading;
using System.CommandLine;

namespace Fpas.Cli.Analysis;

internal sealed class ContextCommandOptions
{
    internal Argument<string> Context { get; } = new("context") { Description = "Explicit .fpasproj or .slnx path." };
    internal Option<string> Configuration { get; } = new("--configuration") { Description = "Evaluated build configuration.", DefaultValueFactory = _ => "Debug" };
    internal Option<bool> NoRestore { get; } = new("--no-restore") { Description = "Skip restore; dependency builds remain enabled." };
    internal Option<int> Timeout { get; } = new("--timeout") { Description = "Overall deadline in seconds (1..3600).", DefaultValueFactory = _ => 120 };

    internal void AddTo(Command command)
    {
        Range(Timeout, 1, 3600);
        command.Arguments.Add(Context);
        foreach (var option in new Option[] { Configuration, NoRestore, Timeout }) command.Options.Add(option);
    }

    internal ContextOptions Read(ParseResult result)
        => new(Path.GetFullPath(result.GetValue(Context)!), result.GetValue(Configuration)!, result.GetValue(NoRestore));

    internal static void Range(Option<int> option, int minimum, int maximum)
        => option.Validators.Add(result =>
        {
            if (result.Tokens.LastOrDefault()?.Value is { } text && int.TryParse(text, out var value) && (value < minimum || value > maximum))
                result.AddError($"{option.Name} must be between {minimum} and {maximum}.");
        });
}
