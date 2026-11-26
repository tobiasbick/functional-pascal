// [Documentation](../../../../docs/pascal/tools/api-documentation.md)
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Types;

namespace Fpas.Compiler.Analysis;

internal sealed class UsageUnit
{
    private readonly List<string> inputs = [];
    private readonly List<string> parameters = [];
    internal UsageNames Names { get; } = new();
    internal bool HasByRefInputs { get; private set; }

    internal string Receiver(ImportedType definition, TypeSymbol owner, bool isStatic, bool constructor = false)
        => isStatic ? Names.Named(definition, owner.TypeArguments, groupBeforeSuffix: !constructor) : Input("receiver", owner, definition.IsValueType ? "ref" : "");

    internal string Input(string parameterName, TypeSymbol type, string mode = "")
    {
        var spelling = Names.Type(type);
        var name = Names.Parameter(parameterName, inputs.Count);
        inputs.Add(name);
        HasByRefInputs |= mode.Length != 0;
        parameters.Add((mode.Length == 0 ? "" : mode + " ") + name + ": " + spelling);
        return name;
    }

    internal ApiUsage Render(string statement, string? result = null, bool isAsync = false)
    {
        var routine = result is null ? "procedure" : "function";
        var signature = result is null ? "" : ": " + result;
        var source = $"""
            unit ApiUsage;

            {Names.Imports}public {(isAsync ? "async " : "")}{routine} Use({string.Join("; ", parameters)}){signature};
            begin
              {statement}
            end {routine};

            end unit;
            """ + "\n";
        return inputs.Count == 0
            ? new("example", source, [], "Compiler-checked unit with no caller inputs.")
            : new("template", source, inputs.AsReadOnly(), "Compiler-checked unit. Supply the typed inputs when calling Use.");
    }
}
