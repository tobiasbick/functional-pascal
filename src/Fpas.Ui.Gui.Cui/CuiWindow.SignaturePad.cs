using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>A detached native signature snapshot.</summary>
/// <param name="Coordinates">Interleaved X and Y coordinates in CUI's logical drawing space.</param>
/// <param name="StrokeEnds">Exclusive end point index of each stroke.</param>
public readonly record struct CuiSignature(float[] Coordinates, int[] StrokeEnds);

public sealed partial class CuiWindow
{
    // CUI computes stroke ends in a native scratch buffer shared by windows.
    private static readonly Lock signaturePointsLock = new();

    /// <summary>Draws CUI's signature pad and copies its strokes on changes.</summary>
    /// <param name="id">Stable pad identity.</param>
    /// <param name="placeholder">Literal instruction, or empty for the native default.</param>
    /// <param name="height">Logical height, or zero for the default.</param>
    /// <param name="rgba">Packed RGBA ink color, or zero for the default.</param>
    /// <param name="enabled">Whether to accept drawing or clearing.</param>
    /// <param name="focusClear">Whether to request focus on the Clear button.</param>
    /// <param name="track">Whether to track the pad bounds.</param>
    /// <param name="readChanges">Whether to copy snapshots of changed strokes.</param>
    /// <param name="clearId">Optional tracking identity for the Clear button.</param>
    /// <param name="clearVisible">Whether the native Clear button is present.</param>
    /// <returns>A detached snapshot on drawing or clearing, or null otherwise.</returns>
    public CuiSignature? SignaturePad(string id, string placeholder, float height, uint rgba, bool enabled,
        bool focusClear, bool track, bool readChanges, string? clearId, out bool clearVisible)
    {
        CheckFrame();
        Utf8.Validate(placeholder, nameof(placeholder));
        if (!float.IsFinite(height) || height < 0) throw new ArgumentOutOfRangeException(nameof(height));
        string stable = Stable(id);
        int changed = NativeMethods.SignaturePad(context, stable, Optional(placeholder), height, rgba,
            enabled ? 1 : 0, focusClear ? 1 : 0, out int node, out int clear);
        clearVisible = clear >= 0;
        if (track)
        {
            TrackControl(id, node);
            if (clearVisible && clearId is not null) TrackControl(clearId, clear);
        }
        if (!readChanges || changed == 0) return null;

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
