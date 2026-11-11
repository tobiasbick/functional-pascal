using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws native progress through an ordered sequence of steps.</summary>
    /// <param name="id">Stable stepper identity.</param>
    /// <param name="steps">Nonempty sequence of literal captions.</param>
    /// <param name="current">Zero-based step in progress, or the count when complete.</param>
    public void Stepper(string id, string[] steps, int current)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(steps);
        if (steps.Length == 0) throw new ArgumentException("Supply at least one step.", nameof(steps));
        ArgumentOutOfRangeException.ThrowIfNegative(current);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(current, steps.Length);
        foreach (string step in steps)
        {
            Utf8.Validate(step, nameof(steps));
            ArgumentException.ThrowIfNullOrWhiteSpace(step, nameof(steps));
        }
        NativeMethods.Stepper(context, Stable(id), steps, steps.Length, current);
    }
}
