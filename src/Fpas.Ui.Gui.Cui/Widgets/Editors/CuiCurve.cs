// [Documentation](../../../../docs/ui/gui/editing-tools/curve-editor.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Editors;

/// <summary>Evaluates the same interpolation as the native curve editor.</summary>
public static class CuiCurve
{
    /// <summary>Samples an ordered curve without creating a GUI window.</summary>
    /// <param name="keys">Keys with finite coordinates and handles, ordered by X.</param>
    /// <param name="x">The horizontal coordinate. Values outside the curve use the nearest endpoint.</param>
    /// <returns>The interpolated value. Empty keys return zero. NaN samples the first key.</returns>
    public static unsafe float Evaluate(CuiCurveKey[] keys, float x)
    {
        ArgumentNullException.ThrowIfNull(keys);
        float previous = float.NegativeInfinity;
        foreach (CuiCurveKey key in keys)
        {
            if (!float.IsFinite(key.X) || !float.IsFinite(key.Y) || key.X < previous ||
                !float.IsFinite(key.InX) || !float.IsFinite(key.InY) ||
                !float.IsFinite(key.OutX) || !float.IsFinite(key.OutY) || key.Mode > 3)
                throw new ArgumentException("Curve keys must be finite, ordered by X, and use a valid mode.", nameof(keys));
            previous = key.X;
        }
        fixed (CuiCurveKey* data = keys) return NativeMethods.EvaluateCurve(data, keys.Length, x);
    }
}
