using System.Text.Json.Nodes;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Dap;

internal sealed partial class DapServer
{
    private async Task Observe()
    {
        try
        {
            while (!lifetime.IsCancellationRequested && !terminal)
            {
                await operations.WaitAsync(lifetime.Token);
                try
                {
                    var state = Host.Snapshot;
                    await Output();
                    foreach (var point in state.Breakpoints ?? [])
                    {
                        if (knownBreakpoints.TryGetValue(point.Id, out var previous) && previous == point.State) continue;
                        knownBreakpoints[point.Id] = point.State;
                        Event("breakpoint", new() { ["reason"] = "changed", ["breakpoint"] = Breakpoint(point) });
                    }
                    if (configured && state.State == "stopped" && state.StopId is { } stop && stop != reportedStop)
                    {
                        reportedStop = stop;
                        var selected = (await Host.Threads(stop, lifetime.Token)).FirstOrDefault(item => item.IsStopThread);
                        Event("stopped", new() { ["reason"] = state.Reason ?? "pause", ["threadId"] = selected is null ? null : checked((int)selected.Id),
                            ["allThreadsStopped"] = true, ["hitBreakpointIds"] = new JsonArray((state.HitBreakpoints ?? []).Select(id => (JsonNode?)JsonValue.Create(checked((int)id))).ToArray()) });
                    }
                    if (state.State is "exited" or "failed")
                    {
                        await Host.Stop(lifetime.Token); await Output(); FlushOutputLoss();
                        if (state.ExitCode is { } exit) Event("exited", new() { ["exitCode"] = exit });
                        if (state.Error is { } failure) Event("output", new() { ["category"] = "stderr", ["output"] = failure.Code + ": " + failure.Message + "\n" });
                        Terminated();
                    }
                }
                finally { operations.Release(); }
                if (!terminal) await Task.Delay(25, lifetime.Token);
            }
        }
        catch (OperationCanceledException) when (lifetime.IsCancellationRequested) { }
        catch (Exception)
        {
            try { await Host.DisposeAsync(); Terminated(); } catch (Exception) { }
            lifetime.Cancel();
        }
    }

    private bool outputLost;
    private void FlushOutputLoss()
    {
        if (!outputLost) return;
        Event("output", new() { ["category"] = "stderr", ["output"] = "[FPAS debugger output omitted by capture/transport limit]\n" });
        outputLost = false;
    }
    private Task Output()
    {
        var captured = Host.Output(outputCursor);
        outputLost |= captured.Truncated;
        foreach (var item in captured.Items)
        {
            if (outputLost && transport.Send(new() { ["type"] = "event", ["event"] = "output", ["body"] = new JsonObject
                { ["category"] = "stderr", ["output"] = "[FPAS debugger output omitted by capture/transport limit]\n" } }, lossy: true)) outputLost = false;
            if (!transport.Send(new() { ["type"] = "event", ["event"] = "output", ["body"] = new JsonObject { ["category"] = item.Stream, ["output"] = item.Text } }, lossy: true)) outputLost = true;
        }
        if (outputLost && transport.Send(new() { ["type"] = "event", ["event"] = "output", ["body"] = new JsonObject
            { ["category"] = "stderr", ["output"] = "[FPAS debugger output omitted by capture/transport limit]\n" } }, lossy: true)) outputLost = false;
        outputCursor = captured.NextSequence; return Task.CompletedTask;
    }
}
