using Fpas.Debugging.Core;
using Fpas.Debugging.Values;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private DebugTypeIdentity ValueTypeIdentity(NativeObject value)
    {
        using var type = ExactType(value); var remaining = 128;
        return DescribeType(type, 0, ref remaining) with { Exact = !NullReference(value) };
    }

    private DebugTypeIdentity DescribeType(NativeObject type, int depth, ref int remaining)
    {
        if (depth >= 16 || --remaining < 0) throw new DebugException("limit_exceeded", "Runtime type description exceeds 16 levels or 128 nodes.");
        var kind = type.UInt32(3);
        if (ScalarValue.TypeName(kind) is { } primitive) return new(primitive, kind);
        if (kind is 15 or 16 or 20 or 29)
        {
            using var element = type.Object(6); var parameter = DescribeType(element, depth + 1, ref remaining);
            var rank = kind == 20 ? type.UInt32(9) : 1;
            if (rank is < 1 or > 32) throw new DebugException("unavailable", "The array rank is outside 1..32.");
            var suffix = kind switch { 15 => "*", 16 => "&", 29 => "[]", _ => "[" + new string(',', (int)rank - 1) + "]" };
            return new(parameter.Name + suffix, kind, Arguments: [parameter], Rank: kind is 20 or 29 ? (int)rank : null);
        }
        if (kind is not (17 or 18)) return new($"[CLR type 0x{kind:X2}]", kind, Exact: false);
        using var owner = type.Object(4); using var module = owner.Object(3);
        var name = MetadataFor(module).TypeName(owner.UInt32(4)); var parameters = new List<DebugTypeIdentity>();
        using var arguments = type.Object(5);
        while (true)
        {
            using var argument = arguments.Next(); if (argument is null) break;
            if (parameters.Count == 64) throw new DebugException("limit_exceeded", "A runtime type has more than 64 generic arguments.");
            parameters.Add(DescribeType(argument, depth + 1, ref remaining));
        }
        if (parameters.Count != 0) name += "<" + string.Join(',', parameters.Select(parameter => parameter.Name)) + ">";
        if (name.Length > 8192) throw new DebugException("limit_exceeded", "A runtime type description exceeds 8192 UTF-16 code units.");
        var receipt = modules.SingleOrDefault(item => item.Native.SameIdentity(module.Pointer));
        return new(name, kind, receipt?.Id, module.ModuleIdentity(), owner.UInt32(4), parameters.ToArray());
    }
}
