using Fpas.Tests.Support;

namespace Fpas.Gui.Cui.Tests;

/// <summary>Reads platform imports from the delivered native executable.</summary>
internal static class NativeDependencies
{
    internal static async Task<string> Read(string executable)
    {
        if (!OperatingSystem.IsWindows())
        {
            var result = await ProcessRunner.RunExecutableAsync("readelf", ["--wide", "--dynamic", executable]);
            Assert.True(result.ExitCode == 0, result.Output + result.Error);
            return result.Output;
        }
        string vswhere = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86),
            "Microsoft Visual Studio", "Installer", "vswhere.exe");
        var locate = await ProcessRunner.RunExecutableAsync(vswhere, ["-latest", "-products", "*", "-requires",
            "Microsoft.VisualStudio.Component.VC.Tools.x86.x64", "-find", @"VC\Tools\MSVC\**\bin\Hostx64\x64\dumpbin.exe"]);
        Assert.True(locate.ExitCode == 0, locate.Output + locate.Error);
        string dumpbin = Assert.Single(locate.Output.Split(['\r', '\n'], StringSplitOptions.RemoveEmptyEntries));
        var dump = await ProcessRunner.RunExecutableAsync(dumpbin, ["/dependents", executable]);
        Assert.True(dump.ExitCode == 0, dump.Output + dump.Error);
        return dump.Output;
    }
}
