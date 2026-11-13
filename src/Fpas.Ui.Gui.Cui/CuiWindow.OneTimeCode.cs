using System.Text;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>A native code edit and CUI's completion event.</summary>
/// <param name="Value">The proposed ASCII code.</param>
/// <param name="Completed">Whether CUI reported a newly completed code this frame.</param>
public readonly record struct CuiOneTimeCodeResult(string Value, bool Completed);

public sealed partial class CuiWindow
{
    /// <summary>Draws a native row of one-time code boxes.</summary>
    /// <param name="id">Stable row identity.</param>
    /// <param name="value">Current ASCII alphanumeric code.</param>
    /// <param name="length">Number of boxes, from 1 to 12.</param>
    /// <param name="enabled">Whether input is accepted.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <param name="track">Whether to track the row in the current frame.</param>
    /// <returns>The proposed code and native completion event.</returns>
    public unsafe CuiOneTimeCodeResult OneTimeCode(string id, string value, int length, bool enabled, bool focus, bool track)
    {
        CheckFrame();
        string stable = Stable(id);
        ArgumentNullException.ThrowIfNull(value);
        if (length is < 1 or > 12) throw new ArgumentOutOfRangeException(nameof(length));
        if (value.Length > length || value.Any(character => !char.IsAsciiLetterOrDigit(character)))
            throw new ArgumentException("Expected an ASCII alphanumeric code that fits the boxes.", nameof(value));
        Span<byte> code = stackalloc byte[length + 1];
        code[Encoding.ASCII.GetBytes(value, code)] = 0;
        try
        {
            int node, complete;
            fixed (byte* bytes = code)
                node = NativeMethods.OneTimeCode(context, stable, bytes, length, enabled ? 1 : 0, focus ? 1 : 0,
                    out complete);
            if (track) TrackControl(id, node);
            return new(enabled ? Encoding.ASCII.GetString(code[..code.IndexOf((byte)0)]) : value, complete != 0);
        }
        finally { code.Clear(); }
    }
}
