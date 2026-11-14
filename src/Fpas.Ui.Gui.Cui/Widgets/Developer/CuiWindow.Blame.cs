using Fpas.Ui.Gui.Cui.Interop.Developer;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native Blame widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Lines">Lines supplied to CUI.</param>
    /// <param name="LineCommits">Line Commits supplied to CUI.</param>
    /// <param name="Revisions">Revisions supplied to CUI.</param>
    /// <param name="Height">Height supplied to CUI.</param>
    /// <param name="Language">Language supplied to CUI.</param>
    /// <param name="Now">Now supplied to CUI.</param>
    /// <param name="FirstLine">First Line supplied to CUI.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult Blame(string Id, string[] Lines, int[] LineCommits, CuiDevRevision[] Revisions, float Height, string Language, long Now, int FirstLine, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        using var memory = new ChartMemory();
        NativeDevResult result;
        fixed (int* line_commitsPtr = LineCommits)
            NativeMethods.DevBlame(context, Stable(Id), memory.Labels(Lines), Lines.Length, line_commitsPtr, LineCommits.Length, PackRevision(memory, Revisions), Revisions.Length, Height, OptionalAi(Language), Now, FirstLine, WidthKind, Width, Enabled ? 1 : 0, out result);
        return result.ToManaged();
    }
}
