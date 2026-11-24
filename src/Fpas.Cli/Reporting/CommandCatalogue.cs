// [Documentation](../../../docs/pascal/tools/report-schemas.md)
using System.CommandLine;
using System.Text.Json.Nodes;

namespace Fpas.Cli.Reporting;

internal static class CommandCatalogue
{
    internal static JsonObject? Create(Command root, IReadOnlyList<string> selection)
    {
        var commands = new JsonArray();
        var command = root;
        var path = root.Name;
        var inherited = Array.Empty<Option>();
        foreach (var word in selection)
        {
            var child = VisibleChildren(command).FirstOrDefault(child => child.Name == word);
            if (child is null) return null;
            inherited = Options(command, inherited).Where(option => option.Recursive).ToArray();
            command = child;
            path += " " + child.Name;
        }
        Visit(command, path, inherited, recursive: selection.Count == 0);
        return new() { ["schemaVersion"] = 1, ["commands"] = commands };

        void Visit(Command current, string currentPath, IReadOnlyList<Option> inheritedOptions, bool recursive)
        {
            var options = Options(current, inheritedOptions);
            var children = VisibleChildren(current).ToArray();
            var entry = new JsonObject
            {
                ["command"] = currentPath,
                ["description"] = Summary(current),
                ["arguments"] = new JsonArray(current.Arguments.Where(argument => !argument.Hidden).Select(DescribeArgument).ToArray()),
                ["options"] = new JsonArray(options.Select(DescribeOption).ToArray())
            };
            if (children.Length > 0) entry["subcommands"] = Strings(children.Select(child => child.Name));
            var rules = CommandConstraints.Get(current);
            if (rules.Count > 0)
                entry["rules"] = new JsonArray(rules.Select(rule => new JsonObject { ["kind"] = rule.Kind, ["options"] = Strings(rule.Options) }).ToArray());
            if (ReportName(currentPath) is { } report)
            {
                entry["reportSchema"] = "urn:fpas:cli:" + report + ":1";
                if (options.Any(option => option.Name == "--report"))
                    entry["reportOption"] = "--report json";
            }
            if (currentPath is "fpas check" or "fpas refactor draft")
            {
                entry["inputSchema"] = "urn:fpas:cli:check-overlay-input:1";
                entry["inputOption"] = "--overlay";
            }
            if (currentPath == "fpas query batch")
            {
                entry["inputSchema"] = "urn:fpas:cli:query-batch-input:1";
                entry["optionInputSchemas"] = new JsonObject { ["--overlay"] = "urn:fpas:cli:check-overlay-input:1" };
            }
            if (currentPath == "fpas terminal test") entry["inputSchema"] = "urn:fpas:cli:terminal-script:1";
            commands.Add(entry);
            if (!recursive) return;
            var recursiveOptions = options.Where(option => option.Recursive).ToArray();
            foreach (var child in children)
                Visit(child, currentPath + " " + child.Name, recursiveOptions, recursive);
        }
    }

    private static IEnumerable<Command> VisibleChildren(Command command)
        => command.Subcommands.Where(child => !child.Hidden).OrderBy(child => child.Name, StringComparer.Ordinal);

    private static Option[] Options(Command command, IReadOnlyList<Option> inherited)
        => command.Options.Concat(inherited).Where(option => !option.Hidden).DistinctBy(option => option.Name).ToArray();

    internal static JsonArray Strings(IEnumerable<string> values) => new(values.Select(value => (JsonNode?)value).ToArray());

    private static JsonObject DescribeArgument(Argument argument)
    {
        var entry = Describe(argument, argument.ValueType, argument.Arity, argument.Arity.MinimumNumberOfValues > 0);
        if (argument.HasDefaultValue) entry["default"] = DefaultValue(argument.GetDefaultValue());
        return entry;
    }

    private static JsonObject DescribeOption(Option option)
    {
        var entry = Describe(option, option.ValueType, option.Arity, option.Required);
        if (option.Aliases.Count > 0) entry["aliases"] = new JsonArray(option.Aliases.Order(StringComparer.Ordinal).Select(alias => (JsonNode?)alias).ToArray());
        if (option.HasDefaultValue) entry["default"] = DefaultValue(option.GetDefaultValue());
        else if (option.ValueType == typeof(bool)) entry["default"] = false;
        if (CommandConstraints.Get(option) is { } constraints)
        {
            if (constraints.Allowed is { } allowed) entry["allowedValues"] = Strings(allowed);
            if (constraints.Minimum is { } minimum) entry["minimum"] = minimum;
            if (constraints.Maximum is { } maximum) entry["maximum"] = maximum;
        }
        if (option.AllowMultipleArgumentsPerToken) entry["multipleArgumentsPerToken"] = true;
        return entry;
    }

    private static JsonObject Describe(Symbol symbol, Type type, ArgumentArity arity, bool required) => new()
    {
        ["name"] = symbol.Name,
        ["description"] = Summary(symbol),
        ["type"] = type == typeof(bool) ? "boolean" : type == typeof(int) ? "integer" : type == typeof(string[]) ? "string[]" : "string",
        ["required"] = required,
        ["minValues"] = arity.MinimumNumberOfValues,
        ["maxValues"] = arity.MaximumNumberOfValues == int.MaxValue ? null : JsonValue.Create(arity.MaximumNumberOfValues)
    };

    private static JsonNode? DefaultValue(object? value) => value switch
    {
        bool boolean => JsonValue.Create(boolean),
        int integer => JsonValue.Create(integer),
        string text => JsonValue.Create(text),
        string[] strings => new JsonArray(strings.Select(text => (JsonNode?)text).ToArray()),
        _ => null
    };

    private static string? Summary(Symbol symbol) => symbol.Description?.Split('\n', 2)[0].Trim();

    private static string? ReportName(string path)
    {
        if (path == "fpas query outline") return "query-outline";
        if (path == "fpas query batch") return "query-batch";
        var command = path.Split(' ', 3).ElementAtOrDefault(1);
        return command switch
        {
            "init" or "build" or "run" or "target" or "publish" or "clean" or "restore" => "project",
            "profile" or "terminal" or "diagnose" => "operation",
            "workspace" => "workspace",
            "env" => "environment",
            "fmt" => "format",
            "compile" or "check" or "query" or "test" or "debug" or "doc" or "refactor" => command,
            _ => null
        };
    }
}
