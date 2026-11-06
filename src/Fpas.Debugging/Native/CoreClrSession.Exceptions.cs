using Fpas.Debugging.Core;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private ExceptionPolicy exceptionPolicy = new();
    private string? stoppedException;

    internal ExceptionPolicy SetExceptions(ExceptionPolicy policy) { lock (gate) return exceptionPolicy = policy; }

    internal void ExceptionRaised(nint domain, nint thread, int kind)
    {
        lock (gate)
        {
            if (kind == 1 && exceptionPolicy.Raised || kind == 4 && exceptionPolicy.Unhandled)
            {
                stoppedException = kind == 1 ? "raised" : "unhandled";
                Stopped("exception", thread);
            }
            else ContinueCallback(domain);
        }
    }

    internal DebugExceptionInfo ExceptionInfo()
    {
        lock (gate)
        {
            if (stoppedThread is null || stoppedException is null) throw new DebugException("invalid_state", "The current stop is not an exception stop.");
            var stack = Stack(stoppedThread.UInt32(4), 0, 32);
            using var value = stoppedThread.Object(10);
            using var instance = ObjectValue(value);
            using var exact = ExactType(value);
            using var runtimeClass = exact.Object(4);
            using var module = runtimeClass.Object(3);
            var name = MetadataFor(module).TypeName(runtimeClass.UInt32(4));
            var message = StoredMessage(instance, exact);
            var thrown = ExceptionStack(instance);
            return new(name, message, stoppedException, thrown.Frames.FirstOrDefault(frame => frame.Source is not null)?.Source
                ?? stack.Items.FirstOrDefault(frame => frame.Source is not null)?.Source, thrown.Frames, stack.Items,
                thrown.Truncated || stack.Truncated || stack.Total is { } total && total > stack.Items.Length);
        }
    }

    private PassiveText StoredMessage(NativeObject instance, NativeObject exact)
    {
        var current = NativeObject.Borrow(exact.Pointer);
        try
        {
            for (var depth = 0; current is not null && depth < 64; depth++)
            {
                using var runtimeClass = current.Object(4);
                using var module = runtimeClass.Object(3);
                var image = MetadataFor(module); var token = runtimeClass.UInt32(4);
                if (image.TypeName(token) == "System.Exception")
                {
                    var field = image.Fields(token).FirstOrDefault(field => field.Name == "_message");
                    if (field.Token == 0) return new(null, Status: "unavailable");
                    using var value = instance.Field(runtimeClass, field.Token);
                    return StringValue(value);
                }
                var next = current.OptionalObject(7); current.Dispose(); current = next;
            }
            return new(null, Status: "unavailable");
        }
        finally { current?.Dispose(); }
    }
}
