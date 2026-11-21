// [Documentation](../../docs/build/distribution.md#windows-portable-launcher)
using System.ComponentModel;
using System.Diagnostics;
using System.Reflection;
using System.Runtime.InteropServices;

namespace Fpas.Deployment.WindowsPortable;

internal static class Program
{
    private static int Main(string[] args)
    {
        // The child shares the console and receives Ctrl+C itself. Wait for its exit code.
        Console.CancelKeyPress += (_, signal) => signal.Cancel = true;
        try
        {
            var bundle = Assembly.GetExecutingAssembly();
            var adjacent = Path.GetFullPath(AppContext.BaseDirectory);
            var name = bundle.GetName().Name!;
            var runtime = Path.Combine(adjacent, name + "-runtime");
            var executable = Payload.Extract(bundle, runtime);
            var start = new ProcessStartInfo(executable)
            {
                UseShellExecute = false,
                WorkingDirectory = Environment.CurrentDirectory
            };
            foreach (var argument in args)
                start.ArgumentList.Add(argument);

            // The CLI payload is itself a .NET single-file application.
            if (name.Equals("fpas", StringComparison.OrdinalIgnoreCase))
            {
                start.Environment["DOTNET_BUNDLE_EXTRACT_BASE_DIR"] = Path.Combine(Path.GetDirectoryName(executable)!, "dotnet");
                if (string.IsNullOrEmpty(Environment.GetEnvironmentVariable("FPAS_ASSET_CACHE")))
                    start.Environment["FPAS_ASSET_CACHE"] = Path.Combine(runtime, "fpas-assets");
            }

            using var child = Process.Start(start) ?? throw new IOException("The bundled program could not be started.");
            child.WaitForExit();
            return child.ExitCode;
        }
        catch (Exception error) when (error is IOException or InvalidDataException or UnauthorizedAccessException or Win32Exception)
        {
            var message = "Could not start the portable application. " + error.Message;
            Console.Error.WriteLine(message);
#if PORTABLE_GUI
            _ = MessageBox(0, message, "FPas-Amp", 0x10);
#endif
            return 1;
        }
    }

#if PORTABLE_GUI
    [DllImport("user32.dll", EntryPoint = "MessageBoxW", CharSet = CharSet.Unicode, ExactSpelling = true)]
    private static extern int MessageBox(nint window, string text, string caption, uint type);
#endif
}
