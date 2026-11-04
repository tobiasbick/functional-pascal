using System.CommandLine;
using System.CommandLine.Help;

namespace Fpas.Cli;

internal static class CliHelp
{
    internal static async Task<int> WriteAsync(ParseResult result, HelpAction action)
    {
        var descriptions = result.CommandResult.Command.Subcommands.ToDictionary(command => command, command => command.Description);
        using var text = new StringWriter();
        try
        {
            foreach (var command in descriptions.Keys) command.Description = command.Description?.Split('\n')[0].Trim();
            action.MaxWidth = Console.IsOutputRedirected ? 110 : Math.Max(1, Math.Min(110, Console.WindowWidth));
            var exit = await result.InvokeAsync(new InvocationConfiguration { Output = text, EnableDefaultExceptionHandler = false });
            var names = descriptions.Keys.Select(command => command.Name).ToHashSet(StringComparer.Ordinal);
            var commands = false; var previousEntry = false;
            using var lines = new StringReader(text.ToString());
            while (lines.ReadLine() is { } line)
            {
                if (line == "Commands:") commands = true;
                var entry = commands && names.Contains(line.TrimStart().Split(' ', 2)[0]);
                if (entry && previousEntry) Console.WriteLine();
                Console.WriteLine(line);
                if (entry) previousEntry = true;
            }
            return exit;
        }
        finally { foreach (var (command, description) in descriptions) command.Description = description; }
    }
}
