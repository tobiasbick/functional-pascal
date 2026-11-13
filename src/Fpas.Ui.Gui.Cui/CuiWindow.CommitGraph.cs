using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native CommitGraph widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Commits">Commits supplied to CUI.</param>
    /// <param name="Selection">Selection supplied to CUI.</param>
    /// <param name="Height">Height supplied to CUI.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult CommitGraph(string Id, CuiDevCommit[] Commits, int Selection, float Height, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        using var memory = new ChartMemory();
        NativeMethods.DevCommitGraph(context, Stable(Id), PackCommit(memory, Commits), Commits.Length, Selection, Height, WidthKind, Width, Enabled ? 1 : 0, out var result);
        return result.ToManaged();
    }
}
