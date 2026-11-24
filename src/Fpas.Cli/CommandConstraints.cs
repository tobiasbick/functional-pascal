// [Documentation](../../docs/pascal/tools/report-schemas.md)
using System.CommandLine;
using System.Runtime.CompilerServices;

namespace Fpas.Cli;

internal static class CommandConstraints
{
    private static readonly ConditionalWeakTable<Option, Values> Options = new();
    private static readonly ConditionalWeakTable<Command, List<Rule>> Commands = new();

    internal sealed record Values(string[]? Allowed = null, int? Minimum = null, int? Maximum = null);
    internal sealed record Rule(string Kind, string[] Options);

    internal static Values? Get(Option option) => Options.TryGetValue(option, out var values) ? values : null;
    internal static IReadOnlyList<Rule> Get(Command command) => Commands.TryGetValue(command, out var rules) ? rules : [];

    internal static void AcceptValues<T>(this Option<T> option, params string[] values)
    {
        option.AcceptOnlyFromAmong(values);
        Options.Add(option, new(Allowed: values));
    }

    internal static void Range(this Option<int> option, int minimum, int maximum)
    {
        Options.Add(option, new(Minimum: minimum, Maximum: maximum));
        option.Validators.Add(result =>
        {
            if (result.Tokens.LastOrDefault()?.Value is { } text && int.TryParse(text, out var value) && (value < minimum || value > maximum))
                result.AddError($"{option.Name} must be between {minimum} and {maximum}.");
        });
    }

    internal static void Requires(this Command command, Option<bool> option, Option<bool> required)
        => command.Requires(option, required, result => result.GetValue(option), result => result.GetValue(required));

    internal static void Requires(this Command command, Option option, Option required,
        Func<System.CommandLine.Parsing.CommandResult, bool> selected, Func<System.CommandLine.Parsing.CommandResult, bool> satisfied)
    {
        Commands.GetOrCreateValue(command).Add(new("requires", [option.Name, required.Name]));
        command.Validators.Add(result =>
        {
            if (selected(result) && !satisfied(result))
                result.AddError($"{option.Name} requires {required.Name}.");
        });
    }

    internal static void ExactlyOne(this Command command, Option<string> left, Option<string> right)
    {
        Commands.GetOrCreateValue(command).Add(new("exactlyOne", [left.Name, right.Name]));
        command.Validators.Add(result =>
        {
            if ((result.GetValue(left) is null) == (result.GetValue(right) is null))
                result.AddError($"Specify exactly one of {left.Name} and {right.Name}.");
        });
    }

    internal static void Excludes(this Command command, Option<bool> option, Option other, Func<System.CommandLine.Parsing.CommandResult, bool> selected)
        => command.Excludes(option, other, result => result.GetValue(option), selected);

    internal static void Excludes(this Command command, Option option, Option other,
        Func<System.CommandLine.Parsing.CommandResult, bool> selected, Func<System.CommandLine.Parsing.CommandResult, bool> conflicting)
    {
        Commands.GetOrCreateValue(command).Add(new("excludes", [option.Name, other.Name]));
        command.Validators.Add(result =>
        {
            if (selected(result) && conflicting(result))
                result.AddError($"{option.Name} cannot combine with {other.Name}.");
        });
    }
}
