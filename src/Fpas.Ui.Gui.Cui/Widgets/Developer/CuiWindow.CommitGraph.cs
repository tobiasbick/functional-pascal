// [Documentation](../../../../docs/ui/gui/developer-tools/commit-graph.md)
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native CommitGraph widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Commits">Commits supplied to CUI.</param>
    /// <param name="Selection">Model selection, minus one for none, or null for native selection.</param>
    /// <param name="Height">Logical height, zero for the default, or minus one to fill.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <param name="Focus">Whether to request focus on the graph.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult CommitGraph(string Id, CuiDevCommit[] Commits, int? Selection,
        float Height, int WidthKind, float Width, bool Enabled, bool Focus = false)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        if (Height != -1) CheckFeedbackNumber(Height, nameof(Height));
        if (Selection is int row && (row < -1 || row >= Commits.Length))
            throw new ArgumentOutOfRangeException(nameof(Selection));
        for (int index = 0; index < Commits.Length; ++index)
        {
            var commit = Commits[index];
            Utf8.Validate(commit.Hash, nameof(Commits));
            Utf8.Validate(commit.Message, nameof(Commits));
            if (commit.Author is string author) Utf8.Validate(author, nameof(Commits));
            if (commit.Date is string date) Utf8.Validate(date, nameof(Commits));
            if (commit.Refs is string refs) Utf8.Validate(refs, nameof(Commits));
            if (commit.Parent != -1 && (commit.Parent <= index || commit.Parent >= Commits.Length)
                || commit.MergeParent != -1 && (commit.MergeParent <= index || commit.MergeParent >= Commits.Length))
                throw new ArgumentOutOfRangeException(nameof(Commits));
        }
        using var memory = new ChartMemory();
        int flags = (Enabled ? 1 : 0) | (Focus ? 2 : 0);
        NativeMethods.DevCommitGraph(context, Stable(Id), PackCommit(memory, Commits), Commits.Length,
            Selection ?? -2, Height, WidthKind, Width, flags, out var result);
        return result.ToManaged();
    }
}
