// [Documentation](../../../docs/ui/gui/cui.md#touch-input)
using Fpas.Ui.Gui.Cui.Interop;
using Fpas.Ui.Gui.Cui.Widgets.Canvas;

namespace Fpas.Ui.Gui.Cui;

/// <summary>One finger's touch state in the current frame.</summary>
/// <param name="Down">Whether the finger touches the window.</param>
/// <param name="Pressed">Whether it landed this frame.</param>
/// <param name="Released">Whether it lifted this frame.</param>
/// <param name="Position">Logical position; a finger that lifted this frame keeps its last position.</param>
public readonly record struct CuiTouch(bool Down, bool Pressed, bool Released, CuiPoint? Position);

public sealed partial class CuiWindow
{
    /// <summary>The number of fingers CUI tracks, numbered from zero.</summary>
    public const int TouchSlots = 10;

    /// <summary>Lands or lifts a finger through CUI's raw multi-touch input.</summary>
    /// <param name="finger">Finger slot below <see cref="TouchSlots"/>.</param>
    /// <param name="down">Whether the finger touches the window.</param>
    /// <param name="x">Finite horizontal coordinate.</param>
    /// <param name="y">Finite vertical coordinate.</param>
    public void Touch(int finger, bool down, float x, float y)
    {
        CheckTouchInput(finger, x, y);
        NativeMethods.InputTouch(context, finger, down ? (byte)1 : (byte)0, x, y);
    }

    /// <summary>Moves a finger that touches the window.</summary>
    /// <param name="finger">Finger slot below <see cref="TouchSlots"/>.</param>
    /// <param name="x">Finite horizontal coordinate.</param>
    /// <param name="y">Finite vertical coordinate.</param>
    public void MoveTouch(int finger, float x, float y)
    {
        CheckTouchInput(finger, x, y);
        NativeMethods.InputTouchMove(context, finger, x, y);
    }

    /// <summary>Lifts a finger whose gesture the system took over.</summary>
    /// <param name="finger">Finger slot below <see cref="TouchSlots"/>.</param>
    public void CancelTouch(int finger)
    {
        CheckTouchInput(finger, 0, 0);
        NativeMethods.InputTouchCancel(context, finger);
    }

    /// <summary>Gets the number of fingers touching the window in the current frame.</summary>
    public int TouchCount { get { CheckFrame(); return NativeMethods.TouchCount(context); } }

    /// <summary>Reads one finger's state in the current frame.</summary>
    /// <param name="finger">Finger slot, or minus one for the touch that landed first.</param>
    /// <returns>The finger's state; the position is null while it is unknown.</returns>
    public CuiTouch ReadTouch(int finger = -1)
    {
        CheckFrame();
        if (finger is < -1 or >= TouchSlots) throw new ArgumentOutOfRangeException(nameof(finger));
        bool down = NativeMethods.TouchDown(context, finger) != 0;
        bool released = NativeMethods.TouchReleased(context, finger) != 0;
        CuiPoint position = NativeMethods.TouchPosition(context, finger);
        return new(down, NativeMethods.TouchPressed(context, finger) != 0, released,
            down || released ? position : null);
    }

    private void CheckTouchInput(int finger, float x, float y)
    {
        CheckFrame();
        if (finger is < 0 or >= TouchSlots) throw new ArgumentOutOfRangeException(nameof(finger));
        if (!float.IsFinite(x)) throw new ArgumentOutOfRangeException(nameof(x));
        if (!float.IsFinite(y)) throw new ArgumentOutOfRangeException(nameof(y));
    }
}
