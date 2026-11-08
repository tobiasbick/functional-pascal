using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.Win32.SafeHandles;

namespace Fpas.Processes.Terminal;

public sealed record TerminalInput(string Kind, string Key = "", int Modifiers = 0, string Text = "",
    int Column = 0, int Row = 0, int Delta = 0, string Button = "left");

public sealed class ConsoleTerminal : IDisposable
{
    private const string EnterScreen = "\x1b[?1049h\x1b[?7l\x1b[?25l\x1b[?1003h\x1b[?1006h\x1b[?2004h";
    private const string LeaveScreen = "\x1b[0m\x1b[?2004l\x1b[?1006l\x1b[?1003l\x1b[?25h\x1b[?7h\x1b[?1049l";
    private readonly Stream? output;
    private readonly byte[]? previousUnixMode;
    private readonly uint previousInputMode, previousOutputMode;
    private readonly nint inputHandle, outputHandle;
    private readonly SafeFileHandle? outputModeHandle;
    private readonly VtInput decoder = new();
    private readonly byte[] bytes = new byte[4096];
    private bool disposed;
    private bool outputFailed;
    private uint pressedButtons;
    private TerminalInput? pendingMouse;
    private char highSurrogate;
    public int Columns { get; private set; }
    public int Rows { get; private set; }

    public ConsoleTerminal()
    {
        if (Console.IsInputRedirected || Console.IsOutputRedirected)
            throw new IOException("Interactive TUI requires a terminal for input and output.");
        if (OperatingSystem.IsWindows())
        {
            inputHandle = GetStdHandle(-10);
            outputHandle = GetStdHandle(-11);
            Check(GetConsoleMode(inputHandle, out previousInputMode));
            Check(GetConsoleMode(outputHandle, out previousOutputMode));
            // Keep a handle to the original screen buffer if stdout becomes unusable.
            var process = GetCurrentProcess();
            Check(DuplicateHandle(process, outputHandle, process, out var modeHandle, 0, false, 2));
            try
            {
                // Preserve bracketed paste framing when reading input records from ConPTY.
                Check(SetConsoleMode(inputHandle, (previousInputMode & ~0x47u) | 0x298u));
                if (!SetConsoleMode(outputHandle, previousOutputMode | 7))
                {
                    var failure = Marshal.GetLastPInvokeError();
                    SetConsoleMode(inputHandle, previousInputMode);
                    throw new Win32Exception(failure);
                }
            }
            catch { modeHandle.Dispose(); throw; }
            outputModeHandle = modeHandle;
        }
        else if (OperatingSystem.IsLinux())
        {
            previousUnixMode = new byte[256];
            Check(TcGetAttr(0, previousUnixMode) == 0);
            var raw = (byte[])previousUnixMode.Clone();
            CfMakeRaw(raw);
            Check(TcSetAttr(0, 0, raw) == 0);
        }
        else throw new PlatformNotSupportedException("Interactive TUI supports Windows and Linux.");

        try
        {
            if (OperatingSystem.IsLinux()) output = Console.OpenStandardOutput();
            _ = ReadSize();
            Write(EnterScreen);
        }
        catch
        {
            try
            {
                if (OperatingSystem.IsWindows() || output is not null) Dispose();
                else Restore();
            }
            catch (IOException) { }
            throw;
        }
    }

    public void Write(string text)
    {
        outputFailed = true;
        try
        {
            if (output is { } stream)
            {
                stream.Write(Encoding.UTF8.GetBytes(text));
                stream.Flush();
            }
            else WriteWindows(text);
        }
        catch (UnauthorizedAccessException failure)
        {
            throw new IOException(failure.Message, failure);
        }
        outputFailed = false;
    }

    private void WriteWindows(string text)
    {
        // Write UTF-16 directly so glyphs and VT sequences reach the screen on any code page.
        for (var offset = 0; offset < text.Length;)
        {
            var count = Math.Min(16_384, text.Length - offset);
            if (offset + count < text.Length && char.IsHighSurrogate(text[offset + count - 1])) count--;
            if (!WriteConsole(outputHandle, text.Substring(offset, count), count, out var written, 0))
                throw new IOException("Terminal output write failed.", new Win32Exception(Marshal.GetLastPInvokeError()));
            if (written <= 0) throw new IOException("Terminal output write made no progress.");
            offset += written;
        }
    }

    public TerminalInput? Poll(int milliseconds)
    {
        if (milliseconds is < 0 or > 1000) throw new ArgumentOutOfRangeException(nameof(milliseconds));
        if (ReadSize()) return new("resize", Column: Columns, Row: Rows);
        if (OperatingSystem.IsWindows()) return PollWindows(milliseconds);

        if (decoder.Take(false) is { } pending) return pending;
        var descriptor = new PollDescriptor { Descriptor = 0, Events = 1 };
        var ready = PollNative(ref descriptor, 1, milliseconds);
        if (ready < 0)
        {
            if (Marshal.GetLastPInvokeError() == 4) return null;
            throw new Win32Exception(Marshal.GetLastPInvokeError());
        }
        if (ready == 0) return decoder.Take(milliseconds >= 20);
        var count = ReadNative(0, bytes, (nuint)bytes.Length);
        if (count <= 0)
        {
            if (count < 0 && Marshal.GetLastPInvokeError() == 4) return null;
            throw new IOException("Terminal input closed.");
        }
        decoder.Append(bytes.AsSpan(0, (int)count));
        return decoder.Take(false);
    }

    private bool ReadSize()
    {
        int width, height;
        if (OperatingSystem.IsWindows()) { width = Console.WindowWidth; height = Console.WindowHeight; }
        else
        {
            Check(GetWindowSize(0, 0x5413, out var size) == 0);
            width = size.Columns; height = size.Rows;
        }
        width = Math.Clamp(width, 1, 500); height = Math.Clamp(height, 1, 300);
        if (width == Columns && height == Rows) return false;
        Columns = width; Rows = height;
        return true;
    }

    private TerminalInput? PollWindows(int milliseconds)
    {
        if (pendingMouse is { } mouse)
        {
            pendingMouse = null;
            return mouse;
        }
        if (decoder.Take(false) is { } pending) return pending;
        var ready = WaitForSingleObject(inputHandle, (uint)milliseconds);
        if (ready == 258) return decoder.Take(milliseconds >= 20);
        Check(ready == 0);
        Check(ReadConsoleInput(inputHandle, out var record, 1, out var count));
        if (count == 0) return null;
        if (record.Type == 4) return ReadSize() ? new("resize", Column: Columns, Row: Rows) : null;
        if (record.Type == 2) return ReadMouse(record);
        if (record.Type != 1 || record.KeyDown == 0) return null;
        if (char.IsHighSurrogate(record.Character)) { highSurrogate = record.Character; return null; }
        var text = highSurrogate != 0 && char.IsLowSurrogate(record.Character)
            ? new string([highSurrogate, record.Character]) : record.Character.ToString();
        highSurrogate = (char)0;
        if (record.Character != 0 && (record.VirtualKey == 0 || record.Character == 27 || decoder.HasPending))
        {
            decoder.Append(Encoding.UTF8.GetBytes(text));
            return decoder.Take(false);
        }
        var modifiers = ((record.KeyState & 0x10) != 0 ? 1 : 0) |
            ((record.KeyState & 3) != 0 ? 2 : 0) | ((record.KeyState & 12) != 0 ? 4 : 0);
        var name = record.VirtualKey switch
        {
            8 => "Backspace", 9 => "Tab", 13 => "Enter", 27 => "Escape", 33 => "PageUp",
            34 => "PageDown", 35 => "End", 36 => "Home", 37 => "Left", 38 => "Up", 39 => "Right",
            40 => "Down", 45 => "Insert", 46 => "Delete", >= 112 and <= 123 => $"F{record.VirtualKey - 111}",
            >= 65 and <= 90 when (modifiers & 6) == 2 ||
                (modifiers & 4) != 0 && char.IsControl(record.Character) => ((char)record.VirtualKey).ToString(),
            _ => null
        };
        if (name is not null) return new("key", name, modifiers);
        if (record.Character == 0 || char.IsControl(record.Character)) return null;
        return new("character", Text: text);
    }

    private TerminalInput? ReadMouse(InputRecord record)
    {
        const uint left = 1, middle = 4;
        if (record.MouseFlags == 4)
            return new("scroll", Column: record.X, Row: record.Y, Delta: -3 * Math.Sign((short)(record.Buttons >> 16)));
        if (record.MouseFlags == 1) return new("move", Column: record.X, Row: record.Y);
        if (record.MouseFlags is not (0 or 2)) return null;

        var changed = (pressedButtons ^ record.Buttons) & (left | middle);
        pressedButtons = record.Buttons;
        if ((changed & middle) != 0) pendingMouse = Button(middle, "middle");
        if ((changed & left) != 0) return Button(left, "left");
        var result = pendingMouse;
        pendingMouse = null;
        return result;

        TerminalInput Button(uint mask, string name) => new((record.Buttons & mask) != 0 ? "press" : "release",
            Column: record.X, Row: record.Y, Button: name);
    }

    public void Dispose()
    {
        if (disposed) return;
        disposed = true;
        var recovering = outputFailed;
        try { Write(LeaveScreen); }
        catch (IOException) when (recovering) { } // Preserve the earlier presentation error.
        finally { Restore(); output?.Dispose(); }
    }

    private void Restore()
    {
        if (outputModeHandle is { } modeHandle)
        {
            SetConsoleMode(inputHandle, previousInputMode);
            SetConsoleMode(modeHandle.DangerousGetHandle(), previousOutputMode);
            modeHandle.Dispose();
        }
        else if (previousUnixMode is not null) TcSetAttr(0, 0, previousUnixMode);
    }

    private static void Check(bool success)
    { if (!success) throw new Win32Exception(Marshal.GetLastPInvokeError()); }

    [StructLayout(LayoutKind.Sequential)] private struct PollDescriptor { internal int Descriptor; internal short Events, Result; }
    [StructLayout(LayoutKind.Sequential)] private struct WindowSize { internal ushort Rows, Columns, X, Y; }
    [StructLayout(LayoutKind.Explicit, Size = 20, CharSet = CharSet.Unicode)]
    private struct InputRecord
    {
        [FieldOffset(0)] internal ushort Type;
        [FieldOffset(4)] internal int KeyDown;
        [FieldOffset(10)] internal ushort VirtualKey;
        [FieldOffset(14)] internal char Character;
        [FieldOffset(16)] internal uint KeyState;
        [FieldOffset(4)] internal short X;
        [FieldOffset(6)] internal short Y;
        [FieldOffset(8)] internal uint Buttons;
        [FieldOffset(16)] internal uint MouseFlags;
    }
    [DllImport("libc", EntryPoint = "tcgetattr", SetLastError = true)] private static extern int TcGetAttr(int fd, [Out] byte[] mode);
    [DllImport("libc", EntryPoint = "tcsetattr", SetLastError = true)] private static extern int TcSetAttr(int fd, int action, byte[] mode);
    [DllImport("libc", EntryPoint = "cfmakeraw")] private static extern void CfMakeRaw([In, Out] byte[] mode);
    [DllImport("libc", EntryPoint = "ioctl", SetLastError = true)] private static extern int GetWindowSize(int fd, nuint operation, out WindowSize size);
    [DllImport("libc", EntryPoint = "poll", SetLastError = true)] private static extern int PollNative(ref PollDescriptor fd, nuint count, int milliseconds);
    [DllImport("libc", EntryPoint = "read", SetLastError = true)] private static extern nint ReadNative(int fd, [Out] byte[] buffer, nuint count);
    [DllImport("kernel32.dll")] private static extern nint GetStdHandle(int handle);
    [DllImport("kernel32.dll")] private static extern nint GetCurrentProcess();
    [DllImport("kernel32.dll", SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool DuplicateHandle(nint sourceProcess, nint source, nint targetProcess, out SafeFileHandle target, uint access, bool inherit, uint options);
    [DllImport("kernel32.dll", SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)] private static extern bool GetConsoleMode(nint console, out uint mode);
    [DllImport("kernel32.dll", SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)] private static extern bool SetConsoleMode(nint console, uint mode);
    [DllImport("kernel32.dll", EntryPoint = "WriteConsoleW", CharSet = CharSet.Unicode, SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool WriteConsole(nint console, string buffer, int length, out int written, nint reserved);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern uint WaitForSingleObject(nint handle, uint milliseconds);
    [DllImport("kernel32.dll", EntryPoint = "ReadConsoleInputW", SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool ReadConsoleInput(nint console, out InputRecord record, uint length, out uint count);
}
