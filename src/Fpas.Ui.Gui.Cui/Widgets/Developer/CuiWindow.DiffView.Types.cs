// [Documentation](../../../../docs/ui/gui/code/diff-viewer.md)
using Fpas.Ui.Gui.Cui.Interop;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Widgets.Developer;

/// <summary>The native kind of a displayed diff row.</summary>
public enum CuiDiffRowKind
{
    /// <summary>A shared unchanged line.</summary>
    Same,
    /// <summary>A line removed from the old text.</summary>
    Removed,
    /// <summary>A line added to the new text.</summary>
    Added,
    /// <summary>A pair of lines in the split view.</summary>
    Changed,
    /// <summary>A collapsed range of unchanged lines.</summary>
    Fold
}

/// <summary>A detached row after the view has applied its layout and folds.</summary>
/// <param name="Kind">The row's kind.</param>
/// <param name="OldLine">Zero-based line in the old text, or minus one.</param>
/// <param name="NewLine">Zero-based line in the new text, or minus one.</param>
/// <param name="Change">Zero-based change index, or minus one.</param>
/// <param name="Hidden">Number of lines hidden by a fold.</param>
[StructLayout(LayoutKind.Sequential)]
public readonly record struct CuiDiffRow(CuiDiffRowKind Kind, int OldLine, int NewLine, int Change, int Hidden);

/// <summary>A page of displayed diff rows.</summary>
/// <param name="Total">Total displayed row count, including fold markers.</param>
/// <param name="Rows">Requested rows in display order.</param>
public sealed record CuiDiffRows(int Total, CuiDiffRow[] Rows);

/// <summary>Native diff selection and layout.</summary>
/// <param name="Selected">Selected change, or minus one.</param>
/// <param name="Count">Number of changes.</param>
/// <param name="Inline">Whether the layout is inline.</param>
/// <param name="Added">Number of added lines.</param>
/// <param name="Removed">Number of removed lines.</param>
/// <param name="Folds">Number of folded rows.</param>
/// <param name="Hidden">Number of unchanged lines hidden by the folds.</param>
/// <param name="Approximate">Whether the shortest-edit search reached its limit.</param>
/// <param name="CurrentChanged">Whether the selection changed in this frame.</param>
/// <param name="Recomputed">Whether CUI recomputed the diff in this frame.</param>
public sealed record CuiDiffViewEvent(int Selected, int Count, bool Inline, int Added, int Removed, int Folds, int Hidden,
    bool Approximate, bool CurrentChanged, bool Recomputed);
