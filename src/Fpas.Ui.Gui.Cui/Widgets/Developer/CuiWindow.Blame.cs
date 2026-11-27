// [Documentation](../../../../docs/ui/gui/more-developer-tools/blame.md)
using Fpas.Ui.Gui.Cui.Interop.Developer;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native Blame widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Lines">Source lines in display order.</param>
    /// <param name="LineCommits">One revision index or minus one per line, or empty for no attribution.</param>
    /// <param name="Revisions">Revisions addressed by line attribution indices.</param>
    /// <param name="Height">Logical height, zero for 360, or negative to fill the parent.</param>
    /// <param name="Language">Syntax language for the source lines.</param>
    /// <param name="Now">Reference time in Unix seconds, or zero for the system clock.</param>
    /// <param name="FirstLine">First displayed source line number, or zero to begin at one.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <returns>The clicked and hovered revision indices, with minus one for absence.</returns>
    public unsafe CuiDevResult Blame(string Id, string[] Lines, int[] LineCommits, CuiDevRevision[] Revisions, float Height, string Language, long Now, int FirstLine, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        if (!float.IsFinite(Height)) throw new ArgumentOutOfRangeException(nameof(Height));
        if (FirstLine > int.MaxValue - Math.Max(0, Lines.Length - 1))
            throw new ArgumentOutOfRangeException(nameof(FirstLine));
        if (LineCommits.Length != 0 && LineCommits.Length != Lines.Length)
            throw new ArgumentException("Supply one attribution index per source line, or an empty array.", nameof(LineCommits));
        foreach (var index in LineCommits)
            if (index < -1 || index >= Revisions.Length) throw new ArgumentOutOfRangeException(nameof(LineCommits));
        Utf8.Validate(Language, nameof(Language));
        foreach (var line in Lines) Utf8.Validate(line, nameof(Lines));
        foreach (var revision in Revisions)
        {
            Utf8.Validate(revision.Hash, nameof(Revisions));
            Utf8.Validate(revision.Author, nameof(Revisions));
            if (revision.Summary is { } summary) Utf8.Validate(summary, nameof(Revisions));
        }
        using var memory = new ChartMemory();
        NativeDevResult result;
        fixed (int* line_commitsPtr = LineCommits)
            NativeMethods.DevBlame(context, Stable(Id), memory.Labels(Lines), Lines.Length, line_commitsPtr, LineCommits.Length, PackRevision(memory, Revisions), Revisions.Length, Height, OptionalAi(Language), Now, FirstLine, WidthKind, Width, Enabled ? 1 : 0, out result);
        return result.ToManaged();
    }
}
