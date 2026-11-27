// [Documentation](../../../../docs/ui/gui/more-developer-tools/regex-tester.md)
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's native RegexTester widget.</summary>
    /// <param name="Id">Id supplied to the widget.</param>
    /// <param name="Pattern">Pattern supplied to the widget.</param>
    /// <param name="Sample">Sample supplied to the widget.</param>
    /// <param name="Modifiers">Match flags, or null for native flags.</param>
    /// <param name="PatternCapacity">Pattern Capacity supplied to the widget.</param>
    /// <param name="TextCapacity">Text Capacity supplied to the widget.</param>
    /// <param name="Height">Height supplied to the widget.</param>
    /// <param name="WidthKind">Width Kind supplied to the widget.</param>
    /// <param name="Width">Width supplied to the widget.</param>
    /// <param name="Enabled">Enabled supplied to the widget.</param>
    /// <returns>Native edit or action proposals.</returns>
    public unsafe CuiRegexProposal RegexTester(string Id, string Pattern, string Sample, int? Modifiers, int PatternCapacity, int TextCapacity, float Height, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        if (Modifiers is < 0 or > 7) throw new ArgumentOutOfRangeException(nameof(Modifiers));
        byte[] pattern = DevBuffer(Pattern, PatternCapacity);
        byte[] sample = DevBuffer(Sample, TextCapacity);
        fixed (byte* p = pattern, s = sample)
        {
            nint error = NativeMethods.DevRegexTester(context, Stable(Id), p, (nuint)pattern.Length, s, (nuint)sample.Length,
                Modifiers ?? -1, Height, WidthKind, Width, Enabled ? 1 : 0, out var result);
            return new(Enabled ? ReadDevText(pattern) : Pattern, Enabled ? ReadDevText(sample) : Sample,
                result.Index, result.Flags != 0, result.Count, Marshal.PtrToStringUTF8(error), Enabled && result.Action != 0);
        }
    }
}
