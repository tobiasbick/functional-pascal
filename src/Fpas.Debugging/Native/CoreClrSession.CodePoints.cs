using Fpas.Debugging.Core;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private NativeObject CreateCodePoint(NativeObject module, uint method, int offset)
    {
        using var function = module.Object(9, method); using var code = function.Object(6);
        for (var attempt = 0; attempt < 128; attempt++)
        {
            NativeObject? point = null;
            try { point = code.Object(7, checked((uint)offset)); NativeObject.Check(point.Call(3, 1)); return point; }
            catch (NativeOperationException exception) when (exception.Result == unchecked((int)0x80131345))
            {
                point?.Dispose();
                var symbols = catalog.Probe(module.Name(6), module.ModuleIdentity()).Symbols;
                if (symbols?.NextBindingOffset(method, offset) is not { } next)
                    throw new DebugException("unavailable", "No native breakpoint can represent the requested IL position.");
                offset = next;
            }
            catch { point?.Dispose(); throw; }
        }
        throw new DebugException("limit_exceeded", "Breakpoint binding exceeds 128 operand instructions.");
    }
}
