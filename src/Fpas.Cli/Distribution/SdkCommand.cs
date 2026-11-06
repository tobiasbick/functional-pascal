using System.CommandLine;
using System.Security.Cryptography;
using Fpas.Cli.Projects;
using Fpas.Deployment;

namespace Fpas.Cli.Distribution;

internal sealed class SdkCommand
{
    internal Command Command { get; } = new("sdk", "Export the matching embedded project SDK for offline restore.\n\nExamples:\n  fpas sdk export ./feed\n");
    internal Command Licenses { get; } = new("licenses", "Print bundled debugger and runtime license notices.\n\nExamples:\n  fpas licenses\n");

    internal SdkCommand()
    {
        var export = new Command("export", """
            Export this compiler's SDK package to a local NuGet feed. Identical files are preserved;
            conflicting files are refused. Requires a published single-file FPAS distribution.

            Examples:
              fpas sdk export ./feed
              fpas init project Hello --sdk-feed ./feed
            """);
        var directory = new Argument<string>("directory") { Description = "Destination directory for the matching .nupkg." };
        var force = new Option<bool>("--force") { Description = "Replace a different package with the same filename." };
        export.Arguments.Add(directory);
        export.Options.Add(force);
        export.SetAction(result => Export(result.GetValue(directory)!, result.GetValue(force)));
        Command.Subcommands.Add(export);
        Licenses.SetAction(_ =>
        {
            Console.Write(File.ReadAllText(DistributionAssets.Sidecar(typeof(SdkCommand).Assembly,
                DistributionAssets.IsBundled ? "THIRD-PARTY-NOTICES.txt" : "debugger-notices.txt")));
            if (DistributionAssets.IsBundled)
                foreach (var name in new[] { "dotnet-license.txt", "dotnet-notices.txt" })
                    Console.Write(File.ReadAllText(DistributionAssets.Materialize(name)));
            return 0;
        });
    }

    private static int Export(string directory, bool force)
    {
        string? temporary = null;
        try
        {
            var name = "Fpas.NET.Sdk." + ToolSdk.Version + ".nupkg";
            var source = DistributionAssets.Materialize(name);
            var target = Path.Combine(Path.GetFullPath(directory), name);
            if (File.Exists(target) && !force)
            {
                using var left = File.OpenRead(source); using var right = File.OpenRead(target);
                if (!SHA256.HashData(left).AsSpan().SequenceEqual(SHA256.HashData(right)))
                    throw new IOException("The destination contains a different SDK package. Choose an empty feed directory.");
            }
            else
            {
                Directory.CreateDirectory(Path.GetDirectoryName(target)!);
                temporary = target + "." + Guid.NewGuid().ToString("N") + ".tmp";
                File.Copy(source, temporary);
                try { File.Move(temporary, target, overwrite: force); }
                catch (IOException) when (File.Exists(target) && File.ReadAllBytes(source).AsSpan().SequenceEqual(File.ReadAllBytes(target))) { }
            }
            Console.WriteLine(target); return 0;
        }
        catch (Exception error) when (error is IOException or UnauthorizedAccessException or ArgumentException)
        { Console.Error.WriteLine(error.Message); return 1; }
        finally { if (temporary is not null) File.Delete(temporary); }
    }
}
