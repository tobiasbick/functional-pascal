using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private readonly Dictionary<string, int> trackedNodes = new(StringComparer.Ordinal);
    private readonly Dictionary<string, uint> controlKeys = new(StringComparer.Ordinal);

    /// <summary>Lets the owner keep pumping while asynchronous shutdown completes.</summary>
    public bool DeferClose { get { CheckOpen(); return deferClose; } set { CheckOpen(); deferClose = value; } }
    private bool deferClose;

    /// <summary>Reports a native close request observed by the frame pump.</summary>
    public bool CloseRequested { get; private set; }

    /// <summary>Ends an unfinished view without presenting it.</summary>
    public void DiscardFrame()
    {
        CheckOpen();
        NativeMethods.FinishFrameForDestroy(app);
        layouts.Clear();
        trackedNodes.Clear();
        inFrame = false;
    }

    /// <summary>Requests native closure, observed by the next frame pump.</summary>
    public void RequestClose() { CheckOpen(); NativeMethods.Quit(app); }

    /// <summary>Makes the last widget fill its vertical parent's available width.</summary>
    public void FillLastWidth() { CheckFrame(); NativeMethods.FillLastWidth(context); }

    /// <summary>Records the last widget under a host identity for inspection after presentation.</summary>
    /// <param name="id">Nonempty key, unique within the current frame.</param>
    public void TrackLastControl(string id)
    {
        CheckFrame();
        TrackControl(id, NativeMethods.LastNode(context));
    }

    private void TrackControl(string id, int node)
    {
        ArgumentException.ThrowIfNullOrEmpty(id);
        trackedNodes.Add(id, node);
        uint key = NativeMethods.NodeKey(context, node);
        if (controlKeys.TryGetValue(id, out uint previous) && previous != key)
        {
            ForgetConfirmButton(previous);
            ForgetStatusButton(previous);
            NativeMethods.Forget(context, previous);
        }
        controlKeys[id] = key;
    }

    /// <summary>Releases native state retained by a control that has disappeared.</summary>
    /// <param name="id">Previously tracked identity.</param>
    public void ForgetControl(string id)
    {
        CheckOpen();
        ForgetSplitButton(id);
        if (controlKeys.Remove(id, out uint key))
        {
            ForgetConfirmButton(key);
            ForgetStatusButton(key);
            NativeMethods.Forget(context, key);
        }
    }

    /// <summary>Reads finalized geometry from the most recently presented or captured frame.</summary>
    /// <param name="id">A key tracked in that frame.</param>
    /// <returns>The widget rectangle in logical units.</returns>
    public CuiRectangle GetControlRectangle(string id)
    {
        CheckOpen();
        if (inFrame || !hasPresentedFrame)
            throw new InvalidOperationException("Present the current frame before reading its finalized geometry.");
        return NativeMethods.NodeRectangle(context, trackedNodes[id]);
    }
}
