using System.Runtime.InteropServices;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private string? stepping;
    private int hiddenTransitions;
    private (Guid Module, uint Method, string Instance, SourceLocation Source)? asyncOrigin;
    [StructLayout(LayoutKind.Sequential)] private struct StepRange { internal uint Start, End; }

    internal void Step(string kind, Action? prepared = null)
    {
        if (kind is not ("into" or "over" or "out")) throw new DebugException("invalid_argument", "Step kind must be into, over or out.");
        lock (gate)
        {
            using var frame = Frame();
            CancelStep(); stepping = kind; hiddenTransitions = 0;
            using var function = frame.Object(5); using var module = function.Object(3);
            var symbols = catalog.Probe(module.Name(6), module.ModuleIdentity()).Symbols;
            var (method, offset) = FramePosition(frame);
            if (symbols?.Position(method, (int)offset) is { } source && AsyncFrameFor(frame, symbols, method) is { Id: { } identity })
                asyncOrigin = (module.ModuleIdentity(), method, identity, source);
            try
            {
                if (kind == "into") WatchAsyncCalls(stoppedThread!.UInt32(4));
                var awaiting = kind == "out" ? StartAsyncReturn(frame) : StartAwaitStep(frame, kind);
                if (!awaiting || kind == "into") StartStepper(frame, kind);
            }
            catch (NativeOperationException exception) when (exception.Result is unchecked((int)0x80131304) or unchecked((int)0x80131306) or unchecked((int)0x80131334))
            {
                CancelStep(); throw new DebugException("unavailable", "Optimized code no longer exposes the receiver or storage required for this source step.");
            }
            catch { CancelStep(); throw; }
            prepared?.Invoke();
            if (awaitStep is not null) AsyncProgress("waiting_for_resume");
            if (asyncReturn is not null) PublishAsyncReturn(asyncReturn);
            ResumeProcess();
        }
    }

    private unsafe (uint Token, uint Offset) FramePosition(NativeObject frame)
    {
        if (breakpointFrame?.SameIdentity(frame.Pointer) == true) return breakpointPosition;
        uint offset = 0, mapping = 0;
        NativeObject.Check(((delegate* unmanaged[Stdcall]<nint, uint*, uint*, int>)frame.Slot(11))(frame.Pointer, &offset, &mapping));
        return (frame.UInt32(6), offset);
    }

    private unsafe void StartStepper(NativeObject frame, string kind)
    {
        stepper?.Dispose(); stepper = frame.Object(10);
        NativeObject.Check(stepper.Call(5, 0)); NativeObject.Check(stepper.Call(6, 0));
        if (kind == "out") { NativeObject.Check(stepper.Call(9)); return; }
        using var function = frame.Object(5);
        using var module = function.Object(3);
        var metadata = catalog.Probe(module.Name(6), module.ModuleIdentity()).Symbols;
        var (token, offset) = FramePosition(frame);
        if (metadata is null) { NativeObject.Check(stepper.Call(7, kind == "into" ? 1 : 0)); return; }
        var range = new StepRange { Start = offset, End = checked((uint)metadata.NextOffset(token, (int)offset)) };
        if (range.End <= range.Start) { NativeObject.Check(stepper.Call(9)); return; }
        NativeObject.Check(((delegate* unmanaged[Stdcall]<nint, int, StepRange*, uint, int>)stepper.Slot(8))
            (stepper.Pointer, kind == "into" ? 1 : 0, &range, 1));
    }

    internal void StepCompleted(nint domain, nint thread)
    {
        lock (gate)
        {
            if (stepping is null) { ContinueCallback(domain); return; }
            using var managedThread = NativeObject.Borrow(thread);
            using var active = managedThread.Object(15);
            using var frame = active.TryQuery(ilFrameId);
            if (frame is null)
            {
                ContinueHidden(managedThread, null, domain); return;
            }
            using var function = frame.Object(5);
            using var module = function.Object(3);
            var metadata = catalog.Probe(module.Name(6), module.ModuleIdentity()).Symbols;
            var (token, offset) = FramePosition(frame);
            var location = metadata?.Position(token, checked((int)offset));
            if (metadata is not null && ContinueAwaitRegion(frame, metadata, token, location, domain)) return;
            if (asyncOrigin is { } origin && metadata is not null && location is not null && origin.Module == module.ModuleIdentity()
                && token == origin.Method && location.Path == origin.Source.Path && location.Line == origin.Source.Line && location.Column == origin.Source.Column
                && AsyncFrameFor(frame, metadata, token)?.Id == origin.Instance)
            {
                if (++hiddenTransitions > 4096) throw new DebugException("limit_exceeded", "Async stepping exceeded its dispatch-transition limit.");
                StartStepper(frame, stepping == "into" ? "into" : "over"); ContinueCallback(domain); return;
            }
            if (metadata is null || !metadata.HasSourcePoints(token) || metadata.ShouldSkip(token))
            { ContinueHidden(managedThread, frame, domain); return; }
            if (location is null || !metadata.IsSourcePoint(token, (int)offset) && metadata.HasNextSourcePoint(token, (int)offset))
            {
                if (++hiddenTransitions > 4096) throw new DebugException("limit_exceeded", "Source stepping exceeded its hidden-transition limit.");
                StartStepper(frame, "over"); ContinueCallback(domain); return;
            }
            CancelStep(); Stopped("step", thread);
        }
    }

    private void ContinueHidden(NativeObject thread, NativeObject? frame, nint domain)
    {
        if (++hiddenTransitions > 4096) throw new DebugException("limit_exceeded", "Source stepping exceeded its hidden-transition limit.");
        if (frame is null)
        {
            stepper?.Dispose(); stepper = thread.Object(12); NativeObject.Check(stepper.Call(9));
        }
        else StartStepper(frame, "out");
        ContinueCallback(domain);
    }

    private void CancelStep()
    {
        if (stepper is not null) { NativeObject.Check(stepper.Call(4)); stepper.Dispose(); stepper = null; }
        ReleaseAwaitStep();
        ReleaseAsyncEntry();
        ReleaseAsyncReturn();
        asyncOrigin = null;
        stepping = null;
    }
}
