using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Invokes the native PresencePill operation.</summary>
    /// <param name="id">id supplied to the native operation.</param>
    /// <param name="people">people supplied to the native operation.</param>
    /// <param name="size">size supplied to the native operation.</param>
    /// <param name="maxVisible">maxVisible supplied to the native operation.</param>
    public unsafe void PresencePill(string id, CuiLivePerson[] people, float size, int maxVisible)
    {
        CheckFrame();
        using var memory = new ChartMemory();
        NativeMethods.PresencePill(context, Stable(id), PackPeople(memory, people), people.Length, size, maxVisible);
    }
}
