using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native metric tile with an optional trend and sparkline.</summary>
    /// <param name="label">Literal caption.</param>
    /// <param name="value">Formatted reading.</param>
    /// <param name="delta">Percentage change.</param>
    /// <param name="deltaLabel">Optional override for the percentage caption.</param>
    /// <param name="spark">Finite samples; two or more draw a line.</param>
    /// <param name="icon">Optional CUI glyph.</param>
    /// <param name="widthKind">Native bridge width kind.</param>
    /// <param name="width">Logical width or percentage.</param>
    /// <param name="lowerIsBetter">Whether a decrease is favorable.</param>
    public unsafe void Metric(string label, string value, float delta, string deltaLabel,
        float[] spark, string icon, int widthKind, float width, bool lowerIsBetter)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(value, nameof(value));
        Utf8.Validate(deltaLabel, nameof(deltaLabel));
        Utf8.Validate(icon, nameof(icon));
        ArgumentNullException.ThrowIfNull(spark);
        if (!float.IsFinite(delta)) throw new ArgumentOutOfRangeException(nameof(delta));
        foreach (float sample in spark)
            if (!float.IsFinite(sample)) throw new ArgumentOutOfRangeException(nameof(spark));
        fixed (float* samples = spark)
            NativeMethods.Metric(context, label, value, delta, Optional(deltaLabel), samples,
                spark.Length, Optional(icon), widthKind, width, lowerIsBetter ? 1 : 0);
    }
}
