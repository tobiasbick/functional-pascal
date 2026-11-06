using Fpas.Debugging.Core;
using Fpas.Debugging.Symbols;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private static readonly Guid arrayValueId = new("0405B0DF-A660-11D2-BD02-0000F80849BD");
    private sealed record AsyncCaller(NativeObject Receiver, ModuleSymbols Symbols, AsyncMethodInfo Method, long Module);

    private AsyncCaller? AwaitingCaller(NativeObject receiver)
    {
        using var builder = StoredField(receiver, "<>t__builder");
        if (builder is null) return null;
        using var task = StoredField(builder, "m_task", name => name.StartsWith("System.Runtime.CompilerServices.Async", StringComparison.Ordinal));
        if (task is null || NullReference(task)) return null;
        using var continuation = StoredField(task, "m_continuationObject", name => name == "System.Threading.Tasks.Task");
        if (continuation is null || NullReference(continuation)) return null;
        var found = new List<AsyncCaller>(); var visited = new HashSet<ulong>(); var unknown = false;
        try
        {
            Visit(continuation, 0);
            if (found.Count > 1) throw new DebugException("ambiguous_caller", "More than one async caller is awaiting this task.");
            if (unknown) throw new DebugException("unavailable", "The task has a continuation whose awaiting source context is unavailable.");
            var result = found.SingleOrDefault(); found.Clear(); return result;
        }
        finally { foreach (var caller in found) caller.Receiver.Dispose(); }

        void Visit(NativeObject value, int depth)
        {
            if (NullReference(value)) return;
            using var reference = value.Query(referenceValueId);
            if (!visited.Add(reference.UInt64(8))) return;
            if (depth >= 16 || visited.Count > 128) throw new DebugException("limit_exceeded", "Async caller discovery exceeds its continuation bounds.");
            var name = RuntimeTypeName(value);
            if (name.StartsWith("System.Runtime.CompilerServices.AsyncTaskMethodBuilder`1+AsyncStateMachineBox`1", StringComparison.Ordinal)
                || name.StartsWith("System.Runtime.CompilerServices.AsyncTaskMethodBuilder`1+DebugFinalizableAsyncStateMachineBox`1", StringComparison.Ordinal))
            {
                using var machine = StoredField(value, "StateMachine");
                if (machine is null || NullReference(machine)) { unknown = true; return; }
                using var type = ExactType(machine); using var runtimeClass = type.Object(4); using var module = runtimeClass.Object(3);
                var symbols = catalog.Probe(module.Name(6), module.ModuleIdentity()).Symbols;
                var method = symbols?.AsyncType(runtimeClass.UInt32(4));
                var loaded = modules.SingleOrDefault(item => item.Native.SameIdentity(module.Pointer));
                if (method is null || loaded is null) { unknown = true; return; }
                using var machineReference = machine.Query(referenceValueId);
                if (found.Any(item => item.Receiver.UInt64(8) == machineReference.UInt64(8))) return;
                found.Add(new(NativeObject.Borrow(machineReference.Pointer), symbols!, method, loaded.Id)); return;
            }
            if (name == "System.Collections.Generic.List`1")
            {
                using var size = StoredField(value, "_size"); using var items = StoredField(value, "_items");
                if (size is null || items is null || ReadInt32(size) is not { } count || count < 0) { unknown = true; return; }
                if (count > 64) throw new DebugException("limit_exceeded", "Async caller discovery exceeds 64 continuation entries.");
                using var contents = items.Object(10); using var array = contents.Query(arrayValueId);
                if (array.UInt32(11) < count) throw new DebugException("unavailable", "The task continuation list has inconsistent storage.");
                for (uint index = 0; index < count; index++) { using var item = array.Object(16, index); Visit(item, depth + 1); }
                return;
            }
            var field = name is "System.Action" ? "_target" : name.StartsWith("System.Threading.Tasks.", StringComparison.Ordinal) && name.EndsWith("AwaitTaskContinuation", StringComparison.Ordinal) ? "m_action" : null;
            if (field is not null)
            {
                using var next = StoredField(value, field);
                if (next is not null) { Visit(next, depth + 1); return; }
            }
            unknown = true;
        }
    }
}
