namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private bool[] TrackScheduleParts(int[] nodes, Func<int, string>? part)
    {
        var available = new bool[nodes.Length];
        for (int i = 0; i < nodes.Length; ++i)
        {
            available[i] = nodes[i] >= 0;
            if (available[i] && part is not null) TrackControl(part(i), nodes[i]);
        }
        return available;
    }
}
