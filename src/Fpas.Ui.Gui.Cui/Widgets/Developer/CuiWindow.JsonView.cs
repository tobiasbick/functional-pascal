using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native JSON viewer.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="json">Literal JSON or a parse-error example.</param>
    /// <param name="depth">Initial expansion depth.</param>
    /// <param name="height">Maximum viewport height, or zero.</param>
    /// <param name="enabled">Whether folding accepts input.</param>
    /// <returns>Whether native CUI parsed the JSON.</returns>
    public bool JsonView(string id, string json, int depth, float height, bool enabled)
    {
        CheckFrame();
        Utf8.Validate(json, nameof(json));
        CheckFeedbackNumber(height, nameof(height));
        ArgumentOutOfRangeException.ThrowIfNegative(depth);
        return NativeMethods.JsonView(context, Stable(id), json, depth, height, enabled ? 1 : 0) != 0;
    }
}
