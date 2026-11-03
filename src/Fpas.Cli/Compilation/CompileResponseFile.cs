using System.CommandLine;
using System.CommandLine.Parsing;
using System.Text.Json;

namespace Fpas.Cli.Compilation;

internal static class CompileResponseFile
{
    internal static ParseResult Expand(ParseResult parsed, CompileCommand compile, ParserConfiguration configuration)
    {
        if (parsed.GetResult(compile.ResponseFile) is not { } response)
            return parsed;
        if (response.Tokens.Count != 1 || response.IdentifierTokenCount != 1 || parsed.UnmatchedTokens.Count > 0 ||
            parsed.CommandResult.Children.Any(child => child switch
            {
                OptionResult option => !option.Implicit && option != response,
                ArgumentResult argument => !argument.Implicit,
                _ => false
            }))
            throw new ArgumentException("--response-file replaces all compile arguments; use it alone with one JSON file path.");
        string[] arguments;
        try
        {
            arguments = JsonSerializer.Deserialize<string[]>(File.ReadAllText(response.GetValueOrDefault<string>()!))
                ?? throw new JsonException("Missing arguments.");
        }
        catch (JsonException exception)
        {
            throw new ArgumentException("Invalid JSON argument array in response file.", exception);
        }
        if (arguments.Any(argument => argument is null))
            throw new ArgumentException("Response file arguments must be non-null strings.");
        var expanded = compile.Command.Parse(arguments, configuration);
        return expanded;
    }
}
