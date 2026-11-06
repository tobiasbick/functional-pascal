namespace Fpas.Debugging.Core;

internal sealed partial class WorkerSession
{
    internal BreakpointInfo[] Breakpoints()
    {
        lock (gate) return native?.Breakpoints() ?? [];
    }

    internal async Task<BreakpointInfo[]> SetBreakpoints(BreakpointsRequest request)
    {
        await execution.WaitAsync(lifetime.Token);
        try
        {
            Native.CoreClrSession selected;
            lock (gate)
            {
                if (stopping || native is null || snapshot.State is not ("starting" or "stopped" or "running"))
                    throw new DebugException("invalid_state", "Prepare a live target before configuring breakpoints.");
                if (request.StopId is { } stop) CheckStop(stop);
                selected = native;
            }
            return selected.SetBreakpoints(request);
        }
        finally { execution.Release(); }
    }
}
