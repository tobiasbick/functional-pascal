// [Documentation](../../../docs/ui/gui/capture/README.md)
using System.Runtime.InteropServices;
using System.Text;

namespace Fpas.Ui.Gui.Cui.Tests.Media;

internal static partial class WindowsCaptureInput
{
    private const uint DropFiles = 0x0233;

    [StructLayout(LayoutKind.Sequential)]
    private struct DropHeader
    {
        public int Offset, X, Y, NonClient, Wide;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct Rect
    {
        public int Left, Top, Right, Bottom;
    }

    internal static nint Window(string title)
    {
        nint window = FindWindow(null, title);
        Assert.NotEqual(0, window);
        return window;
    }

    internal static void Drop(nint window, CuiSize logicalSize, CuiRectangle target, params string[] paths)
    {
        Assert.NotEqual(0, GetClientRect(window, out Rect client));
        int headerSize = Marshal.SizeOf<DropHeader>();
        byte[] files = Encoding.Unicode.GetBytes(string.Join('\0', paths) + "\0\0");
        nint handle = GlobalAlloc(0x0042, (nuint)(headerSize + files.Length));
        Assert.NotEqual(0, handle);
        bool sent = false;
        try
        {
            nint data = GlobalLock(handle);
            Assert.NotEqual(0, data);
            try
            {
                Marshal.StructureToPtr(new DropHeader
                {
                    Offset = headerSize,
                    X = (int)((target.X + target.Width / 2) * client.Right / logicalSize.Width),
                    Y = (int)((target.Y + target.Height / 2) * client.Bottom / logicalSize.Height),
                    Wide = 1
                }, data, false);
                Marshal.Copy(files, 0, data + headerSize, files.Length);
            }
            finally { GlobalUnlock(handle); }

            sent = PostMessage(window, DropFiles, (nuint)handle, 0) != 0;
            Assert.True(sent, "Could not post the shell drop to the CUI window.");
        }
        finally
        {
            // The window's DragFinish owns the handle after delivery.
            if (!sent) GlobalFree(handle);
        }
    }

    [LibraryImport("user32", EntryPoint = "FindWindowW", StringMarshalling = StringMarshalling.Utf16)]
    private static partial nint FindWindow(string? className, string title);
    [LibraryImport("user32")]
    private static partial int GetClientRect(nint window, out Rect client);
    [LibraryImport("user32", EntryPoint = "PostMessageW")]
    private static partial int PostMessage(nint window, uint message, nuint wParam, nint lParam);
    [LibraryImport("kernel32")]
    private static partial nint GlobalAlloc(uint flags, nuint size);
    [LibraryImport("kernel32")]
    private static partial nint GlobalLock(nint memory);
    [LibraryImport("kernel32")]
    private static partial int GlobalUnlock(nint memory);
    [LibraryImport("kernel32")]
    private static partial nint GlobalFree(nint memory);
}
