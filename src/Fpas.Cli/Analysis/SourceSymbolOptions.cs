// [Documentation](../../../docs/pascal/program-structure/cli.md#reuse-a-found-symbol)
using System.CommandLine;

namespace Fpas.Cli.Analysis;

internal static class SourceSymbolOptions
{
    internal static Option<string> Create() => new("--symbol") { Description = "Exact symbol from query find. Supply its --project and --expected-snapshot." };

    internal static void Bind(Command command, Option<string> symbol, Option<string> project, Option<string> snapshot,
        Option<string> file, Option<int> line, Option<int> column)
    {
        file.Required = line.Required = column.Required = false;
        command.Options.Add(symbol);
        command.ExactlyOne(file, symbol);
        command.Requires(symbol, project, result => result.GetValue(symbol) is not null, result => result.GetValue(project) is not null);
        command.Requires(symbol, snapshot, result => result.GetValue(symbol) is not null, result => result.GetValue(snapshot) is not null);
        foreach (var coordinate in new[] { line, column })
        {
            command.Requires(file, coordinate, result => result.GetValue(file) is not null, result => result.GetResult(coordinate) is { IdentifierTokenCount: > 0 });
            command.Excludes(symbol, coordinate, result => result.GetValue(symbol) is not null, result => result.GetResult(coordinate) is { IdentifierTokenCount: > 0 });
        }
    }
}
