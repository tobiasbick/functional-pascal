// [Documentation](../../docs/pascal/program-structure/cli.md)
using System.CommandLine;
using System.CommandLine.Parsing;

namespace Fpas.Cli;

internal static class CommandArguments
{
    internal static string? Supplied(ParseResult result, Argument<string> argument)
        => result.GetResult(argument)?.Tokens.FirstOrDefault()?.Value;

    internal static string? Supplied(ParseResult result, Option<string> option)
        => result.GetResult(option)?.Tokens.FirstOrDefault()?.Value;

    internal static string ErrorMessage(ParseResult result)
    {
        var missing = result.CommandResult.Command.Arguments.Where(argument => !argument.HasDefaultValue &&
            (result.GetResult(argument)?.Tokens.Count ?? 0) < argument.Arity.MinimumNumberOfValues).Select(argument => argument.Name).ToArray();
        var details = string.Join(" ", result.Errors.Select(error => error.Message));
        return missing.Length == 0 ? details : $"Missing required arguments ({string.Join(", ", missing)}). {details}";
    }

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
