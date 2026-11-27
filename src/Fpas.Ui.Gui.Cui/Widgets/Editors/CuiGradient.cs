// [Documentation](../../../../docs/ui/gui/editing-tools/gradient-editor.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Editors;

/// <summary>Samples RGBA with CUI's native sRGB gradient interpolation.</summary>
public static class CuiGradient
{
    /// <summary>Samples a ramp without creating a GUI window.</summary>
    /// <param name="stops">Stops with finite positions from zero through one. Any order is accepted.</param>
    /// <param name="position">Sample position. Values beyond the stops use the nearest end color.</param>
    /// <returns>Packed straight RGBA. Empty stops return transparent black. NaN samples the first stop.</returns>
    public static unsafe uint Evaluate(CuiGradientStop[] stops, float position)
    {
        ArgumentNullException.ThrowIfNull(stops);
        foreach (CuiGradientStop stop in stops)
            if (!float.IsFinite(stop.Position) || stop.Position is < 0 or > 1)
                throw new ArgumentException("Gradient stops must be within zero and one.", nameof(stops));
        fixed (CuiGradientStop* data = stops) return NativeMethods.EvaluateGradient(data, stops.Length, position);
    }
}
