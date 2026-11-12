using System.Runtime.InteropServices;
using System.Text;

namespace Fpas.Ui.Gui.Cui.Tests;

internal static partial class WindowsCaptureInput
{
    private const uint DropFiles = 0x0233;
    private const uint SetText = 0x000C;
    private const uint ClickButton = 0x00F5;
    private const uint Close = 0x0010;

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

    internal static void RespondToDialog(nint owner, string? path)
    {
        nint dialog = 0;
        bool closed = false;
        try
        {
            for (int attempt = 0; attempt < 200 && dialog == 0; ++attempt)
            {
                nint candidate = OwnedDialog(owner);
                if (IsWindowVisible(candidate) != 0) dialog = candidate;
                else Thread.Sleep(25);
            }
            Assert.NotEqual(0, dialog);
            if (path is not null)
            {
                // Explorer-style common dialogs put the file name in a nested edit control.
                nint fileName = Descendant(Descendant(dialog, "ComboBoxEx32", 0x047C), "Edit");
                if (fileName == 0) fileName = Descendant(dialog, "Edit", 0x0480);
                Assert.NotEqual(0, fileName);
                Assert.NotEqual(0, SendMessageTimeout(fileName, SetText, 0, path, 2, 1500, out _));
            }
            nint button = GetDlgItem(dialog, path is null ? 2 : 1);
            Assert.NotEqual(0, button);
            Assert.NotEqual(0, PostMessage(button, ClickButton, 0, 0));
            for (int attempt = 0; attempt < 120 && IsWindow(dialog) != 0; ++attempt)
                Thread.Sleep(25);
            closed = IsWindow(dialog) == 0;
            Assert.True(closed, "The native file dialog did not close after the selection.");
        }
        finally
        {
            if (!closed)
            {
                if (dialog == 0) dialog = OwnedDialog(owner);
                if (dialog != 0)
                {
                    nint error = OwnedDialog(dialog);
                    if (error != 0) PostMessage(error, Close, 0, 0);
                    PostMessage(dialog, Close, 0, 0);
                }
            }
        }
    }

    private static nint Descendant(nint parent, string className, int controlId = -1)
    {
        if (parent == 0) return 0;
        nint child = 0;
        while ((child = FindWindowEx(parent, child, null, null)) != 0)
        {
            if (controlId == -1 || GetDlgCtrlID(child) == controlId)
            {
                nint match = 0;
                while ((match = FindWindowEx(parent, match, className, null)) != 0)
                    if (match == child && IsWindowVisible(match) != 0) return child;
            }
            nint nested = Descendant(child, className, controlId);
            if (nested != 0) return nested;
        }
        return 0;
    }

    private static nint OwnedDialog(nint owner)
    {
        nint candidate = 0;
        while ((candidate = FindWindowEx(0, candidate, "#32770", null)) != 0)
            if (GetWindow(candidate, 4) == owner) return candidate;
        return 0;
    }

    [LibraryImport("user32", EntryPoint = "FindWindowW", StringMarshalling = StringMarshalling.Utf16)]
    private static partial nint FindWindow(string? className, string title);
    [LibraryImport("user32", EntryPoint = "FindWindowExW", StringMarshalling = StringMarshalling.Utf16)]
    private static partial nint FindWindowEx(nint parent, nint after, string? className, string? title);
    [LibraryImport("user32")]
    private static partial nint GetWindow(nint window, uint command);
    [LibraryImport("user32")]
    private static partial nint GetDlgItem(nint dialog, int id);
    [LibraryImport("user32")]
    private static partial int GetDlgCtrlID(nint control);
    [LibraryImport("user32")]
    private static partial int GetClientRect(nint window, out Rect client);
    [LibraryImport("user32")]
    private static partial int IsWindow(nint window);
    [LibraryImport("user32")]
    private static partial int IsWindowVisible(nint window);
    [LibraryImport("user32", EntryPoint = "PostMessageW")]
    private static partial int PostMessage(nint window, uint message, nuint wParam, nint lParam);
    [LibraryImport("user32", EntryPoint = "SendMessageTimeoutW", StringMarshalling = StringMarshalling.Utf16)]
    private static partial nint SendMessageTimeout(nint window, uint message, nuint wParam,
        string text, uint flags, uint timeout, out nuint result);
    [LibraryImport("kernel32")]
    private static partial nint GlobalAlloc(uint flags, nuint size);
    [LibraryImport("kernel32")]
    private static partial nint GlobalLock(nint memory);
    [LibraryImport("kernel32")]
    private static partial int GlobalUnlock(nint memory);
    [LibraryImport("kernel32")]
    private static partial nint GlobalFree(nint memory);
}
