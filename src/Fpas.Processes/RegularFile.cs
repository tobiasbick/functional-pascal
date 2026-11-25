// [Documentation](../../docs/pascal/libraries/processes.md#reading-files-for-tool-input)
using System.ComponentModel;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

namespace Fpas.Processes;

/// <summary>Opens filesystem input without waiting for a pipe producer on Linux.</summary>
public static class RegularFile
{
    private const int NonBlocking = 0x800, CloseOnExec = 0x80000, EmptyPath = 0x1000;
    private const int LockShared = 1, LockNoWait = 4, WouldBlock = 11;
    private const uint StatusType = 1, DiskFile = 1;
    private const ushort FileKind = 0xf000, RegularKind = 0x8000;
    private static readonly bool DisableFileLocking = AppContext.TryGetSwitch("System.IO.DisableFileLocking", out var disabled)
        ? disabled
        : Environment.GetEnvironmentVariable("DOTNET_SYSTEM_IO_DISABLEFILELOCKING") is { } value &&
            (value == "1" || value.Equals("true", StringComparison.OrdinalIgnoreCase));

    /// <summary>Opens a read-only stream and verifies the opened file type on Linux and Windows.</summary>
    /// <param name="path">A file path, which may include a symbolic link to a regular file.</param>
    /// <param name="shareDelete">Allow concurrent deletion on platforms with mandatory file sharing.</param>
    /// <returns>The stream. The caller owns its lifetime.</returns>
    /// <exception cref="IOException">The file could not be opened or is not a regular file.</exception>
    public static FileStream OpenRead(string path, bool shareDelete = false)
    {
        var fullPath = Path.GetFullPath(path);
        var share = FileShare.Read | (shareDelete ? FileShare.Delete : FileShare.None);
        var handle = OperatingSystem.IsLinux() ? OpenLinux(fullPath) : File.OpenHandle(fullPath, FileMode.Open, FileAccess.Read, share);
        try
        {
            if (OperatingSystem.IsLinux() && !IsRegularLinux(handle, fullPath) ||
                OperatingSystem.IsWindows() && GetFileType(handle) != DiskFile)
                throw new IOException("Expected a regular input file.");
            if (OperatingSystem.IsLinux() && !DisableFileLocking && Flock(handle.DangerousGetHandle().ToInt32(), LockShared | LockNoWait) != 0 &&
                Marshal.GetLastPInvokeError() == WouldBlock)
                throw new IOException($"The file '{fullPath}' is being used by another process.");
            return new FileStream(handle, FileAccess.Read);
        }
        catch
        {
            handle.Dispose();
            throw;
        }
    }

    private static SafeFileHandle OpenLinux(string path)
    {
        var descriptor = Open(path, NonBlocking | CloseOnExec);
        if (descriptor >= 0) return new SafeFileHandle(descriptor, ownsHandle: true);
        var error = Marshal.GetLastPInvokeError();
        var message = ErrorMessage(path, error);
        throw error switch
        {
            2 => new FileNotFoundException(message, path),
            20 => new DirectoryNotFoundException(message),
            1 or 13 => new UnauthorizedAccessException(message),
            36 => new PathTooLongException(message),
            _ => new IOException(message)
        };
    }

    private static bool IsRegularLinux(SafeFileHandle handle, string path)
    {
        if (Statx(handle.DangerousGetHandle().ToInt32(), "", EmptyPath, StatusType, out var status) != 0)
            throw new IOException(ErrorMessage(path, Marshal.GetLastPInvokeError()));
        return (status.Mask & StatusType) != 0 && (status.Mode & FileKind) == RegularKind;
    }

    private static string ErrorMessage(string path, int error) => $"Could not read '{path}': {new Win32Exception(error).Message}";

    // statx has a fixed 256-byte layout across Linux architectures.
    [StructLayout(LayoutKind.Explicit, Size = 256)]
    private struct LinuxFileStatus
    {
        [FieldOffset(0)] internal uint Mask;
        [FieldOffset(28)] internal ushort Mode;
    }

    [DllImport("libc", EntryPoint = "open", SetLastError = true)]
    private static extern int Open([MarshalAs(UnmanagedType.LPUTF8Str)] string path, int flags);

    [DllImport("libc", EntryPoint = "statx", SetLastError = true)]
    private static extern int Statx(int descriptor, [MarshalAs(UnmanagedType.LPUTF8Str)] string path,
        int flags, uint mask, out LinuxFileStatus status);

    [DllImport("libc", EntryPoint = "flock", SetLastError = true)]
    private static extern int Flock(int descriptor, int operation);

    [DllImport("kernel32.dll")]
    private static extern uint GetFileType(SafeFileHandle handle);
}
