using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using Fpas.Compiler.Binding;

namespace Fpas.Compiler.Emission;

internal sealed partial class AssemblyEmitter
{
    private void EmitAlias(AliasSymbol alias, MethodBodyStreamEncoder bodies, ref int parameterCount, AssemblyIdentity runtime)
    {
        var name = "<Alias:" + alias.Syntax.Name.Text + ">";
        var method = new RoutineSymbol(name, alias.Target!, [], false, false, -1);
        var parameter = AddParameters(method, ref parameterCount, runtime);
        var il = new InstructionEncoder(new BlobBuilder());
        il.OpCode(ILOpCode.Newobj); il.Token(Reference(new ImportedMethod(catalog.Symbol(catalog.Required("System.NotSupportedException")), ".ctor", TypeSymbol.Void, [], false))); il.OpCode(ILOpCode.Throw);
        var handle = metadata.AddMethodDefinition(MethodAttributes.Private | MethodAttributes.Static | MethodAttributes.HideBySig, MethodImplAttributes.IL,
            metadata.GetOrAddString(name), MethodSignature(alias.Target!, []), bodies.AddMethodBody(il, maxStack: 1), parameter);
        var attribute = catalog.Symbol(catalog.Required("Fpas.Core.AliasAttribute"));
        var constructor = new ImportedMethod(attribute, ".ctor", TypeSymbol.Void, [TypeSymbol.Int32, TypeSymbol.String], false);
        var blob = new BlobBuilder(); blob.WriteUInt16(1); blob.WriteInt32(1); blob.WriteSerializedString(alias.Syntax.Name.Text); blob.WriteUInt16(0);
        metadata.AddCustomAttribute(handle, Reference(constructor), metadata.GetOrAddBlob(blob));
    }
}
