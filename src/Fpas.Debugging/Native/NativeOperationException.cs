namespace Fpas.Debugging.Native;

internal sealed class NativeOperationException(int result) : IOException($"CoreCLR debugging operation failed (HRESULT 0x{result:X8}).")
{
    internal int Result { get; } = result;
}
