using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using Fpas.Compiler.Binding;

namespace Fpas.Compiler.Emission;

internal sealed partial class AssemblyEmitter
{
    private void EmitGlobalFields(BoundUnit unit, AssemblyIdentity runtime)
    {
        foreach (var global in unit.Globals)
        {
            var signature = new BlobBuilder();
            EncodeType(new BlobEncoder(signature).FieldSignature(), global.Type);
            var attributes = FieldAttributes.Static | (global.Syntax.IsPublic ? FieldAttributes.Public : FieldAttributes.Private);
            if (!global.Syntax.Binding.Mutable) attributes |= FieldAttributes.InitOnly;
            var handle = metadata.AddFieldDefinition(attributes, metadata.GetOrAddString(global.Name), metadata.GetOrAddBlob(signature));
            if (NullableFlags(global.Type).Contains((byte)2)) AddNullableAnnotation(handle, runtime, global.Type);
        }
    }
}
