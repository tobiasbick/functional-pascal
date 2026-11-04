using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Types;
using System.Diagnostics;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;

namespace Fpas.Compiler.Emission.Metadata;

internal sealed partial class AssemblyEmitter
{
    private const string GeneratedAttribute = "System.Runtime.CompilerServices.CompilerGeneratedAttribute";
    private const string DisplayAttribute = "System.Diagnostics.DebuggerDisplayAttribute";
    private const string ProxyAttribute = "System.Diagnostics.DebuggerTypeProxyAttribute";

    private void AddDebuggableAttribute(bool optimize)
    {
        var modes = DebuggableAttribute.DebuggingModes.IgnoreSymbolStoreSequencePoints;
        if (debug is not null && !optimize)
            modes |= DebuggableAttribute.DebuggingModes.Default | DebuggableAttribute.DebuggingModes.DisableOptimizations;
        AddDebugAttribute(MetadataTokens.EntityHandle(TableIndex.Assembly, 1), "System.Diagnostics.DebuggableAttribute",
            catalog.Symbol(catalog.Required("System.Diagnostics.DebuggableAttribute+DebuggingModes")), blob => blob.WriteInt32((int)modes));
    }

    private void AddDebugAttribute(EntityHandle target, string name, TypeSymbol? argument = null, Action<BlobBuilder>? write = null)
    {
        var owner = catalog.Symbol(catalog.Required(name));
        var constructor = new ImportedMethod(owner, ".ctor", TypeSymbol.Void, argument is null ? [] : [argument], false);
        var blob = new BlobBuilder(); blob.WriteUInt16(1); write?.Invoke(blob); blob.WriteUInt16(0);
        metadata.AddCustomAttribute(target, Reference(constructor), metadata.GetOrAddBlob(blob));
    }

    private void AddGeneratedMethod(MethodDefinitionHandle method, bool hidden = false)
    {
        AddDebugAttribute(method, GeneratedAttribute);
        AddDebugAttribute(method, "System.Diagnostics.DebuggerNonUserCodeAttribute");
        AddDebugAttribute(method, hidden ? "System.Diagnostics.DebuggerHiddenAttribute" : "System.Diagnostics.DebuggerStepThroughAttribute");
    }

    private void AddGeneratedField(FieldDefinitionHandle field, bool hidden = true)
    {
        AddDebugAttribute(field, GeneratedAttribute);
        if (hidden) AddDebuggerBrowsable(field, DebuggerBrowsableState.Never);
    }

    private void AddDebuggerBrowsable(EntityHandle target, DebuggerBrowsableState state)
        => AddDebugAttribute(target, "System.Diagnostics.DebuggerBrowsableAttribute",
            catalog.Symbol(catalog.Required("System.Diagnostics.DebuggerBrowsableState")), blob => blob.WriteInt32((int)state));

    private void AddDebuggerDisplay(TypeDefinitionHandle type, string display, IReadOnlyList<BoundAttribute>? attributes = null)
    {
        if (attributes?.Any(attribute => AttributeIs(attribute, DisplayAttribute)) != true)
            AddDebugAttribute(type, DisplayAttribute, TypeSymbol.String, blob => blob.WriteSerializedString(display));
    }

    private void AddRecordDebugDisplay(TypeDefinitionHandle type, RecordSymbol record)
    {
        var inherited = record.UnionCase?.Owner.Attributes.Where(attribute => AttributeIs(attribute, DisplayAttribute)).ToArray() ?? [];
        if (inherited.Length != 0 && !record.Attributes.Any(attribute => AttributeIs(attribute, DisplayAttribute)))
            AddSourceAttributes(type, inherited);
        else
            AddDebuggerDisplay(type, (record.UnionCase?.Syntax.Name.Text ?? record.Name) + "(" + string.Join(", ", record.Fields.Select(field => field.Name + " = {" + field.Name + "}")) + ")", record.Attributes);
    }
}
