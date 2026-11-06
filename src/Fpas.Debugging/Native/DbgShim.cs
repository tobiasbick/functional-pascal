using System.Runtime.InteropServices;
using System.Text;

namespace Fpas.Debugging.Native;

internal sealed class DbgShim
{
    internal const string Version = "10.0.750501";
    private static readonly Lazy<DbgShim> instance = new(() => new());
    internal static DbgShim Instance => instance.Value;
    private readonly nint library;
    internal readonly Launch CreateProcess;
    internal readonly HandleOperation Resume, CloseResume, Unregister;
    internal readonly Register Startup;

    private DbgShim()
    {
        if (RuntimeInformation.ProcessArchitecture != Architecture.X64 || !(OperatingSystem.IsWindows() || OperatingSystem.IsLinux()))
            throw new PlatformNotSupportedException("The FPAS debugger requires Windows x64 or Linux x64.");
        // Keep the shim loaded for the lifetime of native callbacks and the process.
        library = NativeLibrary.Load(OperatingSystem.IsWindows() ? "dbgshim.dll" : "libdbgshim.so", typeof(DbgShim).Assembly, null);
        CreateProcess = Export<Launch>("CreateProcessForLaunch");
        Resume = Export<HandleOperation>("ResumeProcess");
        CloseResume = Export<HandleOperation>("CloseResumeHandle");
        Startup = Export<Register>("RegisterForRuntimeStartup");
        Unregister = Export<HandleOperation>("UnregisterForRuntimeStartup");
    }

    private T Export<T>(string name) where T : Delegate => Marshal.GetDelegateForFunctionPointer<T>(NativeLibrary.GetExport(library, name));

    internal static string CommandLine(string executable, IEnumerable<string> arguments)
        => string.Join(' ', new[] { executable }.Concat(arguments).Select(Quote));

    private static string Quote(string value)
    {
        if (value.Contains('\0')) throw new ArgumentException("Program arguments cannot contain NUL.");
        return OperatingSystem.IsWindows() ? QuoteWindows(value) : QuotePal(value);
    }

    private static string QuotePal(string value)
    {
        var end = value.Length;
        while (end > 0 && value[end - 1] == '\\') end--;
        var text = new StringBuilder("\"");
        foreach (var character in value.AsSpan(0, end))
        {
            if (character == '"') text.Append('\\');
            text.Append(character);
        }
        return text.Append('"').Append(value.AsSpan(end)).ToString();
    }

    private static string QuoteWindows(string value)
    {
        var text = new StringBuilder("\""); var slashes = 0;
        foreach (var character in value)
        {
            if (character == '\\') { slashes++; continue; }
            text.Append('\\', character == '"' ? slashes * 2 + 1 : slashes).Append(character); slashes = 0;
        }
        return text.Append('\\', slashes * 2).Append('"').ToString();
    }

    [UnmanagedFunctionPointer(CallingConvention.StdCall)]
    internal delegate int Launch([MarshalAs(UnmanagedType.LPWStr)] string commandLine, int suspended, nint environment,
        [MarshalAs(UnmanagedType.LPWStr)] string directory, out uint processId, out nint resume);
    [UnmanagedFunctionPointer(CallingConvention.StdCall)] internal delegate int HandleOperation(nint handle);
    [UnmanagedFunctionPointer(CallingConvention.StdCall)] internal delegate int Register(uint processId, nint callback, nint parameter, out nint registration);
    [UnmanagedFunctionPointer(CallingConvention.StdCall)] internal delegate void StartupCallback(nint corDebug, nint parameter, int result);
}
