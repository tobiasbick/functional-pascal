using System.Text.Json.Nodes;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Dap;

internal sealed partial class DapServer
{
    private async Task<JsonObject> ThreadList(CancellationToken token)
    {
        if (Host.Snapshot.State is "exited" or "failed") return new() { ["threads"] = new JsonArray() };
        var all = await Host.Threads(Stop, token);
        return new() { ["threads"] = new JsonArray(all.Select(thread => (JsonNode)new JsonObject { ["id"] = checked((int)thread.Id), ["name"] = thread.Name }).ToArray()) };
    }

    private async Task<JsonObject> Stack(JsonObject args, CancellationToken token)
    {
        var stop = Stop; var start = args.Number("startFrame", 0); var count = args.Number("levels", 0); var unpaged = count == 0;
        if (count == 0) count = 128;
        if (start < 0 || count is < 1 or > 128 || args.Number("threadId") <= 0) throw DapArguments.Invalid("stackTrace page/threadId");
        var stack = await Host.Stack(new(stop, checked((uint)args.Number("threadId")), start, count), token);
        if (unpaged && (stack.Truncated || stack.Total is null || stack.Total > start + stack.Items.Length))
            throw new DebugException("limit_exceeded", "The unpaged stack exceeds 128 frames; request explicit startFrame/levels pages.");
        var items = new JsonArray();
        foreach (var frame in stack.Items)
        {
            var item = new JsonObject { ["id"] = frames.Add(stop, frame.Id), ["name"] = frame.Name, ["line"] = frame.Source is { } line ? ToLine(line.Line) : 0,
                ["column"] = frame.Source is { } column ? ToColumn(column.Column) : 0, ["moduleId"] = frame.Module };
            if (frame.Source is { } source)
            { item["source"] = Source(source.Path); item["endLine"] = ToLine(source.EndLine); item["endColumn"] = ToColumn(source.EndColumn); }
            if (frame.Status != "available" || frame.Generated) item["presentationHint"] = "subtle";
            items.Add(item);
        }
        var body = new JsonObject { ["stackFrames"] = items };
        if (stack.Total is { } total) body["totalFrames"] = total;
        return body;
    }

    private async Task<JsonObject> Scopes(JsonObject args, CancellationToken token)
    {
        var stop = Stop; var frame = frames.Get(args.Number("frameId"), stop);
        async Task<JsonObject> Scope(string name, string view, bool statics = false)
        {
            var scope = new JsonObject { ["name"] = name, ["variablesReference"] = values.Add(stop, new(frame, View: view, Static: statics)), ["expensive"] = statics };
            try
            {
                var page = statics ? await Host.StaticBindings(new(stop, frame, Count: 1), token) : await Host.FrameBindings(new(stop, frame, Count: 1, View: view), token);
                if (variablePaging) scope["namedVariables"] = page.Total;
            }
            catch (DebugException failure) when (failure.Code is "unavailable" or "unsupported_type") { }
            return scope;
        }
        return new() { ["scopes"] = new JsonArray(await Scope("Locals and arguments", "source"), await Scope("Statics", "source", true), await Scope("Raw CLR storage", "raw")) };
    }

    private async Task<JsonObject> Variables(JsonObject args, CancellationToken token)
    {
        var stop = Stop; var handle = values.Get(args.Number("variablesReference"), stop);
        var start = args.Number("start", 0); var count = args.Number("count", 0); var unpaged = count == 0; if (unpaged) count = 128;
        if (start < 0 || count is < 1 or > 128) throw DapArguments.Invalid("variables page");
        var filter = args.Text("filter", "all");
        if (filter is not ("all" or "indexed" or "named")) throw DapArguments.Invalid("filter");
        BindingPage page = handle.Reference != 0 ? await Host.ValueChildren(new(stop, handle.Reference, start, count, handle.View), token)
            : handle.Static ? await Host.StaticBindings(new(stop, handle.Frame, Start: start, Count: count), token)
            : await Host.FrameBindings(new(stop, handle.Frame, start, count, handle.View), token);
        if (page.Truncated || unpaged && page.Total > start + page.Items.Length)
            throw new DebugException("limit_exceeded", "The requested value page exceeds its item/byte budget; request a smaller explicit start/count page.");
        return new() { ["variables"] = new JsonArray(page.Items.Where(value => filter == "all" || (value.Name.StartsWith('[') ? filter == "indexed" : filter == "named"))
            .Select(value => (JsonNode)Variable(value, stop, handle.View)).ToArray()) };
    }

    private JsonObject Variable(DebugBinding value, long stop, string view)
    {
        var reference = value.Reference is { } core ? values.Add(stop, new(Reference: core, View: view)) : 0;
        var result = new JsonObject { ["name"] = value.Name, ["value"] = Display(value), ["type"] = value.Type, ["variablesReference"] = reference,
            ["presentationHint"] = new JsonObject { ["attributes"] = new JsonArray("readOnly") } };
        if (variablePaging && value.Children is { } count)
        {
            if (value.Lengths is not null) result["indexedVariables"] = count;
            else result["namedVariables"] = view == "source" ? value.SourceChildren ?? count : count;
        }
        return result;
    }

    private static string Display(DebugBinding value)
    {
        var text = value.Status != "available" ? "<" + value.Status + (value.Error is null ? "" : ": " + value.Error.Code) + ">"
            : value.Kind == "null" ? "null" : value.Scalar ?? (value.Cycle ? "<cycle>" : "{" + value.Type + "}");
        return value.Truncated ? text + " … [truncated]" : text;
    }

    private async Task<JsonObject> Evaluate(JsonObject args, CancellationToken token)
    {
        var stop = Stop; long frame;
        if (args["frameId"] is not null) frame = frames.Get(args.Number("frameId"), stop);
        else
        {
            var thread = (await Host.Threads(stop, token)).FirstOrDefault(item => item.IsStopThread) ?? throw new DebugException("unavailable", "No stopped thread is available.");
            frame = (await Host.Stack(new(stop, thread.Id, Count: 1), token)).Items.FirstOrDefault()?.Id ?? throw new DebugException("unavailable", "No managed frame is available.");
        }
        var result = await Host.Evaluate(new(stop, frame, args.Text("expression")), token);
        if (result.Error is { } error) throw new DebugException(error.Code, error.Message);
        var variable = Variable(result.Value!, stop, "source"); variable["result"] = variable["value"]!.DeepClone(); variable.Remove("value"); variable.Remove("name");
        return variable;
    }

    private async Task<JsonObject> Exception(JsonObject args, CancellationToken token)
    {
        await ValidateThread(args, token);
        var info = await Host.ExceptionInfo(Stop, token);
        return new()
        {
            ["exceptionId"] = info.Type, ["description"] = info.Message.Text, ["breakMode"] = info.Policy == "raised" ? "always" : "unhandled",
            ["details"] = new JsonObject { ["message"] = info.Message.Text, ["typeName"] = info.Type,
                ["stackTrace"] = string.Join('\n', info.ThrowStack.Select(frame => frame.Name + (frame.Source is { } source ? $" ({source.Path}:{source.Line})" : ""))) }
        };
    }
}
