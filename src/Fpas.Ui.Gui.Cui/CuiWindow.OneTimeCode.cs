using System.Text;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native row of one-time code boxes.</summary>
    /// <param name="id">Stable row identity.</param>
    /// <param name="value">Current ASCII alphanumeric code.</param>
    /// <param name="length">Number of boxes, from 1 to 12.</param>
    /// <param name="enabled">Whether input is accepted.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <param name="track">Whether to track the row in the current frame.</param>
    /// <returns>The proposed code after the frame's input.</returns>
    public unsafe string OneTimeCode(string id, string value, int length, bool enabled, bool focus, bool track)
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
            int node;
            fixed (byte* bytes = code)
                node = NativeMethods.OneTimeCode(context, stable, bytes, length, enabled ? 1 : 0, focus ? 1 : 0);
            if (track) TrackControl(id, node);
            return enabled ? Encoding.ASCII.GetString(code[..code.IndexOf((byte)0)]) : value;
        }
        finally { code.Clear(); }
    }
}
