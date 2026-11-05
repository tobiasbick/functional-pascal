using System.CommandLine;
using System.CommandLine.Parsing;

namespace Fpas.Cli;

internal static class CommandArguments
{
    internal static void Validate(ParseResult result, Argument<string[]> paths, bool standardInput = false)
    {
        var pathTokens = result.GetResult(paths)?.Tokens;
        foreach (var token in result.Tokens.TakeWhile(token => token.Type != TokenType.DoubleDash))
            if (pathTokens?.Any(path => ReferenceEquals(path, token)) == true && token.Value.StartsWith('-') && !(standardInput && token.Value == "-"))
                throw new ArgumentException($"Unknown option '{token.Value}'. Use '--' before paths starting with '-'.");
        foreach (var option in result.CommandResult.Command.Options.Where(option => option.ValueType != typeof(bool)))
            if (result.GetResult(option) is { } parsed && parsed.Tokens.Count < parsed.IdentifierTokenCount)
                throw new ArgumentException($"Each occurrence of '{option.Name}' requires a value.");
    }
}
