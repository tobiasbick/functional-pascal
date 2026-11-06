using Fpas.Debugging.Core;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private sealed record LoadedModule(long Id, NativeObject Native, string Path, Guid Identity);
    private readonly record struct PointKey(long Module, uint Method, int Offset);
    private sealed record BoundPoint(NativeObject Native, long[] Ids, bool NativeEntry = false);
    private readonly List<LoadedModule> modules = [];
    private readonly List<BreakpointInfo> requests = [];
    private readonly Dictionary<PointKey, BoundPoint> points = [];
    private readonly Dictionary<PointKey, DebugProblem> bindingErrors = [];
    private long moduleSequence, breakpointSequence;
    private BreakpointInfo[] publishedBreakpoints = [];
    private DebugModuleIssue[] publishedModuleIssues = [];
    private bool entryInstalled;
    private NativeObject? breakpointFrame;
    private (uint Method, uint Offset) breakpointPosition;

    private static void ValidateSource(string source)
    {
        if (!Path.IsPathFullyQualified(source) || !File.Exists(source))
            throw new ArgumentException("Use an absolute path to an existing source file.");
    }

    internal BreakpointInfo[] Breakpoints() { lock (gate) return requests.ToArray(); }

    internal BreakpointInfo[] SetBreakpoints(BreakpointsRequest request)
    {
        if (!Path.IsPathFullyQualified(request.Source)) throw new ArgumentException("Use an absolute source path.");
        if (request.Positions.Length > 64 || request.Positions.Any(point => point.Line <= 0 || point.Column is <= 0))
            throw new ArgumentException("Use at most 64 positive breakpoint positions.");
        if (request.Positions.Length != 0) ValidateSource(request.Source);
        NativeObject? controller;
        lock (gate) controller = process is not null && stoppedThread is null ? NativeObject.Borrow(process.Pointer) : null;
        using (controller)
        {
            if (controller is not null) NativeObject.Check(controller.Call(3, -1));
            try
            {
                lock (gate)
                {
                    var source = Path.GetFullPath(request.Source);
                    var old = requests.Where(item => SamePath(item.Requested.Source, source)).ToArray();
                    var selected = request.Positions.Distinct().Select(position =>
                        old.FirstOrDefault(item => item.Requested.Line == position.Line && item.Requested.Column == position.Column)
                        ?? new BreakpointInfo(++breakpointSequence, new(source, position.Line, position.Column), "pending")).ToArray();
                    if (requests.Count - old.Length + selected.Length > 64) throw new DebugException("limit_exceeded", "A session supports at most 64 source breakpoints.");
                    requests.RemoveAll(item => SamePath(item.Requested.Source, source)); requests.AddRange(selected);
                    BindBreakpoints();
                    return requests.Where(item => SamePath(item.Requested.Source, source)).ToArray();
                }
            }
            finally { if (controller is not null) NativeObject.Check(controller.Call(4, 0)); }
        }
    }

    private static bool SamePath(string left, string right) => string.Equals(Path.GetFullPath(left), Path.GetFullPath(right),
        OperatingSystem.IsWindows() ? StringComparison.OrdinalIgnoreCase : StringComparison.Ordinal);

    internal void ModuleLoaded(nint pointer)
    {
        lock (gate)
        {
            if (modules.Any(item => item.Native.SameIdentity(pointer))) return;
            if (modules.Count >= 1024) throw new DebugException("limit_exceeded", "The loaded module catalog exceeds 1024 entries.");
            using var module = NativeObject.Borrow(pointer);
            var path = module.Name(6); var identity = module.ModuleIdentity();
            modules.Add(new(++moduleSequence, NativeObject.Borrow(pointer), path, identity));
            if (!entryInstalled && Path.IsPathFullyQualified(path) && SamePath(path, symbols.Path))
            {
                _ = catalog.Open(path, identity);
                using var function = module.Object(9, symbols.EntryToken);
                using var code = function.Object(6);
                entryBreakpoint = code.Object(7, 0);
                NativeObject.Check(entryBreakpoint.Call(3, 1));
                entryInstalled = true;
            }
            BindBreakpoints();
            ArmAsyncEntries(modules[^1]);
        }
    }

    internal void ModuleUnloaded(nint pointer)
    {
        lock (gate)
        {
            var module = modules.FirstOrDefault(item => item.Native.SameIdentity(pointer));
            if (module is null) return;
            foreach (var entry in asyncEntries.Where(entry => entry.Module == module.Id).ToArray())
            { entry.Breakpoint.Dispose(); asyncEntries.Remove(entry); }
            foreach (var key in points.Keys.Where(key => key.Module == module.Id).ToArray())
            { points[key].Native.Dispose(); points.Remove(key); }
            foreach (var key in bindingErrors.Keys.Where(key => key.Module == module.Id).ToArray()) bindingErrors.Remove(key);
            modules.Remove(module); module.Native.Dispose(); catalog.Remove(module.Path);
            BindBreakpoints();
        }
    }

    private void BindBreakpoints()
    {
        var available = modules.Select(module => (Module: module, Probe: catalog.Probe(module.Path, module.Identity))).ToArray();
        var issues = available.Where(item => item.Probe.Error is not null).Select(item => new DebugModuleIssue(item.Module.Path, item.Probe.Error!))
            .Distinct().OrderBy(item => item.Module, StringComparer.Ordinal).ToArray();
        if (!issues.SequenceEqual(publishedModuleIssues))
        {
            publishedModuleIssues = issues;
            if (!events.Writer.TryWrite(new("modules_changed", ModuleIssues: issues))) throw new IOException("The native event queue is full.");
        }
        var desired = new Dictionary<PointKey, (LoadedModule Module, List<long> Ids)>();
        var retainedKeys = new HashSet<PointKey>();
        for (var index = 0; index < requests.Count; index++)
        {
            var request = requests[index];
            var matches = available.Select(item => (item.Module, Symbols: item.Probe.Symbols))
                .Where(item => item.Symbols?.ContainsSource(request.Requested.Source) == true).ToArray();
            var next = request with { State = "pending", Actual = null, Error = null };
            if (matches.Length > 1) next = next with { State = "rejected", Error = new("ambiguous_source", "More than one loaded module contains this source.") };
            else if (matches.Length == 1)
            {
                var match = matches[0];
                try
                {
                    match.Symbols!.ValidateSource(request.Requested.Source);
                    var location = match.Symbols.Resolve(request.Requested.Source, request.Requested.Line, request.Requested.Column);
                    var key = new PointKey(match.Module.Id, location.Token, location.Offset);
                    retainedKeys.Add(key);
                    if (bindingErrors.TryGetValue(key, out var error))
                    {
                        requests[index] = next with { State = "rejected", Error = error };
                        continue;
                    }
                    if (!desired.TryGetValue(key, out var point)) desired.Add(key, point = (match.Module, []));
                    point.Ids.Add(request.Id);
                    next = next with { State = "bound", Actual = match.Symbols.Position(location.Token, location.Offset) };
                }
                catch (ArgumentException exception) { next = next with { State = "rejected", Error = new("non_executable", exception.Message) }; }
                catch (DebugException exception) { next = next with { State = "rejected", Error = new(exception.Code, exception.Message) }; }
            }
            requests[index] = next;
        }
        foreach (var key in bindingErrors.Keys.Except(retainedKeys).ToArray()) bindingErrors.Remove(key);
        foreach (var key in points.Keys.Except(desired.Keys).ToArray())
        { NativeObject.Check(points[key].Native.Call(3, 0)); points[key].Native.Dispose(); points.Remove(key); }
        foreach (var (key, wanted) in desired)
        {
            if (points.TryGetValue(key, out var existing)) { points[key] = existing with { Ids = wanted.Ids.ToArray() }; continue; }
            try { points.Add(key, new(CreateCodePoint(wanted.Module.Native, key.Method, key.Offset), wanted.Ids.ToArray())); }
            catch (DebugException exception) when (exception.Code is "unavailable" or "limit_exceeded")
            {
                var error = exception.Code == "limit_exceeded" ? new DebugProblem("limit_exceeded", exception.Message)
                    : new DebugProblem("unbound_breakpoint", "CoreCLR cannot bind this source point in the selected target.");
                bindingErrors[key] = error;
                foreach (var id in wanted.Ids)
                {
                    var index = requests.FindIndex(item => item.Id == id);
                    requests[index] = requests[index] with { State = "rejected", Error = error };
                }
            }
        }
        var changed = requests.ToArray();
        if (!changed.SequenceEqual(publishedBreakpoints))
        {
            publishedBreakpoints = changed;
            if (!events.Writer.TryWrite(new("breakpoints_changed", ChangedBreakpoints: changed))) throw new IOException("The native event queue is full.");
        }
    }

    internal void BreakpointHit(nint domain, nint thread, nint pointer)
    {
        lock (gate)
        {
            if (entryBreakpoint?.SameIdentity(pointer) == true)
            {
                NativeObject.Check(entryBreakpoint.Call(3, 0)); entryBreakpoint.Dispose(); entryBreakpoint = null;
                if (stopAtEntry) Stopped("entry", thread); else ContinueCallback(domain);
                return;
            }
            var selected = points.FirstOrDefault(item => item.Value.Native.SameIdentity(pointer));
            if (selected.Value is null) { if (!AsyncBreakpoint(domain, thread, pointer)) ContinueCallback(domain); return; }
            if (selected.Value.NativeEntry) { BindAtNativeEntry(selected.Key, selected.Value); ContinueCallback(domain); return; }
            CountHits(selected.Value);
            Stopped("breakpoint", thread, selected.Value.Ids);
            using var managedThread = NativeObject.Borrow(thread); using var active = managedThread.Object(15);
            breakpointFrame = active.Query(ilFrameId);
            breakpointPosition = (selected.Key.Method, checked((uint)selected.Key.Offset));
        }
    }

    private bool HitEntryLocation()
    {
        if (!atEntry || stoppedThread is null) return false;
        using var frame = Frame();
        using var function = frame.Object(5);
        using var module = function.Object(3);
        var loaded = modules.FirstOrDefault(item => item.Native.SameIdentity(module.Pointer));
        var (token, offset) = FramePosition(frame);
        if (loaded is null || !points.TryGetValue(new(loaded.Id, token, (int)offset), out var point)) return false;
        CountHits(point);
        using var thread = NativeObject.Borrow(stoppedThread.Pointer);
        Stopped("breakpoint", thread.Pointer, point.Ids);
        return true;
    }

    private void CountHits(BoundPoint point)
    {
        foreach (var id in point.Ids)
        {
            var index = requests.FindIndex(item => item.Id == id);
            requests[index] = requests[index] with { Hits = requests[index].Hits + 1 };
        }
    }

    private void ReleaseBreakpoints()
    {
        ReleaseBreakpointFrame();
        entryBreakpoint?.Dispose();
        foreach (var point in points.Values) point.Native.Dispose(); points.Clear();
        foreach (var module in modules) module.Native.Dispose(); modules.Clear();
    }

    private void ReleaseBreakpointFrame() { breakpointFrame?.Dispose(); breakpointFrame = null; }
}
