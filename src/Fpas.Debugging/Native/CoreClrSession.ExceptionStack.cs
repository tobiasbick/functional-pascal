using System.Runtime.InteropServices;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    [StructLayout(LayoutKind.Sequential)] private struct ExceptionStackEntry
    {
        internal nint Module;
        internal ulong Address;
        internal uint Method;
        internal int Foreign;
    }
    [StructLayout(LayoutKind.Sequential)] private struct NativeMap
    {
        internal uint Il, Start, End;
    }

    private unsafe (ExceptionFrame[] Frames, bool Truncated) ExceptionStack(NativeObject value)
    {
        using var exception = value.TryQuery(new("AE4CA65D-59DD-42A2-83A5-57E8A08D8719"));
        if (exception is null) return ([], false);
        using var enumerator = exception.Object(3);
        var result = new List<ExceptionFrame>();
        for (var index = 0; index <= 64; index++)
        {
            ExceptionStackEntry entry = default; uint fetched = 0;
            NativeObject.Check(((delegate* unmanaged[Stdcall]<nint, uint, ExceptionStackEntry*, uint*, int>)enumerator.Slot(7))
                (enumerator.Pointer, 1, &entry, &fetched));
            if (fetched == 0) return (result.ToArray(), false);
            using var module = entry.Module == 0 ? null : new NativeObject(entry.Module);
            if (index == 64) return (result.ToArray(), true);
            if (module is null) { result.Add(new("[unavailable frame]", "", entry.Method, null, true, "unavailable")); continue; }
            var path = module.Name(6); var symbols = catalog.Probe(path, module.ModuleIdentity()).Symbols;
            SourceLocation? source = null;
            try { if (symbols is not null && NativeOffset(module, entry.Method, entry.Address) is { } offset) source = symbols.Position(entry.Method, offset); }
            catch (IOException) { }
            var name = symbols?.MethodName(entry.Method) ?? $"[method 0x{entry.Method:X8}]";
            result.Add(new(name, path, entry.Method, source, true, source is null ? "no_source" : source.Status));
        }
        return (result.ToArray(), true);
    }

    private static unsafe int? NativeOffset(NativeObject module, uint token, ulong address)
    {
        using var function = module.Object(9, token);
        using var code = function.Object(7);
        var start = code.UInt64(5); var size = code.UInt32(6);
        if (address < start || address - start >= size) return null;
        uint length = 0;
        NativeObject.Check(((delegate* unmanaged[Stdcall]<nint, uint, uint*, NativeMap*, int>)code.Slot(10))(code.Pointer, 0, &length, null));
        if (length > 4096) return null;
        var map = new NativeMap[length];
        fixed (NativeMap* pointer = map)
            NativeObject.Check(((delegate* unmanaged[Stdcall]<nint, uint, uint*, NativeMap*, int>)code.Slot(10))(code.Pointer, length, &length, pointer));
        var native = (uint)(address - start);
        foreach (var item in map)
            if (item.Il < 0xfffffff0 && native >= item.Start && native < item.End) return checked((int)item.Il);
        return null;
    }
}
