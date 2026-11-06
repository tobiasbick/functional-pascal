using Fpas.LanguageServer;
using System.CommandLine;

namespace Fpas.Cli.LanguageServer;

internal sealed class LspCommand
{
    internal Command Command { get; } = new("lsp", """
        Serve LSP 3.17 over stdio. Stdout contains protocol frames only.
        Select an existing project/solution through initializationOptions.context;
        initializationOptions.configuration defaults to Debug. Positions use UTF-16.
        Analysis uses existing SDK/restore assets and current FPAS sources.
        Restore and foreign-project builds are explicit user operations.
        Exits: 0 clean EOF or shutdown/exit, 1 protocol/setup failure, 2 arguments, 130 cancellation.

        Example:
          fpas lsp
        """);
    internal LspCommand() => Command.SetAction((_, token) => LanguageServerApplication.RunAsync(
        Console.OpenStandardInput(), Console.OpenStandardOutput(), Console.Error, token));
    internal static void ArgumentError(string message) => Console.Error.WriteLine(message + " See fpas lsp --help.");
}
