using System.Text.Json.Nodes;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Dap;

internal static class DapArguments
{
    internal static JsonObject Object(this JsonObject node, string name) => node[name] switch
    { null => new(), JsonObject value => value, _ => throw Invalid(name) };
    internal static string Text(this JsonObject node, string name, string? fallback = null) => node[name] switch
    {
        null when fallback is not null => fallback,
        JsonValue value when value.TryGetValue<string>(out var text) => text,
        _ => throw Invalid(name)
    };
    internal static int Number(this JsonObject node, string name, int? fallback = null) => node[name] switch
    {
        null when fallback is not null => fallback.Value,
        JsonValue value when value.TryGetValue<int>(out var number) => number,
        _ => throw Invalid(name)
    };
    internal static bool Flag(this JsonObject node, string name, bool fallback = false) => node[name] switch
    { null => fallback, JsonValue value when value.TryGetValue<bool>(out var flag) => flag, _ => throw Invalid(name) };
    internal static JsonArray Array(this JsonObject node, string name) => node[name] switch
    { null => new(), JsonArray value => value, _ => throw Invalid(name) };
    internal static DebugException Invalid(string name) => new("invalid_argument", "Invalid or missing DAP argument: " + name + ".");
}
