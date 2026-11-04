using Fpas.Debugging.Sessions;
using ModelContextProtocol.Server;
using System.ComponentModel.DataAnnotations;
using System.Reflection;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Text.Json.Schema;

namespace Fpas.Debugging.Tools;

internal static class DebugToolCatalog
{
    private sealed record FailureResult(FailureInfo Error);

    internal static IEnumerable<McpServerTool> Create(DebugTools target)
    {
        foreach (var method in typeof(DebugTools).GetMethods().Where(method => method.IsDefined(typeof(McpServerToolAttribute))))
        {
            var tool = McpServerTool.Create(method, target, new() { SerializerOptions = JsonSerializerOptions.Web });
            var input = JsonNode.Parse(tool.ProtocolTool.InputSchema.GetRawText())!;
            foreach (var parameter in method.GetParameters())
                if (input["properties"]?[parameter.Name!] is { } schema)
                {
                    DescribeConstraints(parameter, schema);
                    if (parameter.ParameterType == typeof(string[]) && schema["items"] is { } item) item["type"] = "string";
                }
            SeparateUnionTypes(input);
            tool.ProtocolTool.InputSchema = JsonSerializer.SerializeToElement(input);
            var success = JsonNode.Parse(tool.ProtocolTool.OutputSchema!.Value.GetRawText())!;
            var error = JsonSerializerOptions.Web.GetJsonSchemaAsNode(typeof(FailureResult), new() { TreatNullObliviousAsNonNullable = true });
            RebaseReferences(success, "#/anyOf/0");
            RebaseReferences(error, "#/anyOf/1");
            var result = new JsonObject { ["type"] = "object", ["anyOf"] = new JsonArray(success, error) };
            SeparateUnionTypes(result);
            tool.ProtocolTool.OutputSchema = JsonSerializer.SerializeToElement(result);
            yield return tool;
        }
    }

    private static void RebaseReferences(JsonNode node, string prefix)
    {
        if (node is JsonArray array)
            foreach (var child in array) { if (child is not null) RebaseReferences(child, prefix); }
        if (node is not JsonObject schema) return;
        if (schema["$ref"] is JsonValue value && value.TryGetValue<string>(out var reference))
        {
            if (reference == "#") schema["$ref"] = prefix;
            else if (reference.StartsWith("#/", StringComparison.Ordinal)) schema["$ref"] = prefix + reference[1..];
        }
        foreach (var property in schema.ToArray()) if (property.Value is not null) RebaseReferences(property.Value, prefix);
    }

    private static void SeparateUnionTypes(JsonNode node)
    {
        if (node is JsonArray array)
        {
            foreach (var child in array) if (child is not null) SeparateUnionTypes(child);
            return;
        }
        if (node is not JsonObject schema) return;
        foreach (var property in schema.ToArray()) if (property.Value is not null) SeparateUnionTypes(property.Value);
        if (schema["type"] is not JsonArray types) return;
        var alternatives = new JsonArray(types.Select(type => (JsonNode)new JsonObject { ["type"] = type!.DeepClone() }).ToArray());
        schema.Remove("type");
        if (!schema.ContainsKey("anyOf")) schema["anyOf"] = alternatives;
        else
        {
            if (schema["allOf"] is not JsonArray conjunction) schema["allOf"] = conjunction = [];
            conjunction.Add(new JsonObject { ["anyOf"] = alternatives });
        }
    }

    private static void DescribeConstraints(ParameterInfo parameter, JsonNode schema)
    {
        foreach (var attribute in parameter.GetCustomAttributes())
        {
            switch (attribute)
            {
                case RangeAttribute range:
                    schema["minimum"] = JsonSerializer.SerializeToNode(range.Minimum);
                    schema["maximum"] = JsonSerializer.SerializeToNode(range.Maximum);
                    break;
                case StringLengthAttribute length:
                    schema["minLength"] = length.MinimumLength; schema["maxLength"] = length.MaximumLength;
                    break;
                case MaxLengthAttribute length:
                    schema[parameter.ParameterType.IsArray ? "maxItems" : "maxLength"] = length.Length;
                    break;
                case AllowedValuesAttribute allowed:
                    schema["enum"] = JsonSerializer.SerializeToNode(allowed.Values);
                    break;
            }
        }
    }
}
