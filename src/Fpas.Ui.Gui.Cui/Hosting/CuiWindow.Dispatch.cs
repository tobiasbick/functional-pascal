// [Documentation](../../../docs/ui/gui/cui.md#window-ownership-and-frames)
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
    private int resizeRenderVersion;
    private bool renderingResize;

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
    /// <remarks>The native resize loop owns frame completion while the renderer runs.</remarks>
    public unsafe void SetResizeRenderer(Action? renderer)
    {
        CheckOpen();
        CheckOutsideResizeRenderer();
        NativeMethods.ResizeRenderer(app, null, 0);
        if (resizeRoot.IsAllocated) resizeRoot.Free();
        resizeRenderer = renderer;
        if (renderer is null) return;
        resizeRoot = GCHandle.Alloc(this);
        NativeMethods.ResizeRenderer(app, &RenderResize, GCHandle.ToIntPtr(resizeRoot));
    }

    private void CheckOutsideResizeRenderer()
    {
        if (renderingResize)
            throw new InvalidOperationException("A live-resize renderer cannot change the window or frame lifecycle.");
        if (disabledScopeDepth != 0)
            throw new InvalidOperationException("Finish the disabled drawing scope before changing the frame lifecycle.");
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static void RenderResize(nint ui, nint user)
    {
        var window = (CuiWindow)GCHandle.FromIntPtr(user).Target!;
        try
        {
            window.resizeRenderVersion++;
            window.renderingResize = true;
            window.inFrame = true;
            window.trackedNodes.Clear();
            window.scrollRequests.Clear();
            window.scrollNodes.Clear();
            window.scrollMetricNodes.Clear();
            window.usedModals.Clear();
            window.moneyDrawn.Clear();
            window.resizeRenderer?.Invoke();
            if (window.layouts.Count != 0) throw new InvalidOperationException("Unbalanced live-resize view.");
            window.ResolveHostLayout();
            window.lastDroppedFileCount = NativeMethods.DroppedFileCount(window.context);
            window.discardedGeometry = false;
        }
        catch (Exception error)
        {
            // Drop scopes left open by the failed view so the next frame can begin.
            window.layouts.Clear();
            window.resizeFailure ??= ExceptionDispatchInfo.Capture(error);
        }
        finally { window.inFrame = false; window.renderingResize = false; }
    }
}
