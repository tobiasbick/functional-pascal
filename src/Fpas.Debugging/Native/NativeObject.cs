using System.Runtime.InteropServices;

namespace Fpas.Debugging.Native;

// CoreCLR uses COM-style vtables on both Windows and Unix. No COM activation is involved.
internal sealed unsafe class NativeObject : IDisposable
{
    private nint pointer;
    internal nint Pointer => pointer != 0 ? pointer : throw new ObjectDisposedException(nameof(NativeObject));
    internal NativeObject(nint pointer) => this.pointer = pointer != 0 ? pointer : throw new IOException("The runtime returned a null debugging interface.");
    internal nint Slot(int slot) => (*(nint**)Pointer)[slot];

    internal static NativeObject Borrow(nint pointer)
    {
        var value = new NativeObject(pointer);
        ((delegate* unmanaged[Stdcall]<nint, uint>)value.Slot(1))(pointer);
        return value;
    }

    internal NativeObject Query(Guid id)
    {
        nint result = 0;
        Check(((delegate* unmanaged[Stdcall]<nint, Guid*, nint*, int>)Slot(0))(Pointer, &id, &result));
        return new(result);
    }

    internal NativeObject? TryQuery(Guid id)
    {
        nint result = 0;
        var status = ((delegate* unmanaged[Stdcall]<nint, Guid*, nint*, int>)Slot(0))(Pointer, &id, &result);
        if (status == unchecked((int)0x80004002)) return null;
        Check(status); return new(result);
    }

    internal NativeObject? Next()
    {
        nint result = 0; uint fetched = 0;
        Check(((delegate* unmanaged[Stdcall]<nint, uint, nint*, uint*, int>)Slot(7))(Pointer, 1, &result, &fetched));
        return fetched == 0 ? null : new(result);
    }

    internal Guid ModuleIdentity()
    {
        var importId = new Guid("7DAC8207-D3AE-4C75-9B67-92801A497D44");
        nint pointer = 0;
        Check(((delegate* unmanaged[Stdcall]<nint, Guid*, nint*, int>)Slot(14))(Pointer, &importId, &pointer));
        using var metadata = new NativeObject(pointer);
        Guid identity = default; uint length = 0;
        Check(((delegate* unmanaged[Stdcall]<nint, char*, uint, uint*, Guid*, int>)metadata.Slot(10))
            (metadata.Pointer, null, 0, &length, &identity));
        return identity;
    }

    internal bool SameIdentity(nint other)
    {
        var id = new Guid("00000000-0000-0000-C000-000000000046");
        using var left = Query(id);
        using var borrowed = Borrow(other);
        using var right = borrowed.Query(id);
        return left.Pointer == right.Pointer;
    }

    internal int Call(int slot) => ((delegate* unmanaged[Stdcall]<nint, int>)Slot(slot))(Pointer);
    internal int Call(int slot, int value) => ((delegate* unmanaged[Stdcall]<nint, int, int>)Slot(slot))(Pointer, value);
    internal int Call(int slot, int first, int second) => ((delegate* unmanaged[Stdcall]<nint, int, int, int>)Slot(slot))(Pointer, first, second);
    internal int CallPointer(int slot, nint value) => ((delegate* unmanaged[Stdcall]<nint, nint, int>)Slot(slot))(Pointer, value);
    internal NativeObject Object(int slot)
    {
        nint result = 0;
        Check(((delegate* unmanaged[Stdcall]<nint, nint*, int>)Slot(slot))(Pointer, &result));
        return new(result);
    }
    internal NativeObject? OptionalObject(int slot)
    {
        nint result = 0;
        Check(((delegate* unmanaged[Stdcall]<nint, nint*, int>)Slot(slot))(Pointer, &result));
        return result == 0 ? null : new(result);
    }
    internal NativeObject Field(NativeObject owner, uint token)
    {
        nint result = 0;
        Check(((delegate* unmanaged[Stdcall]<nint, nint, uint, nint*, int>)Slot(8))(Pointer, owner.Pointer, token, &result));
        return new(result);
    }
    internal ulong UInt64(int slot)
    {
        ulong result = 0;
        Check(((delegate* unmanaged[Stdcall]<nint, ulong*, int>)Slot(slot))(Pointer, &result));
        return result;
    }
    internal NativeObject Object(int slot, uint argument)
    {
        nint result = 0;
        Check(((delegate* unmanaged[Stdcall]<nint, uint, nint*, int>)Slot(slot))(Pointer, argument, &result));
        return new(result);
    }
    internal uint UInt32(int slot)
    {
        uint result = 0;
        Check(((delegate* unmanaged[Stdcall]<nint, uint*, int>)Slot(slot))(Pointer, &result));
        return result;
    }
    internal string Name(int slot)
    {
        uint length = 0;
        Check(((delegate* unmanaged[Stdcall]<nint, uint, uint*, char*, int>)Slot(slot))(Pointer, 0, &length, null));
        if (length is 0 or > 32768) throw new IOException("The runtime returned an invalid name length.");
        var text = new char[length];
        fixed (char* buffer = text)
            Check(((delegate* unmanaged[Stdcall]<nint, uint, uint*, char*, int>)Slot(slot))(Pointer, length, &length, buffer));
        return new string(text, 0, (int)length - 1);
    }

    internal static void Check(int result)
    {
        if (result < 0) throw new NativeOperationException(result);
    }
    public void Dispose()
    {
        var previous = Interlocked.Exchange(ref pointer, 0);
        if (previous != 0) ((delegate* unmanaged[Stdcall]<nint, uint>)(*(nint**)previous)[2])(previous);
    }
}
