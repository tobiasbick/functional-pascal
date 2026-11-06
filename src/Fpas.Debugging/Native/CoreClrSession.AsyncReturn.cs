using Fpas.Debugging.Core;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private sealed record AsyncReturnPoint(NativeObject Point, SourceLocation Source);
    private sealed record AsyncReturn(NativeObject Receiver, string Id, string Method, List<AsyncReturnPoint> Points);
    private AsyncReturn? asyncReturn;

    private bool StartAsyncReturn(NativeObject frame)
    {
        using var function = frame.Object(5); using var module = function.Object(3);
        var symbols = catalog.Probe(module.Name(6), module.ModuleIdentity()).Symbols;
        var (token, _) = FramePosition(frame);
        if (symbols?.AsyncMethod(token)?.MoveNext != token) return false;
        using var receiver = frame.Object(16, 0);
        var caller = AwaitingCaller(receiver);
        if (caller is null)
        {
            var stack = Stack(stoppedThread!.UInt32(4), 0, 128);
            if (stack.Items.Skip(1).Any(item => item.Source is not null && !item.Generated)) return false;
            throw new DebugException("unavailable", "No awaiting source caller or physical source caller is available at this stop.");
        }
        using var parent = caller.Receiver;
        var identity = AsyncIdentity(parent); var method = caller.Symbols.MethodName(caller.Method.Kickoff);
        var points = new List<AsyncReturnPoint>();
        var context = new AsyncReturn(HeapHandle(parent, 1), identity, method, points);
        try
        {
            var parentModule = modules.SingleOrDefault(item => item.Id == caller.Module)
                ?? throw new DebugException("unavailable", "The awaiting caller's module is no longer loaded.");
            foreach (var point in caller.Method.Awaits.DistinctBy(point => (point.ResumeMethod, point.Resume)))
            {
                if (points.Count >= 128) throw new DebugException("limit_exceeded", "Async step-out exceeds 128 candidate resumption points.");
                if (caller.Symbols.VisibleBefore(caller.Method.MoveNext, point.Yield) is not { } source) continue;
                points.Add(new(CreateCodePoint(parentModule.Native, point.ResumeMethod, point.Resume), source));
            }
            if (points.Count == 0) throw new DebugException("unavailable", "The awaiting caller has no mapped resumption point.");
            asyncReturn = context;
            return true;
        }
        catch (Exception failure)
        {
            if (ReferenceEquals(asyncReturn, context)) asyncReturn = null;
            try { ReleaseAsyncReturnContext(context, true, true); }
            catch (Exception cleanup) { throw new AggregateException("Async return preparation and cleanup failed.", failure, cleanup); }
            throw;
        }
    }

    private bool AsyncReturnBreakpoint(nint domain, nint thread, nint pointer)
    {
        var context = asyncReturn;
        var point = context?.Points.FirstOrDefault(item => item.Point.SameIdentity(pointer));
        if (point is null) return false;
        using var managedThread = NativeObject.Borrow(thread); using var active = managedThread.Object(15);
        using var frame = active.Query(ilFrameId); using var receiver = frame.Object(16, 0); using var reference = receiver.Query(referenceValueId);
        if (reference.UInt64(8) != context!.Receiver.UInt64(8)) { ContinueCallback(domain); return true; }
        ReleaseAsyncReturnContext(context, true, false); asyncReturn = null;
        awaitStep = new(context.Receiver, context.Id, context.Method, point.Source) { Resumed = true };
        AsyncProgress("resuming"); StartStepper(frame, "over"); ContinueCallback(domain); return true;
    }

    private void PublishAsyncReturn(AsyncReturn context)
    {
        if (!events.Writer.TryWrite(new("async_progress", AsyncStep: new(context.Id, context.Method, "waiting_for_caller", null))))
            throw new IOException("The native event queue is full.");
    }

    private void ReleaseAsyncReturn(bool deactivate = true)
    {
        var context = asyncReturn; asyncReturn = null;
        if (context is not null) ReleaseAsyncReturnContext(context, deactivate, true);
    }

    private static void ReleaseAsyncReturnContext(AsyncReturn context, bool deactivate, bool receiver)
    {
        Exception? failure = null;
        foreach (var point in context.Points)
        {
            try { if (deactivate) NativeObject.Check(point.Point.Call(3, 0)); }
            catch (Exception exception) { failure ??= exception; }
            finally { point.Point.Dispose(); }
        }
        context.Points.Clear(); if (receiver) context.Receiver.Dispose();
        if (failure is not null) throw new IOException("Unable to release async return breakpoints.", failure);
    }
}
