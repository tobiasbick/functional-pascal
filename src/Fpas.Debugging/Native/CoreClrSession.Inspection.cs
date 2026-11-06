using System.Runtime.InteropServices;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private static readonly Guid ilFrameId = new("03E26311-4F76-11D3-88C6-006097945418");
    private static readonly Guid genericValueId = new("CC7BCAF8-8A68-11D2-983C-0000F808342D");

    private NativeObject Frame()
    {
        if (stoppedThread is null) throw new InvalidOperationException("The target is not stopped.");
        using var frame = stoppedThread.Object(15);
        return frame.Query(ilFrameId);
    }

    internal unsafe (uint Method, uint Offset) Position()
    {
        lock (gate)
        {
            using var frame = Frame();
            uint offset = 0, mapping = 0;
            NativeObject.Check(((delegate* unmanaged[Stdcall]<nint, uint*, uint*, int>)frame.Slot(11))(frame.Pointer, &offset, &mapping));
            return (frame.UInt32(6), offset);
        }
    }

    internal unsafe IReadOnlyDictionary<string, int> IntegerLocals()
    {
        lock (gate)
        {
            var (method, offset) = Position();
            using var frame = Frame();
            using var function = frame.Object(5);
            using var module = function.Object(3);
            var metadata = catalog.Open(module.Name(6), module.ModuleIdentity()) ?? throw new IOException("No symbols are available for this frame.");
            if (metadata.AsyncMethod(method)?.MoveNext == method)
            {
                var bindings = AsyncFrameBindings(frame, metadata, method, (int)offset, 0, 128);
                if (bindings.Truncated || bindings.Total > bindings.Items.Length) throw new DebugException("limit_exceeded", "The integer binding snapshot exceeds one inspection page.");
                return bindings.Items
                    .Where(value => value.Type == "int" && value.Status == "available" && value.Scalar is not null).ToDictionary(value => value.Name,
                    value => int.Parse(value.Scalar!, System.Globalization.CultureInfo.InvariantCulture));
            }
            var values = new Dictionary<string, int>();
            foreach (var local in metadata.Locals(method, (int)offset))
            {
                using var value = frame.Object(14, (uint)local.Slot);
                if (value.UInt32(3) != 8) continue; // ELEMENT_TYPE_I4
                using var generic = value.Query(genericValueId);
                int scalar = 0;
                NativeObject.Check(((delegate* unmanaged[Stdcall]<nint, int*, int>)generic.Slot(7))(generic.Pointer, &scalar));
                values.Add(local.Name, scalar);
            }
            return values;
        }
    }

    internal void StepOver() => Step("over");

    internal void Pause()
    {
        NativeObject controller;
        lock (gate)
        {
            if (stoppedThread is not null) return;
            if (process is null) throw new InvalidOperationException("The target has not initialized.");
            controller = NativeObject.Borrow(process.Pointer);
        }
        using (controller)
        {
            NativeObject.Check(controller.Call(3, -1));
            lock (gate)
            {
                if (stoppedThread is not null) { NativeObject.Check(controller.Call(4, 0)); return; }
                try
                {
                    using var threads = controller.Object(7);
                    using var thread = FirstThread(threads);
                    Stopped("pause", thread.Pointer);
                }
                catch { NativeObject.Check(controller.Call(4, 0)); throw; }
            }
        }
    }

    private static unsafe NativeObject FirstThread(NativeObject threads)
    {
        nint pointer = 0; uint fetched = 0;
        NativeObject.Check(((delegate* unmanaged[Stdcall]<nint, uint, nint*, uint*, int>)threads.Slot(7))(threads.Pointer, 1, &pointer, &fetched));
        if (fetched == 0) throw new IOException("The stopped target has no managed thread.");
        return new(pointer);
    }

}
