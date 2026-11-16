using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Injects a logical pointer position before drawing widgets in the current frame.</summary>
    /// <param name="x">Finite horizontal coordinate.</param>
    /// <param name="y">Finite vertical coordinate.</param>
    public void MovePointer(float x, float y)
    {
        CheckFrame();
        if (!float.IsFinite(x)) throw new ArgumentOutOfRangeException(nameof(x));
        if (!float.IsFinite(y)) throw new ArgumentOutOfRangeException(nameof(y));
        NativeMethods.MovePointer(context, x, y);
    }

    /// <summary>Injects the left pointer button through CUI's input API.</summary>
    /// <param name="down">Whether the button is pressed.</param>
    public void SetLeftButton(bool down) { CheckFrame(); NativeMethods.MouseButton(context, 0, down ? (byte)1 : (byte)0); }

    /// <summary>Injects the right pointer button through CUI's input API.</summary>
    /// <param name="down">Whether the button is pressed.</param>
    public void SetRightButton(bool down) { CheckFrame(); NativeMethods.MouseButton(context, 1, down ? (byte)1 : (byte)0); }

    /// <summary>Reads a native key press during a frame.</summary>
    /// <param name="key">Native key ordinal, tab=1, enter=2, escape=3, space=6, arrows=7..10.</param>
    /// <returns>Whether pressed.</returns>
    public bool KeyPressed(int key) { CheckFrame(); return NativeMethods.KeyPressed(context, key) != 0; }

    /// <summary>Gets native modifier bits during a frame.</summary>
    public int Modifiers { get { CheckFrame(); return NativeMethods.KeyModifiers(context); } }

    /// <summary>Injects a key for a bounded native integration test.</summary>
    /// <param name="key">Native key ordinal.</param>
    /// <param name="down">Pressed state.</param>
    /// <param name="modifiers">Native modifier bits.</param>
    public void SetKey(int key, bool down, int modifiers = 0)
    { CheckFrame(); NativeMethods.InputKey(context, key, down ? (byte)1 : (byte)0, modifiers); }

    /// <summary>Supplies a committed text event to the native input queue.</summary>
    /// <param name="text">Text to deliver to the focused editor in the current frame.</param>
    public void InputText(string text)
    {
        CheckFrame();
        Utf8.Validate(text, nameof(text));
        NativeMethods.InputText(context, text);
    }

    /// <summary>Injects wheel movement for integration tests.</summary>
    /// <param name="delta">Vertical movement in lines.</param>
    public void Wheel(float delta) => Wheel(0, delta);

    /// <summary>Supplies horizontal and vertical wheel input to CUI.</summary>
    /// <param name="horizontal">Horizontal movement in native wheel lines.</param>
    /// <param name="vertical">Vertical movement in native wheel lines.</param>
    public void Wheel(float horizontal, float vertical)
    {
        CheckFrame();
        if (!float.IsFinite(horizontal)) throw new ArgumentOutOfRangeException(nameof(horizontal));
        if (!float.IsFinite(vertical)) throw new ArgumentOutOfRangeException(nameof(vertical));
        NativeMethods.InputScroll(context, horizontal, vertical);
    }
}
