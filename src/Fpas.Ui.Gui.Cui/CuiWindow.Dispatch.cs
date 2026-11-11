using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Runtime.ExceptionServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private Action? resizeRenderer;
    private GCHandle resizeRoot;
    private ExceptionDispatchInfo? resizeFailure;

    /// <summary>Wakes the owner when asynchronous cleanup finishes, without retaining native pointers.</summary>
    /// <param name="completion">The cleanup to observe.</param>
    public void WakeWhenCompleted(Task completion)
    {
        ArgumentNullException.ThrowIfNull(completion);
        _ = completion.ContinueWith(_ => TryWake(), CancellationToken.None,
            TaskContinuationOptions.ExecuteSynchronously, TaskScheduler.Default);
    }

    /// <summary>Installs a cached-view renderer for the platform's modal resize loop.</summary>
    /// <param name="renderer">Owner-only drawing callback without application transitions; null detaches it.</param>
    public unsafe void SetResizeRenderer(Action? renderer)
    {
        CheckOpen();
        NativeMethods.ResizeRenderer(app, null, 0);
        if (resizeRoot.IsAllocated) resizeRoot.Free();
        resizeRenderer = renderer;
        if (renderer is null) return;
        resizeRoot = GCHandle.Alloc(this);
        NativeMethods.ResizeRenderer(app, &RenderResize, GCHandle.ToIntPtr(resizeRoot));
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static void RenderResize(nint ui, nint user)
    {
        var window = (CuiWindow)GCHandle.FromIntPtr(user).Target!;
        try
        {
            window.inFrame = true;
            window.trackedNodes.Clear();
            window.scrollRequests.Clear(); window.scrollNodes.Clear();
            window.usedModals.Clear();
            window.resizeRenderer?.Invoke();
            if (window.layouts.Count != 0) throw new InvalidOperationException("Unbalanced live-resize view.");
            window.ResolveHostLayout();
        }
        catch (Exception error) { window.resizeFailure ??= ExceptionDispatchInfo.Capture(error); }
        finally { window.inFrame = false; }
    }
}
