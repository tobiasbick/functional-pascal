using System.Text;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static class Utf8
{
    internal static readonly UTF8Encoding Encoding = new(false, true);

    internal static void Validate(string value, string parameter)
    {
        ArgumentNullException.ThrowIfNull(value, parameter);
        if (value.Contains('\0')) throw new ArgumentException("Embedded zero characters are not supported.", parameter);
        Encoding.GetByteCount(value);
    }
}
