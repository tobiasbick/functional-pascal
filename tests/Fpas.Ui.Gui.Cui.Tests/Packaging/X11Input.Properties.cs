using System.Runtime.InteropServices;
using System.Text;

namespace Fpas.Ui.Gui.Cui.Tests.Packaging;

internal sealed partial class X11Input
{
    internal string TextProperty(string name) => Encoding.UTF8.GetString(
        Property(name, 8).Select(value => (byte)value).ToArray());

    internal long[] IntegerProperty(string name) => Property(name, 32);

    private long[] Property(string name, int format)
    {
        nuint atom = XInternAtom(display, name, false);
        Assert.Equal(0, XGetWindowProperty(display, window, atom, 0, 4096, false, 0,
            out _, out int actualFormat, out nuint count, out _, out nint data));
        try
        {
            Assert.Equal(format, actualFormat);
            var values = new long[checked((int)count)];
            for (int i = 0; i < values.Length; i++)
                values[i] = format == 8 ? Marshal.ReadByte(data, i) : Marshal.ReadIntPtr(data, i * nint.Size).ToInt64();
            return values;
        }
        finally { if (data != 0) XFree(data); }
    }

    [DllImport("libX11.so.6")]
    private static extern int XGetWindowProperty(nint display, nuint window, nuint property,
        nint offset, nint length, bool delete, nuint requestedType, out nuint actualType,
        out int format, out nuint count, out nuint remaining, out nint data);
}
