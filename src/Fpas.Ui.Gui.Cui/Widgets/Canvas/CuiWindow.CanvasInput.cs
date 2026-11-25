// [Documentation](../../../../docs/ui/gui/text-and-display/canvas.md)
using System.Runtime.CompilerServices;
using System.Runtime.ExceptionServices;
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;
using Fpas.Ui.Gui.Cui.Widgets.Canvas;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private const int PointerButtonEvent = 1;
    private const int LeftButton = 0;
    private readonly record struct CapturedCanvasInput(CuiCanvasInput Input, CuiPoint? Press = null);

    private readonly Dictionary<uint, Action<CuiCanvasInput>> canvasHandlers = [];
    private readonly List<(uint Target, CuiCanvasInput Input, bool NeedsFocus)> routedCanvasInput = [];
    private readonly List<CapturedCanvasInput> capturedCanvasInput = [];
    private ExceptionDispatchInfo? captureFailure;
    private bool listenForCanvasInput;
    private bool deliveringCanvasInput;
    private bool pendingPointerFocus;

    /// <summary>Restricts native Tab traversal to the active pane. Nest scopes inside the layout they cover.</summary>
    /// <param name="id">Stable pane ID.</param><param name="active">Whether its descendants participate in Tab traversal.</param>
    public void PushFocusScope(string id, bool active)
    {
        CheckFrame();
        NativeMethods.PushFocusScope(context, NativeMethods.MakeCanvasId(context, Stable(id)), active ? (byte)1 : (byte)0);
        layouts.Add(new(Layout.FocusScope));
    }

    /// <summary>Ends the innermost native focus scope.</summary>
    public void PopFocusScope()
    {
        RequireLayout(Layout.FocusScope);
        NativeMethods.PopFocusScope(context);
        layouts.RemoveAt(layouts.Count - 1);
    }

    // CUI pumps OS events before beginning the next frame. Copy borrowed text in
    // the hook, then replay in order against the previous frame's focus list.
    private unsafe int PumpCanvasInput()
    {
        if (!listenForCanvasInput) return NativeMethods.Pump(app, DeferClose ? 1 : 0);
        CheckCanvasEventLayout();
        var root = GCHandle.Alloc(this);
        int result;
        try
        {
            NativeMethods.CanvasHook(context, &CaptureCanvasInput, GCHandle.ToIntPtr(root));
            result = NativeMethods.Pump(app, DeferClose ? 1 : 0);
        }
        finally
        {
            NativeMethods.CanvasHook(context, null, 0);
            root.Free();
        }
        try
        {
            captureFailure?.Throw();
            if (result != 0)
                foreach (var captured in capturedCanvasInput)
                    if (captured.Press is { } press) RouteCanvasPointerFocus(press);
                    else RouteCanvasInput(captured.Input);
        }
        finally { capturedCanvasInput.Clear(); captureFailure = null; }
        return result;
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static unsafe byte CaptureCanvasInput(nint ui, NativeMethods.CanvasEvent* input, nint user)
    {
        var kind = (CuiCanvasInputKind)input->Kind;
        bool leftPress = input->Kind == PointerButtonEvent && input->Button == LeftButton && input->Down != 0 &&
            NativeMethods.CanvasButtonDown(ui, LeftButton) == 0;
        if (!leftPress && kind is not (CuiCanvasInputKind.Key or CuiCanvasInputKind.Text or CuiCanvasInputKind.WindowFocus)) return 0;
        var window = (CuiWindow)GCHandle.FromIntPtr(user).Target!;
        try
        {
            if (leftPress)
                window.capturedCanvasInput.Add(new(default, NativeMethods.CanvasPointer(ui)));
            else
            {
                string text = kind == CuiCanvasInputKind.Text ? Marshal.PtrToStringUTF8(input->Text) ?? "" : "";
                bool down = kind == CuiCanvasInputKind.WindowFocus ? input->Focused != 0 : input->Down != 0;
                window.capturedCanvasInput.Add(new(new(kind, input->Key, input->Mods, down, false, text)));
            }
        }
        catch (Exception error) { window.captureFailure ??= ExceptionDispatchInfo.Capture(error); }
        return leftPress ? (byte)0 : (byte)1;
    }

    private static bool canvasEventLayoutChecked;

    // The hook reads CUI's event in place, so a changed cui_input_event must fail before delivery.
    private static void CheckCanvasEventLayout()
    {
        if (canvasEventLayoutChecked) return;
        NativeMethods.CanvasEventLayout(out int size, out int finger, out int down, out int text);
        if (size != Marshal.SizeOf<NativeMethods.CanvasEvent>()
            || finger != (int)Marshal.OffsetOf<NativeMethods.CanvasEvent>(nameof(NativeMethods.CanvasEvent.Finger))
            || down != (int)Marshal.OffsetOf<NativeMethods.CanvasEvent>(nameof(NativeMethods.CanvasEvent.Down))
            || text != (int)Marshal.OffsetOf<NativeMethods.CanvasEvent>(nameof(NativeMethods.CanvasEvent.Text)))
            throw new InvalidOperationException("The managed copy of cui_input_event does not match the native layout.");
        canvasEventLayoutChecked = true;
    }

    // Nonfocusable hits decide during submission whether they retain focus.
    private void RouteCanvasPointerFocus(CuiPoint press) =>
        pendingPointerFocus = NativeMethods.CanvasPointerFocus(context, press.X, press.Y) != 0;

    private void RouteCanvasInput(CuiCanvasInput input)
    {
        uint id = NativeMethods.CanvasDispatch(context, (int)input.Kind, input.Key, input.Modifiers,
            input.Down ? 1 : 0, input.Text, out int repeated);
        if (id != 0) routedCanvasInput.Add((id, input with { Repeated = repeated != 0 }, pendingPointerFocus));
    }

    private void DiscardCanvasInput()
    {
        listenForCanvasInput = false;
        pendingPointerFocus = false;
        routedCanvasInput.Clear();
        canvasHandlers.Clear();
    }

    // Only the newly submitted view can supply a current MVU mapper. Events for
    // a component that disappeared or became disabled end with this frame.
    private void FinishCanvasInput()
    {
        if (renderingResize) return;
        listenForCanvasInput = canvasHandlers.Count != 0;
        try
        {
            deliveringCanvasInput = true;
            foreach (var (target, input, needsFocus) in routedCanvasInput)
                if ((!needsFocus || NativeMethods.CanvasFocusedId(context) != 0) &&
                    canvasHandlers.TryGetValue(target, out var handler)) handler(input);
        }
        finally
        {
            deliveringCanvasInput = false;
            pendingPointerFocus = false;
            routedCanvasInput.Clear();
            canvasHandlers.Clear();
        }
    }
}
