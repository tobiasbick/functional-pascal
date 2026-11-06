using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Threading.Channels;
using Fpas.Debugging.Symbols;
using Fpas.Debugging.Hosting;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession : IAsyncDisposable
{
    private readonly object gate = new();
    private readonly ModuleSymbols symbols;
    private readonly SymbolCatalog catalog;
    private readonly bool stopAtEntry;
    private readonly ManagedCallbacks callbacks;
    private readonly DbgShim.StartupCallback startupCallback;
    private readonly Channel<NativeStop> events = Channel.CreateBounded<NativeStop>(64);
    private readonly TaskCompletionSource initialized = new(TaskCreationOptions.RunContinuationsAsynchronously);
    private readonly TaskCompletionSource exited = new(TaskCreationOptions.RunContinuationsAsynchronously);
    private NativeObject? debugger, process, stoppedThread, entryBreakpoint, stepper;
    private nint registration, resumeHandle;
    private Process? target;
    private LaunchBroker? broker;
    private Task? disposal;
    private bool atEntry;
    internal int ProcessId => target?.Id ?? 0;
    internal bool IsStopped { get { lock (gate) return stoppedThread is not null; } }

    private CoreClrSession(string program, SourceBreakpoint? initial)
    {
        symbols = new(program);
        try
        {
            symbols.ValidateSources();
            if (symbols.EntryToken == 0) throw new ArgumentException("The target assembly has no managed executable entry point.");
            if (initial is not null)
            {
                ValidateSource(initial.Source);
                if (symbols.ContainsSource(initial.Source)) _ = symbols.Resolve(initial.Source, initial.Line, initial.Column);
                requests.Add(new(++breakpointSequence, initial, "pending"));
            }
        }
        catch { symbols.Dispose(); throw; }
        catalog = new(symbols);
        stopAtEntry = initial is null;
        callbacks = new(this); startupCallback = RuntimeStarted;
    }

    internal static async Task<CoreClrSession> Launch(string dotnet, string program, string cwd, string source, int line, CancellationToken token, WorkerLauncher? launcher = null)
    {
        var session = await Prepare(dotnet, program, cwd, source, line, token, launcher);
        await session.Initialize(token);
        return session;
    }

    // The worker can publish a starting state and establish target I/O before execution.
    internal static async Task<CoreClrSession> Prepare(string dotnet, string program, string cwd, string source, int line, CancellationToken token, WorkerLauncher? launcher = null)
        => await Prepare(dotnet, program, cwd, new SourceBreakpoint(source, line), token, launcher);

    internal static async Task<CoreClrSession> Prepare(string dotnet, string program, string cwd, SourceBreakpoint? initial, CancellationToken token, WorkerLauncher? launcher = null, IReadOnlyList<string>? arguments = null)
    {
        token.ThrowIfCancellationRequested();
        if (OperatingSystem.IsLinux() && (dotnet.Contains('"') || dotnet.EndsWith('\\')))
            throw new ArgumentException("The Linux debugger host path cannot contain a quote or end in a backslash. Select another installed dotnet host path.");
        if (!Path.IsPathFullyQualified(dotnet) || !File.Exists(dotnet)) throw new ArgumentException("Use an absolute path to an installed dotnet host.");
        if (!Path.IsPathFullyQualified(program) || !File.Exists(program)) throw new ArgumentException("Use an absolute path to a built managed executable.");
        if (!Path.IsPathFullyQualified(cwd) || !Directory.Exists(cwd)) throw new IOException("The target working directory does not exist.");
        var argv = arguments?.ToArray() ?? [];
        if (argv.Length > 1024 || argv.Any(argument => argument is null || argument.Contains('\0')) || argv.Sum(argument => (long)argument.Length) > 65536)
            throw new ArgumentException("Target arguments require at most 1024 elements and 65536 UTF-16 code units, without NUL.");
        var session = new CoreClrSession(program, initial);
        try
        {
            if (OperatingSystem.IsLinux())
            {
                session.broker = await LaunchBroker.Start(launcher ?? WorkerLauncher.Current(), new(dotnet, program, cwd, argv), token);
                session.target = CreatedProcess.Capture((uint)session.broker.TargetId);
                NativeObject.Check(DbgShim.Instance.Startup((uint)session.ProcessId, Marshal.GetFunctionPointerForDelegate(session.startupCallback), 0, out session.registration));
            }
            else session.CreateSuspended(dotnet, program, cwd, argv);
            return session;
        }
        catch (Exception failure)
        {
            try { await session.DisposeAsync(); }
            catch (Exception cleanup) { throw new AggregateException("Debug preparation failed and cleanup reported an additional failure.", failure, cleanup); }
            throw;
        }
    }

    internal async Task Initialize(CancellationToken token)
    {
        try
        {
            token.ThrowIfCancellationRequested();
            if (broker is not null) await broker.Resume(token);
            else
            {
                if (resumeHandle == 0) throw new InvalidOperationException("The prepared target has already been resumed.");
                NativeObject.Check(DbgShim.Instance.Resume(resumeHandle));
                NativeObject.Check(DbgShim.Instance.CloseResume(resumeHandle)); resumeHandle = 0;
            }
            var targetExit = WaitForStartupExit(token);
            await Task.WhenAny(initialized.Task, targetExit).WaitAsync(token);
            if (!initialized.Task.IsCompleted)
            {
                await targetExit;
                throw new IOException("The target host exited before CoreCLR debugging initialization.");
            }
            await initialized.Task.WaitAsync(token);
        }
        catch (Exception failure)
        {
            try { await DisposeAsync(); }
            catch (Exception cleanup) { throw new AggregateException("Debug launch failed and cleanup reported an additional failure.", failure, cleanup); }
            throw;
        }
    }

    private void CreateSuspended(string dotnet, string program, string cwd, IReadOnlyList<string> arguments)
    {
        var shim = DbgShim.Instance;
        NativeObject.Check(shim.CreateProcess(DbgShim.CommandLine(dotnet, [program, .. arguments]), 1, 0, cwd, out var id, out resumeHandle));
        target = CreatedProcess.Capture(id);
        NativeObject.Check(shim.Startup(id, Marshal.GetFunctionPointerForDelegate(startupCallback), 0, out registration));
    }

    private unsafe void RuntimeStarted(nint pointer, nint parameter, int result)
    {
        try
        {
            NativeObject.Check(result);
            lock (gate) debugger = NativeObject.Borrow(pointer);
            NativeObject.Check(debugger.Call(3));
            using var handler = callbacks.Export();
            NativeObject.Check(debugger.CallPointer(5, handler.Pointer));
            nint connected = 0;
            NativeObject.Check(((delegate* unmanaged[Stdcall]<nint, uint, int, nint*, int>)debugger.Slot(8))
                (debugger.Pointer, (uint)ProcessId, 0, &connected));
            lock (gate) process = new(connected);
            initialized.TrySetResult();
        }
        catch (Exception exception) { NativeFailure(exception); }
    }

    private async Task WaitForStartupExit(CancellationToken token)
    {
        while (!initialized.Task.IsCompleted)
        {
            lock (gate)
            {
                // DBI owns waitpid after attach. Before that, native-created Linux children
                // must be reaped here; Process.WaitForExitAsync alone keeps seeing a zombie.
                if (debugger is null && CreatedProcess.ReapExitedChild(target!)) return;
            }
            await Task.Delay(10, token);
        }
    }

    internal void ContinueCallback(nint controller)
    {
        using var owned = NativeObject.Borrow(controller);
        NativeObject.Check(owned.Call(4, 0));
    }

    internal void Stopped(string reason, nint thread, long[]? hit = null)
    {
        lock (gate)
        {
            if (reason != "step") CancelStep();
            if (reason != "exception") stoppedException = null;
            atEntry = reason == "entry";
            ReleaseBreakpointFrame();
            stoppedThread?.Dispose(); stoppedThread = NativeObject.Borrow(thread);
            if (!events.Writer.TryWrite(new(reason, hit, requests.ToArray(), publishedModuleIssues))) throw new IOException("The native stop queue is full.");
        }
    }

    internal void Exited() { exited.TrySetResult(); events.Writer.TryComplete(); }
    internal void NativeFailure(Exception exception)
    {
        initialized.TrySetException(exception); events.Writer.TryComplete(exception);
    }

    internal async ValueTask<string> WaitForStop(CancellationToken token)
    {
        while (true) { var next = await WaitForEvent(token); if (next.Reason is not ("breakpoints_changed" or "modules_changed" or "async_progress")) return next.Reason; }
    }
    internal ValueTask<NativeStop> WaitForEvent(CancellationToken token) => events.Reader.ReadAsync(token);
    internal Task WaitForExit(CancellationToken token) => exited.Task.WaitAsync(token);

    internal async Task<int> ExitCode(CancellationToken token) => (await ExitStatus(token)).Code;

    internal async Task<(int Code, int? Signal)> ExitStatus(CancellationToken token)
    {
        await exited.Task.WaitAsync(token);
        if (broker is not null)
        {
            var status = await broker.ExitStatus(token);
            return (status.Code, status.Signal);
        }
        var selected = target ?? throw new IOException("No launched target exists.");
        await selected.WaitForExitAsync(token);
        return (selected.ExitCode, null);
    }

    internal void Continue(Action? prepared = null)
    {
        lock (gate)
        {
            CancelStep();
            prepared?.Invoke();
            if (HitEntryLocation()) return;
            ResumeProcess();
        }
    }

    private void ResumeProcess()
    {
        if (stoppedThread is null) throw new InvalidOperationException("The target is not stopped.");
        atEntry = false;
        stoppedException = null;
        ReleaseBreakpointFrame();
        ReleaseFrames(); stoppedThread.Dispose(); stoppedThread = null;
        NativeObject.Check(process!.Call(4, 0));
    }

    public ValueTask DisposeAsync()
    {
        lock (gate) return new(disposal ??= Task.Run(DisposeCore));
    }

    private async Task DisposeCore()
    {
        Exception? failure = null;
        void Release(Action action)
        {
            try { action(); }
            catch (Exception exception) { failure ??= exception; }
        }
        if (registration != 0)
        {
            Release(() => NativeObject.Check(DbgShim.Instance.Unregister(registration)));
            registration = 0;
        }
        if (process is not null && !exited.Task.IsCompleted)
        {
            try { NativeObject.Check(process.Call(10, 0)); } catch (IOException) { }
        }
        if (target is not null)
        {
            try
            {
                if (!target.HasExited) target.Kill(entireProcessTree: true);
                using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(5));
                if (broker is not null) await broker.ExitCode(deadline.Token);
                else if (process is null) await CreatedProcess.WaitForUnattachedExit(target, deadline.Token);
                else await target.WaitForExitAsync(deadline.Token);
                if (process is not null) await exited.Task.WaitAsync(deadline.Token);
            }
            catch (Exception exception) { failure ??= exception; }
            finally { Release(target.Dispose); }
        }
        Release(() => stoppedThread?.Dispose()); Release(() => stepper?.Dispose());
        Release(ReleaseBreakpoints); Release(() => process?.Dispose());
        if (resumeHandle != 0)
        {
            Release(() => NativeObject.Check(DbgShim.Instance.CloseResume(resumeHandle)));
            resumeHandle = 0;
        }
        if (debugger is not null)
        {
            Release(() => NativeObject.Check(debugger.Call(4)));
            Release(debugger.Dispose);
        }
        GC.KeepAlive(callbacks); GC.KeepAlive(startupCallback);
        Release(ReleaseFrames); Release(catalog.Dispose);
        Release(ReleaseMetadata);
        Release(ReleaseAsyncInstances);
        Release(() => ReleaseAwaitStep(deactivate: false));
        Release(() => ReleaseAsyncEntry(deactivate: false));
        Release(() => ReleaseAsyncReturn(deactivate: false));
        if (broker is not null)
        {
            try { await broker.DisposeAsync(); }
            catch (Exception exception) { failure ??= exception; }
        }
        events.Writer.TryComplete();
        if (failure is not null) throw new IOException("Debugger shutdown reported a cleanup failure.", failure);
    }
}
