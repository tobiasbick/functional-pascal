using System.Globalization;
using System.Runtime.InteropServices;
using Fpas.Debugging.Core;
using Fpas.Debugging.Values;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private DebugBinding ReadBinding(string name, Func<NativeObject> storage, ValuePath? path = null, bool cell = false)
    {
        if (name.Length > 8192) throw new DebugException("limit_exceeded", "A binding name exceeds 8192 UTF-16 code units.");
        try
        {
            using var value = storage();
            if (!cell) return ReadValue(name, value, path: path) with { RuntimeType = ValueTypeIdentity(value) };
            var hop = CellHop(value); using var owner = modules.Single(module => module.Id == hop.Module).Native.Object(11, hop.Type);
            using var instance = ObjectValue(value); using var contents = instance.Field(owner, hop.Field);
            var sourcePath = path is null ? null : path with { Hops = [.. path.Hops, hop] };
            return ReadValue(name, contents, path: sourcePath) with { RuntimeType = ValueTypeIdentity(contents) };
        }
        catch (NativeOperationException exception)
        {
            var unavailable = exception.Result is unchecked((int)0x80131304) or unchecked((int)0x80131306) or unchecked((int)0x80131334);
            return new(name, "[unavailable]", "unknown", null, unavailable ? "optimized_out" : "unavailable",
                Error: new(unavailable ? "optimized_out" : "unavailable", exception.Message));
        }
        catch (DebugException exception) { return new(name, "[unavailable]", "unknown", null, "unavailable", Error: new(exception.Code, exception.Message)); }
        catch (IOException) { return new(name, "[unavailable]", "unknown", null, "unavailable", Error: new("unavailable", "Stored value or metadata is unavailable.")); }
        catch (BadImageFormatException) { return new(name, "[unavailable]", "unknown", null, "unavailable", Error: new("invalid_metadata", "Stored value metadata is invalid.")); }
    }

    private unsafe DebugBinding ReadValue(string name, NativeObject value, int depth = 0, ValuePath? path = null)
    {
        if (depth >= 16) throw new DebugException("limit_exceeded", "Value indirection exceeds 16 levels.");
        var kind = value.UInt32(3);
        if (kind is 15 or 22 or 27)
            return new(name, ValueTypeName(value), "pointer", null, "unavailable",
                Error: new("unsupported_value", "Unmanaged pointers, typed references and function pointers are not dereferenced."));
        if (kind == 16)
        {
            using var reference = value.Query(referenceValueId); using var contents = reference.Object(10);
            return ReadValue(name, contents, depth + 1, path);
        }
        if (ScalarValue.Size(kind) is > 0 and var size)
        {
            if (value.UInt32(4) != size) throw new DebugException("unavailable", "The runtime scalar size does not match its type.");
            using var generic = value.Query(genericValueId);
            Span<byte> bytes = stackalloc byte[size];
            fixed (byte* pointer = bytes)
                NativeObject.Check(((delegate* unmanaged[Stdcall]<nint, byte*, int>)generic.Slot(7))(generic.Pointer, pointer));
            return ScalarValue.Read(name, kind, bytes);
        }
        if (NullReference(value)) return new(name, ValueTypeName(value), "null", null);
        if (kind == 14)
        {
            var text = StringValue(value);
            return new(name, "string", "string", text.Text, text.Status, text.Truncated, text.RawUtf16);
        }
        var typeName = ValueTypeName(value);
        if (typeName == "System.Decimal") return DecimalValue(name, value);
        if (kind == 17)
        {
            using var type = ExactType(value); using var parent = type.OptionalObject(7); var remaining = 128;
            if (parent is not null && TypeName(parent, 0, ref remaining) == "System.Enum")
            {
                using var underlying = StoredField(value, "value__");
                if (underlying is not null) return ReadValue(name, underlying, depth + 1) with { Type = typeName, Kind = "enum" };
            }
        }
        return CompoundValue(name, value, typeName, path);
    }

    private string ValueTypeName(NativeObject value)
    {
        using var type = ExactType(value);
        var remaining = 128;
        return TypeName(type, 0, ref remaining);
    }

    private string TypeName(NativeObject type, int depth, ref int remaining)
    {
        if (depth >= 16 || --remaining < 0) throw new DebugException("limit_exceeded", "Runtime type description exceeds 16 levels or 128 nodes.");
        var kind = type.UInt32(3);
        if (ScalarValue.TypeName(kind) is { } primitive) return primitive;
        if (kind is 15 or 16 or 20 or 29)
        {
            using var element = type.Object(6);
            var rank = kind == 20 ? type.UInt32(9) : 1;
            if (rank is < 1 or > 32) throw new DebugException("unavailable", "The array rank is outside 1..32.");
            var suffix = kind switch { 15 => "*", 16 => "&", 29 => "[]", _ => "[" + new string(',', (int)rank - 1) + "]" };
            return TypeName(element, depth + 1, ref remaining) + suffix;
        }
        if (kind is not (17 or 18)) return $"[CLR type 0x{kind:X2}]";
        using var runtimeClass = type.Object(4); using var module = runtimeClass.Object(3);
        var result = MetadataFor(module).TypeName(runtimeClass.UInt32(4));
        using var arguments = type.Object(5); var parameters = new List<string>();
        while (true)
        {
            using var argument = arguments.Next();
            if (argument is null) break;
            if (parameters.Count >= 64) throw new DebugException("limit_exceeded", "A runtime type has more than 64 generic arguments.");
            parameters.Add(TypeName(argument, depth + 1, ref remaining));
        }
        result = parameters.Count == 0 ? result : result + "<" + string.Join(",", parameters) + ">";
        if (result.Length > 8192) throw new DebugException("limit_exceeded", "A runtime type description exceeds 8192 UTF-16 code units.");
        return result;
    }

    private DebugBinding DecimalValue(string name, NativeObject value)
    {
        using var flags = StoredField(value, "_flags"); using var high = StoredField(value, "_hi32"); using var low = StoredField(value, "_lo64");
        if (flags is null || high is null || low is null || ReadInt32(flags) is not { } bits)
            throw new DebugException("unavailable", "The decimal storage shape is unavailable.");
        var hi = uint.Parse(ReadValue("", high).Scalar!, CultureInfo.InvariantCulture);
        var lo = ulong.Parse(ReadValue("", low).Scalar!, CultureInfo.InvariantCulture);
        var scale = (bits >> 16) & 0xff;
        if ((bits & 0x7f00ffff) != 0 || scale > 28) throw new DebugException("unavailable", "The stored decimal flags are invalid.");
        var scalar = new decimal(unchecked((int)lo), unchecked((int)(lo >> 32)), unchecked((int)hi), bits < 0, (byte)scale);
        return new(name, "decimal", "decimal", scalar.ToString(CultureInfo.InvariantCulture));
    }
}
