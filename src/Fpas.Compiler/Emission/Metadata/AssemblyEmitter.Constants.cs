using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.References;
using System.Reflection.Metadata;

namespace Fpas.Compiler.Emission.Metadata;

internal sealed partial class AssemblyEmitter
{
    private void AddDecimalConstant(EntityHandle owner, decimal number)
    {
        var type = catalog.Symbol(catalog.Required("System.Runtime.CompilerServices.DecimalConstantAttribute"));
        var constructor = new ImportedMethod(type, ".ctor", TypeSymbol.Void,
            [TypeSymbol.Byte, TypeSymbol.Byte, TypeSymbol.UInt32, TypeSymbol.UInt32, TypeSymbol.UInt32], false);
        var bits = decimal.GetBits(number); var value = new BlobBuilder();
        value.WriteUInt16(1); value.WriteByte((byte)((bits[3] >> 16) & 255)); value.WriteByte(bits[3] < 0 ? (byte)128 : (byte)0);
        value.WriteInt32(bits[2]); value.WriteInt32(bits[1]); value.WriteInt32(bits[0]); value.WriteUInt16(0);
        metadata.AddCustomAttribute(owner, Reference(constructor), metadata.GetOrAddBlob(value));
    }
}
