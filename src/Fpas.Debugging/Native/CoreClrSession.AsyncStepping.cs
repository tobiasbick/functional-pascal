using Fpas.Debugging.Core;
using Fpas.Debugging.Symbols;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private sealed class AwaitStep(NativeObject receiver, string id, string method, SourceLocation source)
    {
        internal NativeObject Receiver { get; } = receiver;
        internal string Id { get; } = id;
        internal string Method { get; } = method;
        internal SourceLocation Source { get; } = source;
        internal NativeObject? Yield, Resume;
        internal bool Resumed;
    }
    private AwaitStep? awaitStep;

    private bool StartAwaitStep(NativeObject frame, string kind)
    {
        if (kind == "out") return false;
        using var function = frame.Object(5); using var module = function.Object(3);
        var symbols = catalog.Probe(module.Name(6), module.ModuleIdentity()).Symbols;
        var (token, offset) = FramePosition(frame);
        var info = symbols?.AsyncMethod(token);
        if (info is null || info.MoveNext != token || symbols!.Position(token, (int)offset) is not { } source) return false;
        var point = info.Awaits.Select(point => (Point: point, Source: symbols.VisibleBefore(token, point.Yield)))
            .Where(item => item.Source?.Line == source.Line && item.Source.Column >= source.Column)
            .OrderBy(item => item.Source!.Column).FirstOrDefault();
        if (point.Source is null) return false;
        using var receiver = frame.Object(16, 0); using var reference = receiver.Query(referenceValueId);
        var identity = AsyncIdentity(receiver);
        var method = symbols.MethodName(info.Kickoff);
        var context = new AwaitStep(HeapHandle(reference, 1), identity, method, point.Source);
        try
        {
            context.Yield = CreateCodePoint(module, info.MoveNext, point.Point.Yield);
            context.Resume = CreateCodePoint(module, point.Point.ResumeMethod, point.Point.Resume);
            awaitStep = context;
            return true;
        }
        catch (Exception failure)
        {
            if (ReferenceEquals(awaitStep, context)) awaitStep = null;
            try { ReleaseAwaitContext(context, deactivate: true); }
            catch (Exception cleanup) { throw new AggregateException("Async step preparation and cleanup failed.", failure, cleanup); }
            throw;
        }
    }

    private bool AsyncBreakpoint(nint domain, nint thread, nint pointer)
    {
        if (AsyncEntryBreakpoint(domain, thread, pointer)) return true;
        if (AsyncReturnBreakpoint(domain, thread, pointer)) return true;
        var context = awaitStep;
        if (context is null) return false;
        var yielded = context.Yield?.SameIdentity(pointer) == true;
        var resumed = context.Resume?.SameIdentity(pointer) == true;
        if (!yielded && !resumed) return false;
        using var managedThread = NativeObject.Borrow(thread); using var active = managedThread.Object(15);
        using var frame = active.Query(ilFrameId); using var receiver = frame.Object(16, 0);
        using var reference = receiver.Query(referenceValueId);
        if (reference.UInt64(8) != context.Receiver.UInt64(8)) { ContinueCallback(domain); return true; }
        if (stepper is not null) { NativeObject.Check(stepper.Call(4)); stepper.Dispose(); stepper = null; }
        if (yielded)
        {
            ReleaseAsyncEntry();
            NativeObject.Check(context.Yield!.Call(3, 0)); context.Yield.Dispose(); context.Yield = null;
            AsyncProgress("yielded"); ContinueCallback(domain); return true;
        }
        NativeObject.Check(context.Resume!.Call(3, 0)); context.Resume.Dispose(); context.Resume = null;
        if (context.Yield is not null) { NativeObject.Check(context.Yield.Call(3, 0)); context.Yield.Dispose(); context.Yield = null; }
        context.Resumed = true; AsyncProgress("resuming");
        StartStepper(frame, "over"); ContinueCallback(domain); return true;
    }

    private bool ContinueAwaitRegion(NativeObject frame, ModuleSymbols symbols, uint token, SourceLocation? location, nint domain)
    {
        if (awaitStep is not { } context || symbols.AsyncMethod(token)?.MoveNext != token || location is null || !SamePath(location.Path, context.Source.Path)
            || location.Line != context.Source.Line || location.Column >= context.Source.EndColumn) return false;
        using var receiver = frame.Object(16, 0); using var reference = receiver.Query(referenceValueId);
        if (reference.UInt64(8) != context.Receiver.UInt64(8)) return false;
        if (++hiddenTransitions > 4096) throw new DebugException("limit_exceeded", "Async resumption exceeded its hidden-transition limit.");
        StartStepper(frame, !context.Resumed && stepping == "into" ? "into" : "over"); ContinueCallback(domain); return true;
    }

    private void AsyncProgress(string phase)
    {
        var context = awaitStep;
        var progress = context is null ? null : new AsyncStepInfo(context.Id, context.Method, phase, context.Source);
        if (!events.Writer.TryWrite(new("async_progress", AsyncStep: progress)) && !exited.Task.IsCompleted && disposal is null)
            throw new IOException("The native event queue is full.");
    }

    private void ReleaseAwaitStep(bool deactivate = true)
    {
        var context = awaitStep; awaitStep = null;
        if (context is null) return;
        ReleaseAwaitContext(context, deactivate); AsyncProgress("completed");
    }

    private static void ReleaseAwaitContext(AwaitStep context, bool deactivate)
    {
        Exception? failure = null;
        foreach (var point in new[] { context.Yield, context.Resume })
        {
            if (point is null) continue;
            try { if (deactivate) NativeObject.Check(point.Call(3, 0)); }
            catch (Exception exception) { failure ??= exception; }
            finally { point.Dispose(); }
        }
        context.Receiver.Dispose();
        if (failure is not null) throw new IOException("Unable to release async stepping breakpoints.", failure);
    }
}
