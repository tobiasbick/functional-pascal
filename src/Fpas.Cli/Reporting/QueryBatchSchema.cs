// [Documentation](../../../docs/pascal/tools/report-schemas.md)
using Fpas.Cli.Analysis;
using System.CommandLine;
using System.Text.Json.Nodes;

namespace Fpas.Cli.Reporting;

internal static class QueryBatchSchema
{
    internal static JsonObject Create(Command query) => new()
    {
        ["$schema"] = "https://json-schema.org/draft/2020-12/schema",
        ["$id"] = "urn:fpas:cli:query-batch-input:1",
        ["type"] = "array",
        ["minItems"] = 1,
        ["maxItems"] = QueryBatchCommand.RequestLimit,
        ["items"] = new JsonObject
        {
            ["oneOf"] = new JsonArray(query.Subcommands.Where(command => command.Name is not ("batch" or "outline")).Select(Describe).ToArray())
        }
    };

    private static JsonObject Describe(Command operation)
    {
        var properties = new JsonObject { ["operation"] = new JsonObject { ["const"] = operation.Name } };
        var required = new List<string> { "operation" };
        var chained = QueryRequest.AcceptsSourceResult(operation.Name);
        foreach (var option in operation.Options.Where(option => option.Name is "--name" or "--match" or "--kind" or "--project"
            or "--file" or "--line" or "--column" or "--limit" or "--offset" or "--body" or "--symbol"))
        {
            var name = option.Name == "--kind" ? "kinds" : option.Name[2..];
            properties[name] = DescribeOption(option);
            if (option.Required && (!chained || name is not ("file" or "line" or "column"))) required.Add(name);
        }
        var description = new JsonObject
        {
            ["type"] = "object",
            ["properties"] = properties,
            ["required"] = CommandCatalogue.Strings(required),
            ["additionalProperties"] = false
        };
        if (chained)
        {
            properties["from"] = new JsonObject { ["type"] = "integer", ["minimum"] = 0, ["maximum"] = QueryBatchCommand.RequestLimit - 2,
                ["description"] = "Zero-based index of an earlier find request with a unique returned declaration." };
            description["oneOf"] = JsonNode.Parse("""
                [
                  {"required":["file","line","column"],"not":{"anyOf":[{"required":["from"]},{"required":["symbol"]}]}},
                  {"required":["from"],"not":{"anyOf":[{"required":["file"]},{"required":["line"]},{"required":["column"]},{"required":["project"]},{"required":["symbol"]}]}},
                  {"required":["symbol","project"],"not":{"anyOf":[{"required":["file"]},{"required":["line"]},{"required":["column"]},{"required":["from"]}]}}
                ]
                """);
        }
        return description;
    }

    private static JsonObject DescribeOption(Option option)
    {
        var type = option.ValueType;
        var value = new JsonObject { ["type"] = type == typeof(bool) ? "boolean" : type == typeof(int) ? "integer" : "string" };
        if (option.Name is "--name" or "--file") value["minLength"] = 1;
        if (option.Name is "--file" or "--symbol") value["pattern"] = "\\S";
        if (CommandConstraints.Get(option) is { } constraints)
        {
            if (constraints.Allowed is { } allowed) value["enum"] = CommandCatalogue.Strings(allowed);
            if (constraints.Minimum is { } minimum) value["minimum"] = minimum;
            if (constraints.Maximum is { } maximum) value["maximum"] = maximum;
        }
        return type == typeof(string[]) ? new JsonObject { ["type"] = "array", ["items"] = value } : value;
    }
}
