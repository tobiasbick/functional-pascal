// [Documentation](../../../../docs/ui/gui/capture/signature-pad.md)
using Fpas.Ui.Gui.Cui.Widgets.Files;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    // CUI computes stroke ends in a native scratch buffer shared by windows.
    private static readonly Lock signaturePointsLock = new();

    /// <summary>Draws CUI's signature pad and copies its strokes on changes.</summary>
    /// <param name="id">Stable pad identity.</param>
    /// <param name="placeholder">Literal instruction, or null for the native default.</param>
    /// <param name="widthKind">Native width extent kind.</param>
    /// <param name="width">Fixed logical width or percentage.</param>
    /// <param name="height">Logical height. Nonpositive uses the default.</param>
    /// <param name="ink">Automatic, theme color, or explicit ink.</param>
    /// <param name="enabled">Whether to accept drawing or clearing.</param>
    /// <param name="focusClear">Whether to request focus on the Clear button.</param>
    /// <param name="track">Whether to track the pad bounds.</param>
    /// <param name="readChanges">Whether to copy snapshots of changed strokes.</param>
    /// <param name="clearId">Optional tracking identity for the Clear button.</param>
    /// <param name="clearVisible">Whether the native Clear button is present.</param>
    /// <returns>A detached snapshot on drawing or clearing, or null otherwise.</returns>
    public CuiSignature? SignaturePad(string id, string? placeholder, int widthKind, float width, float height,
        CuiChartColor ink, bool enabled, bool focusClear, bool track, bool readChanges, string? clearId,
        out bool clearVisible)
    {
        CheckFrame();
        if (placeholder is not null) Utf8.Validate(placeholder, nameof(placeholder));
        CheckExtent(widthKind, width, nameof(width));
        if (!float.IsFinite(height)) throw new ArgumentOutOfRangeException(nameof(height));
        if (!ink.IsValid) throw new ArgumentOutOfRangeException(nameof(ink));
        string stable = Stable(id);
        int changed = NativeMethods.SignaturePad(context, stable, placeholder, widthKind, width, height,
            ink.Palette, ink.Rgba,
            enabled ? 1 : 0, focusClear ? 1 : 0, out int node, out int clear);
        clearVisible = clear >= 0;
        if (track) TrackControl(id, node);
        if (clearVisible && clearId is not null) TrackControl(clearId, clear);
        return readChanges && changed != 0 ? ReadSignaturePoints(stable) : null;
    }

    /// <summary>Copies the current signature within the pad's drawing scope.</summary>
    /// <param name="id">Stable pad identity.</param>
    /// <returns>Detached points and stroke boundaries.</returns>
    public CuiSignature ReadSignature(string id)
    {
        CheckFrame();
        return ReadSignaturePoints(Stable(id));
    }

    /// <summary>Clears a signature and cancels any in-progress stroke before drawing the pad.</summary>
    /// <param name="id">Stable pad identity in the current drawing scope.</param>
    public void ClearSignature(string id)
    {
        CheckFrame();
        NativeMethods.SignatureClear(context, Stable(id));
    }

    private CuiSignature ReadSignaturePoints(string stable)
    {
        lock (signaturePointsLock)
        {
            int pointCount = NativeMethods.SignaturePoints(context, stable, out nint points,
                out nint strokeEnds, out int strokeCount);
            if (pointCount == 0) return new CuiSignature([], []);
            var coordinates = new float[pointCount * 2];
            var ends = new int[strokeCount];
            Marshal.Copy(points, coordinates, 0, coordinates.Length);
            Marshal.Copy(strokeEnds, ends, 0, ends.Length);
            return new CuiSignature(coordinates, ends);
        }
    }
}
