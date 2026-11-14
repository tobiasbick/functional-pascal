using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Files;

/// <summary>A child returned by a custom folder listing.</summary>
/// <param name="Name">Name relative to the folder.</param>
/// <param name="Directory">Whether the child is a folder.</param>
/// <param name="Size">Size in bytes.</param>
/// <param name="Modified">UTC modification time as Unix seconds.</param>
public readonly record struct CuiFileEntry(string Name, bool Directory, ulong Size, long Modified);

internal sealed unsafe class CuiFileSource : IDisposable
{
    private readonly Func<string, bool, CuiFileEntry[]?>? list;
    private GCHandle handle;

    internal CuiFileSource(Func<string, bool, CuiFileEntry[]?>? list)
    {
        this.list = list;
        if (list is not null) handle = GCHandle.Alloc(this);
    }

    internal nint User => handle.IsAllocated ? GCHandle.ToIntPtr(handle) : 0;
    internal nint Callback => handle.IsAllocated
        ? (nint)(delegate* unmanaged[Cdecl]<nint, byte*, NativeEntry*, int, byte*, nuint, byte, int>)&List
        : 0;

    internal static void WritePath(string path, Span<byte> destination)
    {
        Utf8.Validate(path, nameof(path));
        if (Utf8.Encoding.GetByteCount(path) >= destination.Length)
            throw new ArgumentOutOfRangeException(nameof(path), "The path exceeds the native buffer.");
        int count = Utf8.Encoding.GetBytes(path, destination);
        destination[count] = 0;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct NativeEntry
    {
        internal byte* Name;
        internal byte Directory;
        internal ulong Size;
        internal long Modified;
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static int List(nint user, byte* path, NativeEntry* output, int capacity, byte* names, nuint nameCapacity, byte hidden)
    {
        try
        {
            var source = (CuiFileSource)GCHandle.FromIntPtr(user).Target!;
            var entries = source.list!(Marshal.PtrToStringUTF8((nint)path)!, hidden != 0);
            if (entries is null) return -1;
            int count = 0;
            var buffer = new Span<byte>(names, checked((int)nameCapacity));
            foreach (var entry in entries)
            {
                if (count == capacity) break;
                Utf8.Validate(entry.Name, nameof(entry.Name));
                if (entry.Name.Length == 0 || entry.Name is "." or ".." || entry.Name.Contains('/') || entry.Name.Contains('\\'))
                    return -1;
                int length = Utf8.Encoding.GetByteCount(entry.Name);
                if (length >= buffer.Length) break;
                int written = Utf8.Encoding.GetBytes(entry.Name, buffer);
                buffer[written] = 0;
                fixed (byte* name = buffer)
                    output[count++] = new() { Name = name, Directory = entry.Directory ? (byte)1 : (byte)0,
                        Size = entry.Size, Modified = entry.Modified };
                buffer = buffer[(written + 1)..];
            }
            return count;
        }
        catch (Exception)
        {
            // Native callbacks cannot propagate managed exceptions. CUI displays its unreadable-folder state.
            return -1;
        }
    }

    public void Dispose()
    {
        if (handle.IsAllocated) handle.Free();
    }
}
