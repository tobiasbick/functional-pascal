// [Documentation](../../../docs/pascal/tools/report-schemas.md)
using System.CommandLine;
using System.Text.Json;
using System.Text.Json.Schema;
using System.Text.Json.Serialization.Metadata;
using System.Text.Json.Serialization;
using System.Text.Json.Nodes;

namespace Fpas.Cli.Reporting;

internal sealed class SchemaCommand
{
    private readonly Argument<string> name = new("name") { Description = "operation, terminal-script, workspace, project, test, debug, format, doc, refactor, environment, compile, check, check-overlay-input, query, query-outline, query-batch, query-batch-input or commands." };
    private readonly Argument<string[]> path = new("command-path") { Arity = ArgumentArity.ZeroOrMore, Description = "Optional command path for the commands catalogue." };
    internal Command Command { get; } = new("schema", "Write a JSON Schema for a CLI report or terminal input, or list registered commands. No files are written.\n\nExamples:\n  fpas schema operation\n\n  fpas schema terminal-script\n\n  fpas schema test\n\n  fpas schema commands\n\n  fpas schema commands query find\n\n  fpas schema query-batch-input\n");
    internal SchemaCommand(Command root)
    {
        Command.Arguments.Add(name);
        Command.Arguments.Add(path);
        Command.SetAction(result =>
        {
            var selected = result.GetValue(name)!;
            if (selected == "commands")
            {
                var catalogue = CommandCatalogue.Create(root, result.GetValue(path) ?? []);
                if (catalogue is null) return OperationReport.Fail("schema", "invalid_argument", "Unknown public command path.", 2);
                Console.WriteLine(catalogue.ToJsonString());
                return 0;
            }
            if (result.GetValue(path) is { Length: > 0 })
                return OperationReport.Fail("schema", "invalid_argument", "A command path is accepted only for the commands catalogue.", 2);
            if (selected == "check-overlay-input")
            {
                Console.WriteLine(new JsonObject
                {
                    ["$schema"] = "https://json-schema.org/draft/2020-12/schema",
                    ["$id"] = "urn:fpas:cli:check-overlay-input:1",
                    ["type"] = "array",
                    ["minItems"] = 1,
                    ["maxItems"] = Analysis.AnalysisOverlay.FileLimit,
                    ["items"] = JsonNode.Parse("""{"type":"object","required":["file","text"],"properties":{"file":{"type":"string","pattern":"\\S"},"text":{"type":"string"},"project":{"type":"string","pattern":"\\S","description":"Owning FPAS project for a file to create. Omit for a replacement."}},"additionalProperties":false}""")
                }.ToJsonString());
                return 0;
            }
            if (selected == "query-batch-input")
            {
                var inputSchema = QueryBatchSchema.Create(root.Subcommands.Single(command => command.Name == "query"));
                Console.WriteLine(inputSchema.ToJsonString(new JsonSerializerOptions { WriteIndented = true }));
                return 0;
            }
            JsonTypeInfo? info = selected switch
            {
                "operation" => OperationJson.Default.OperationReport,
                "terminal-script" => OperationJson.Default.TerminalScript,
                "workspace" => Projects.WorkspaceJson.Default.WorkspaceReport,
                "project" => Projects.ProjectJson.Default.ProjectReport,
                "test" => Testing.TestJson.Default.TestReport,
                "format" => Formatting.FormatJson.Default.FormatReport,
                "doc" => Documentation.DocJson.Default.DocReport,
                "refactor" => Refactoring.RefactorJson.Default.RefactorReport,
                "environment" => Toolchain.EnvironmentJson.Default.EnvironmentReport,
                "compile" => Diagnostics.CompileJson.Default.CompileReport,
                "check" => Analysis.CheckJson.Default.CheckResult,
                "query-outline" => Analysis.OutlineJson.Default.OutlineReport,
                "query" => Analysis.QueryJson.Default.QueryResult,
                "query-batch" => Analysis.QueryJson.Default.QueryBatchReport,
                _ => null
            };
            if (info is null && selected != "debug") return OperationReport.Fail("schema", "invalid_argument", "Unknown schema. See fpas schema --help.", 2);
            var schema = selected == "debug" ? Fpas.Debugging.DebuggerApplication.ReportSchema() : JsonSchemaExporter.GetJsonSchemaAsNode(info!, new()
            {
                TreatNullObliviousAsNonNullable = true,
                TransformSchemaNode = (context, node) =>
                {
                    // Report serializers omit nullable members; constructor requirements describe input, not those output bytes.
                    if (context.TypeInfo.Options.DefaultIgnoreCondition == JsonIgnoreCondition.WhenWritingNull && node is JsonObject value &&
                        value["required"] is JsonArray required && value["properties"] is JsonObject properties)
                        for (var index = required.Count - 1; index >= 0; index--)
                            if (AllowsNull(properties[required[index]!.GetValue<string>()])) required.RemoveAt(index);
                    return node;
                }
            });
            schema["$schema"] = "https://json-schema.org/draft/2020-12/schema";
            schema["$id"] = "urn:fpas:cli:" + selected + ":1";
            if (selected == "terminal-script") TerminalContract(schema);
            if (selected is "operation" or "debug" or "query-batch" or "query-outline" or "workspace")
            {
                schema["properties"]!["schemaVersion"]!["const"] = 1;
                if (schema["required"] is not JsonArray required) { required = new(); schema["required"] = required; }
                if (!required.Any(item => item?.GetValue<string>() == "schemaVersion")) required.Add((JsonNode?)"schemaVersion");
            }
            Console.WriteLine(schema.ToJsonString(new JsonSerializerOptions { WriteIndented = true })); return 0;
        });
    }

    private static bool AllowsNull(JsonNode? node) => node is JsonObject value &&
        (value["type"] is JsonArray types && types.Any(item => item?.GetValue<string>() == "null") || value["type"]?.ToJsonString() == "\"null\"" ||
        value["anyOf"] is JsonArray choices && choices.Any(AllowsNull));

    private static void TerminalContract(JsonNode schema)
    {
        var properties = schema["properties"]!;
        properties["schemaVersion"]!["const"] = 1;
        foreach (var (name, maximum) in new[] { ("columns", 500), ("rows", 300), ("timeoutSeconds", 3600), ("outputBytes", 1_048_576) })
        { properties[name]!["minimum"] = 1; properties[name]!["maximum"] = maximum; }
        properties["steps"]!["minItems"] = 1; properties["steps"]!["maxItems"] = 256;
        properties["arguments"]!["items"]!["type"] = "string";
        properties["steps"]!["items"]!["type"] = "object";
        properties["steps"]!["items"]!["properties"]!["action"]!["enum"] = new JsonArray("send", "expect", "resize", "exit");
        foreach (var (name, maximum) in new[] { ("columns", 500), ("rows", 300) })
        { properties["steps"]!["items"]!["properties"]![name]!["minimum"] = 1; properties["steps"]!["items"]!["properties"]![name]!["maximum"] = maximum; }
        properties["steps"]!["items"]!["oneOf"] = JsonNode.Parse("""
            [
              {"properties":{"action":{"enum":["send","expect"]},"text":{"type":"string","minLength":1},"columns":{"const":null},"rows":{"const":null},"exitCode":{"const":null}},"required":["text"]},
              {"properties":{"action":{"const":"resize"},"columns":{"type":"integer"},"rows":{"type":"integer"},"text":{"const":null},"exitCode":{"const":null}},"required":["columns","rows"]},
              {"properties":{"action":{"const":"exit"},"exitCode":{"type":"integer"},"text":{"const":null},"columns":{"const":null},"rows":{"const":null}},"required":["exitCode"]}
            ]
            """);
    }
}
