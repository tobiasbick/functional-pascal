namespace Fpas.Debugging.Core;

internal sealed partial class WorkerSession
{
    internal ExceptionPolicy SetExceptions(ExceptionPolicy policy)
    {
        lock (gate)
        {
            if (stopping || native is null) throw new DebugException("invalid_state", "Prepare a live target first.");
            return native.SetExceptions(policy);
        }
    }

    internal DebugExceptionInfo Exception(long stopId)
    {
        lock (gate) { CheckStop(stopId); return native!.ExceptionInfo(); }
    }

    internal DebugThread[] Threads(long stopId)
    {
        lock (gate) { CheckStop(stopId); return native!.Threads(); }
    }

    internal StackPage Stack(StackRequest request)
    {
        lock (gate) { CheckStop(request.StopId); return native!.Stack(request.ThreadId, request.Start, request.Count); }
    }

    internal BindingPage FrameBindings(FrameRequest request)
    {
        lock (gate) { CheckStop(request.StopId); return native!.FrameBindings(request.FrameId, request.Start, request.Count, request.View); }
    }

    internal BindingPage ValueChildren(ValueRequest request)
    {
        lock (gate) { CheckStop(request.StopId); return native!.ValueChildren(request.Reference, request.Start, request.Count, request.View); }
    }

    internal BindingPage StaticBindings(StaticRequest request)
    {
        lock (gate) { CheckStop(request.StopId); return native!.StaticBindings(request.FrameId, request.TypeName, request.Start, request.Count); }
    }

    internal EvaluationResult Evaluate(EvaluationRequest request)
    {
        lock (gate) { CheckStop(request.StopId); return native!.Evaluate(request.FrameId, request.Expression); }
    }
}
