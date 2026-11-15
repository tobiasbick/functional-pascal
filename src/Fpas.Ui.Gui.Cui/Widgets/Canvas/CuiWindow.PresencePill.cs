using Fpas.Ui.Gui.Cui.Widgets.Canvas;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the collaborators, using image avatars when available.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="people">Collaborators in display order.</param>
    /// <param name="size">Avatar diameter, or zero for the default.</param>
    /// <param name="maxVisible">Maximum visible avatars, or zero for the default.</param>
    public unsafe void PresencePill(string id, CuiLivePerson[] people, float size, int maxVisible)
    {
        CheckFrame();
        using var memory = new ChartMemory();
        var paths = new string[people.Length];
        for (int i = 0; i < paths.Length; i++)
        {
            paths[i] = people[i].Avatar;
            Utf8.Validate(paths[i], nameof(people));
        }
        NativeMethods.PresencePill(context, Stable(id), PackPeople(memory, people), memory.Labels(paths),
            people.Length, size, maxVisible);
    }
}
