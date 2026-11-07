using System.Globalization;
using System.Runtime.InteropServices;
using Fpas.Debugging.Core;
using Fpas.Debugging.Symbols;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private sealed record RetainedFrame(DebugFrame Data, NativeObject Frame, ModuleSymbols? Symbols);
    private readonly Dictionary<long, RetainedFrame> frames = [];
    private readonly Dictionary<uint, StackPage> stacks = [];
    private long frameSequence;

    internal DebugThread[] Threads()
    {
        lock (gate)
        {
            if (stoppedThread is null) throw new InvalidOperationException("The target is not stopped.");
            var selected = stoppedThread.UInt32(4);
            using var enumeration = process!.Object(7);
            var result = new List<DebugThread>();
            while (true)
            {
                using var thread = enumeration.Next();
                if (thread is null) break;
                if (result.Count == 1024) throw new DebugException("limit_exceeded", "The CLR thread catalog exceeds 1024 entries.");
                var id = thread.UInt32(4);
                result.Add(new(id, id == selected, $"Thread {id}"));
            }
            return result.OrderByDescending(thread => thread.IsStopThread).ThenBy(thread => thread.Id).ToArray();
        }
    }

    internal StackPage Stack(uint threadId, int start, int count)
    {
        if (start < 0 || count is < 1 or > 128) throw new DebugException("invalid_argument", "Stack pages require a nonnegative start and count 1..128.");
        lock (gate)
        {
            if (!stacks.TryGetValue(threadId, out var stack))
            {
                stack = ReadStack(threadId); stacks.Add(threadId, stack);
            }
            return stack with { Items = stack.Items.Skip(start).Take(count).ToArray() };
        }
    }

    private NativeObject Thread(uint id)
    {
        if (stoppedThread is null) throw new InvalidOperationException("The target is not stopped.");
        if (stoppedThread.UInt32(4) == id) return NativeObject.Borrow(stoppedThread.Pointer);
        using var threads = process!.Object(7);
        for (var index = 0; index <= 1024; index++)
        {
            using var thread = threads.Next();
            if (thread is null) throw new DebugException("invalid_argument", "The CLR thread is not present at this stop.");
            if (thread.UInt32(4) == id) return NativeObject.Borrow(thread.Pointer);
        }
        throw new DebugException("limit_exceeded", "The CLR thread catalog exceeds 1024 entries.");
    }

    private StackPage ReadStack(uint threadId)
    {
        var first = frameSequence;
        try { return ReadStackCore(threadId); }
        catch
        {
            foreach (var id in frames.Keys.Where(id => id > first).ToArray())
            {
                frames[id].Frame.Dispose(); frames.Remove(id);
            }
            throw;
        }
    }

    private StackPage ReadStackCore(uint threadId)
    {
        using var thread = Thread(threadId);
        using var chains = thread.Object(13);
        var result = new List<DebugFrame>();
        var nativeChains = 0;
        for (var chainIndex = 0; chainIndex < 4096; chainIndex++)
        {
            using var chain = chains.Next();
            if (chain is null) return new(result.ToArray(), result.Count, false, nativeChains);
            if (chain.UInt32(10) == 0) { nativeChains++; continue; }
            using var enumeration = chain.Object(11);
            while (true)
            {
                using var frame = enumeration.Next();
                if (frame is null) break;
                if (frames.Count >= 4096) return new(result.ToArray(), null, true, nativeChains);
                var retained = DescribeFrame(frame, threadId);
                frames.Add(retained.Data.Id, retained); result.Add(retained.Data);
            }
        }
        return new(result.ToArray(), null, true, nativeChains);
    }

    private unsafe RetainedFrame DescribeFrame(NativeObject frame, uint thread)
    {
        var id = ++frameSequence;
        using var il = frame.TryQuery(ilFrameId);
        if (il is null) return new(new(id, thread, "[runtime frame]", "", 0, null, null, "unavailable", false, true), NativeObject.Borrow(frame.Pointer), null);
        using var function = frame.Object(5);
        using var module = function.Object(3);
        var path = module.Name(6);
        var token = frame.UInt32(6);
        var probe = catalog.Probe(path, module.ModuleIdentity());
        var metadata = probe.Symbols;
        uint offset = 0, mapping = 0;
        NativeObject.Check(((delegate* unmanaged[Stdcall]<nint, uint*, uint*, int>)il.Slot(11))(il.Pointer, &offset, &mapping));
        // CoreCLR may report a prolog mapping at a bound IL breakpoint.
        var atBreakpoint = breakpointFrame?.SameIdentity(il.Pointer) == true;
        var mapped = atBreakpoint || (mapping & (0x10 | 0x20)) != 0;
        var effective = FramePosition(il);
        var source = mapped ? metadata?.Position(token, checked((int)effective.Offset)) : null;
        var async = AsyncFrameFor(il, metadata, token);
        var runtimeName = metadata?.MethodName(token) ?? $"[method 0x{token:X8}]";
        var data = new DebugFrame(id, thread, async?.Method ?? runtimeName, path, token,
            mapped ? checked((int)(atBreakpoint ? effective.Offset : offset)) : null, source, metadata is null ? probe.Error?.Code ?? "missing_symbols" : source?.Status ?? "no_source",
            effective.Offset != offset || (mapping & 0x20) != 0 || source is not null && !metadata!.IsSourcePoint(token, (int)offset), async is null && (metadata?.Generated(token) ?? false), async, runtimeName);
        return new(data, NativeObject.Borrow(frame.Pointer), metadata);
    }

    private static string? Int32(NativeObject value) => ReadInt32(value)?.ToString(CultureInfo.InvariantCulture);

    private static unsafe int? ReadInt32(NativeObject value)
    {
        if (value.UInt32(3) != 8) return null;
        using var generic = value.Query(genericValueId);
        int scalar = 0;
        NativeObject.Check(((delegate* unmanaged[Stdcall]<nint, int*, int>)generic.Slot(7))(generic.Pointer, &scalar));
        return scalar;
    }

    private void ReleaseFrames()
    {
        ReleaseValues();
        foreach (var frame in frames.Values) frame.Frame.Dispose();
        frames.Clear(); stacks.Clear();
    }
}
