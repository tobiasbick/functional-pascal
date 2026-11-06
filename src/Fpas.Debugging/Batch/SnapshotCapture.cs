using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;

namespace Fpas.Debugging.Batch;

internal sealed class SnapshotCapture(DebugSessionHost host, DebugSnapshot stopped, BatchReport report, CancellationToken token)
{
    private readonly Dictionary<string, string> objects = new(StringComparer.Ordinal);
    private int valueCount;
    private int reservedExpressions;
    private long Stop => stopped.StopId ?? throw new DebugException("invalid_state", "No stopped context is available.");

    internal async Task Capture(string[] expressions)
    {
        var threads = await host.Threads(Stop, token);
        report.Capture.TotalThreads = threads.Length;
        var selected = threads.FirstOrDefault(item => item.IsStopThread);
        var first = selected is null ? null : await host.Stack(new(Stop, selected.Id, Count: 1), token);
        var frame = first?.Items.FirstOrDefault();
        report.Stop = new(stopped.Reason ?? "stopped", selected?.Id, frame?.Source);
        reservedExpressions = expressions.Length;
        foreach (var expression in expressions)
        {
            reservedExpressions--;
            if (frame is null) { report.Expressions.Add(new(expression, Error: new("unavailable", "The stop has no managed frame."))); continue; }
            try
            {
                var result = await host.Evaluate(new(Stop, frame.Id, expression), token);
                report.Expressions.Add(new(expression, result.Value is null ? null : await Value(result.Value, 0), result.Error));
            }
            catch (DebugException failure) when (LocalFailure(failure)) { report.Expressions.Add(new(expression, Error: new(failure.Code, failure.Message))); }
        }
        if (stopped.Reason == "exception")
        {
            var exception = await host.ExceptionInfo(Stop, token);
            report.Exception = new(exception.Type, exception.Message, exception.Policy, exception.Source, exception.ThrowStack, exception.Truncated, exception.Status);
        }
        if (threads.Length > 64) report.Capture.Omit("threads");
        foreach (var thread in threads.Take(64))
        {
            if (report.Frames.Count >= 256) { report.Capture.Omit("frames"); break; }
            var stack = await host.Stack(new(Stop, thread.Id, Count: Math.Min(32, 256 - report.Frames.Count)), token);
            var clipped = stack.Truncated || stack.Total > stack.Items.Length;
            report.Threads.Add(new(thread.Id, thread.Name, thread.IsStopThread, stack.Total, stack.OmittedNativeChains, clipped));
            if (clipped) report.Capture.Omit("frames");
            foreach (var item in stack.Items)
            {
                var index = report.Frames.Count;
                report.Frames.Add(new(index, item.ThreadId, item.Name, item.Module, item.MethodToken, item.IlOffset, item.Source,
                    item.Status, item.Approximate, item.Generated, item.Async?.Method, item.Async?.State));
                if (item.Status != "available") continue;
                try
                {
                    await Bindings(item.Id, index, "source");
                    await Bindings(item.Id, index, "statics");
                }
                catch (DebugException failure) when (failure.Code is "unavailable" or "unsupported_type" or "limit_exceeded")
                { report.Capture.Omit("bindings"); }
            }
        }
    }

    private async Task Bindings(long frame, int index, string scope)
    {
        for (var start = 0; start < 4096 && valueCount < 4096;)
        {
            var page = scope == "statics" ? await host.StaticBindings(new(Stop, frame, Start: start, Count: 64), token)
                : await host.FrameBindings(new(Stop, frame, Start: start, Count: 64), token);
            foreach (var item in page.Items)
            {
                if (valueCount >= 4096) { report.Capture.Omit("bindings"); break; }
                report.Bindings.Add(new(index, scope, await Value(item, 0)));
            }
            start += page.Items.Length;
            if (start >= page.Total) return;
            if (page.Items.Length == 0) { report.Capture.Omit("bindings"); return; }
        }
        report.Capture.Omit("bindings");
    }

    private async Task<ReportValue> Value(DebugBinding value, int depth)
    {
        valueCount++;
        string? omission = null; var children = new List<ReportValue>();
        if (value.Reference is { } reference)
        {
            if (depth >= 8 || valueCount + reservedExpressions >= 4096) omission = depth >= 8 ? "value_depth" : "bindings";
            else
            {
                var view = value.SourceChildren is not null ? "source" : "raw";
                try
                {
                    var page = await host.ValueChildren(new(Stop, reference, Count: 64, View: view), token);
                    foreach (var item in page.Items)
                    {
                        if (valueCount + reservedExpressions >= 4096) { omission = "bindings"; break; }
                        children.Add(await Value(item, depth + 1));
                    }
                    if (page.Truncated || page.Total > children.Count) omission ??= "children";
                }
                catch (DebugException failure) when (LocalFailure(failure))
                { value = value with { Error = new(failure.Code, failure.Message), Status = "unavailable" }; omission = "inspection_failure"; }
            }
        }
        if (omission is not null) report.Capture.Omit(omission);
        string? identity = null;
        if (value.ObjectId is { } original)
        {
            if (!objects.TryGetValue(original, out identity)) objects.Add(original, identity = "o" + (objects.Count + 1));
        }
        return new(value.Name, value.Type, value.Kind, value.Scalar, value.Status, value.Truncated || omission is not null, value.RawUtf16,
            value.Error, identity, value.Cycle, Type(value.RuntimeType), value.Lengths, value.LowerBounds, value.SourceChildren ?? value.Children,
            value.SourceError, value.Constant, children.ToArray(), omission);
    }

    private static ReportType? Type(DebugTypeIdentity? type) => type is null ? null : new(type.Name, type.ElementKind, type.ModuleIdentity,
        type.Token, type.Arguments?.Select(item => Type(item)!).ToArray(), type.Rank, type.Exact);

    private static bool LocalFailure(DebugException failure) => failure.Code is "unavailable" or "unsupported_type" or "unsupported_view" or "limit_exceeded";
}
