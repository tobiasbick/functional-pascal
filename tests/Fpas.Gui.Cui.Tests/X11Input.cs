using System.Runtime.InteropServices;

namespace Fpas.Gui.Cui.Tests;

/// <summary>Sends real X server events; it never calls the GUI binding's input injection APIs.</summary>
internal sealed class X11Input : IDisposable
{
    private nint display = XOpenDisplay(0);
    private readonly nuint window;

    internal X11Input(string title)
    {
        if (display == 0) throw new InvalidOperationException("X11 display could not be opened.");
        try
        {
            Assert.NotEqual(0, XTestQueryExtension(display, out _, out _, out _, out _));
            window = Find(XDefaultRootWindow(display), title);
            Assert.NotEqual((nuint)0, window);
            XSetInputFocus(display, window, 1, 0);
            XSync(display, false);
        }
        catch { Dispose(); throw; }
    }

    private nuint Find(nuint parent, string title)
    {
        if (XFetchName(display, parent, out nint name) != 0)
        {
            try { if (Marshal.PtrToStringUTF8(name) == title) return parent; }
            finally { XFree(name); }
        }
        if (XQueryTree(display, parent, out _, out _, out nint children, out uint count) == 0) return 0;
        try
        {
            for (int i = 0; i < count; i++)
            {
                nuint found = Find((nuint)Marshal.ReadIntPtr(children, i * nint.Size), title);
                if (found != 0) return found;
            }
        }
        finally { if (children != 0) XFree(children); }
        return 0;
    }

    internal void Move(int x, int y)
    {
        XTranslateCoordinates(display, window, XDefaultRootWindow(display), x, y, out int rootX, out int rootY, out _);
        Assert.NotEqual(0, XTestFakeMotionEvent(display, -1, rootX, rootY, 0));
        XFlush(display);
    }

    internal void Button(bool down)
    {
        Assert.NotEqual(0, XTestFakeButtonEvent(display, 1, down, 0));
        XFlush(display);
    }

    internal void Key(string symbol, bool down)
    {
        uint code = XKeysymToKeycode(display, XStringToKeysym(symbol));
        Assert.NotEqual(0u, code);
        Assert.NotEqual(0, XTestFakeKeyEvent(display, code, down, 0));
        XFlush(display);
    }

    internal void Resize(int width, int height)
    {
        XResizeWindow(display, window, (uint)width, (uint)height);
        XFlush(display);
    }

    internal void Close()
    {
        var message = new ClientMessage
        {
            Type = 33, Display = display, Window = window, SendEvent = 1, Format = 32,
            MessageType = XInternAtom(display, "WM_PROTOCOLS", false),
            Data0 = XInternAtom(display, "WM_DELETE_WINDOW", false)
        };
        Assert.NotEqual(0, XSendEvent(display, window, false, 0, ref message));
        XFlush(display);
    }

    public void Dispose() { if (display != 0) { XCloseDisplay(display); display = 0; } }

    [StructLayout(LayoutKind.Explicit, Size = 192)]
    private struct ClientMessage
    {
        [FieldOffset(0)] internal int Type;
        [FieldOffset(8)] internal nuint Serial;
        [FieldOffset(16)] internal int SendEvent;
        [FieldOffset(24)] internal nint Display;
        [FieldOffset(32)] internal nuint Window;
        [FieldOffset(40)] internal nuint MessageType;
        [FieldOffset(48)] internal int Format;
        [FieldOffset(56)] internal nuint Data0;
    }

    [DllImport("libX11.so.6")] private static extern nint XOpenDisplay(nint name);
    [DllImport("libX11.so.6")] private static extern int XCloseDisplay(nint display);
    [DllImport("libX11.so.6")] private static extern nuint XDefaultRootWindow(nint display);
    [DllImport("libX11.so.6")] private static extern int XFetchName(nint display, nuint window, out nint name);
    [DllImport("libX11.so.6")] private static extern int XFree(nint pointer);
    [DllImport("libX11.so.6")] private static extern int XQueryTree(nint display, nuint window,
        out nuint root, out nuint parent, out nint children, out uint count);
    [DllImport("libX11.so.6")] private static extern int XTranslateCoordinates(nint display, nuint source,
        nuint target, int x, int y, out int translatedX, out int translatedY, out nuint child);
    [DllImport("libX11.so.6")] private static extern int XSetInputFocus(nint display, nuint window, int revert, nuint time);
    [DllImport("libX11.so.6")] private static extern int XSync(nint display, bool discard);
    [DllImport("libX11.so.6")] private static extern int XFlush(nint display);
    [DllImport("libX11.so.6")] private static extern byte XKeysymToKeycode(nint display, nuint symbol);
    [DllImport("libX11.so.6")] private static extern nuint XStringToKeysym(string name);
    [DllImport("libX11.so.6")] private static extern int XResizeWindow(nint display, nuint window, uint width, uint height);
    [DllImport("libX11.so.6")] private static extern nuint XInternAtom(nint display, string name, bool onlyIfExists);
    [DllImport("libX11.so.6")] private static extern int XSendEvent(nint display, nuint window,
        bool propagate, nint mask, ref ClientMessage message);
    [DllImport("libXtst.so.6")] private static extern int XTestQueryExtension(nint display,
        out int eventBase, out int errorBase, out int major, out int minor);
    [DllImport("libXtst.so.6")] private static extern int XTestFakeMotionEvent(nint display,
        int screen, int x, int y, nuint delay);
    [DllImport("libXtst.so.6")] private static extern int XTestFakeButtonEvent(nint display,
        uint button, bool down, nuint delay);
    [DllImport("libXtst.so.6")] private static extern int XTestFakeKeyEvent(nint display,
        uint key, bool down, nuint delay);
}
