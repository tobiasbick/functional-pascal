using System.CommandLine;
using System.Text.Json;
using System.Text.Json.Schema;
using System.Text.Json.Serialization.Metadata;
using System.Text.Json.Serialization;
using System.Text.Json.Nodes;

namespace Fpas.Cli.Reporting;

internal sealed class SchemaCommand
{
    private readonly Argument<string> name = new("name") { Description = "operation, terminal-script, project, test, debug, format, doc, refactor, environment, compile, check or query." };
    internal Command Command { get; } = new("schema", "Write a JSON Schema 2020-12 document for a CLI report or terminal input. No files are written.\n\nExamples:\n  fpas schema operation\n\n  fpas schema terminal-script\n\n  fpas schema test\n");
    internal SchemaCommand()
    {
        Command.Arguments.Add(name);
        Command.SetAction(result =>
        {
            var selected = result.GetValue(name)!;
            JsonTypeInfo? info = selected switch
            {
                "operation" => OperationJson.Default.OperationReport,
                "terminal-script" => OperationJson.Default.TerminalScript,
                "project" => Projects.ProjectJson.Default.ProjectReport,
                "test" => Testing.TestJson.Default.TestReport,
                "format" => Formatting.FormatJson.Default.FormatReport,
                "doc" => Documentation.DocJson.Default.DocReport,
                "refactor" => Refactoring.RefactorJson.Default.RefactorReport,
                "environment" => Toolchain.EnvironmentJson.Default.EnvironmentReport,
                "compile" => Diagnostics.CompileJson.Default.CompileReport,
                "check" => Analysis.CheckJson.Default.CheckResult,
                "query" => Analysis.QueryJson.Default.QueryResult,
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
            if (selected is "operation" or "debug")
            {
                schema["properties"]!["schemaVersion"]!["const"] = 1;
                if (schema["required"] is not JsonArray required) { required = new(); schema["required"] = required; }
                if (!required.Any(item => item?.GetValue<string>() == "schemaVersion")) required.Add("schemaVersion");
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
