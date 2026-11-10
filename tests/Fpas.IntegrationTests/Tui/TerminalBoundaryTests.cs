using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using Fpas.IntegrationTests.Debugging;

namespace Fpas.IntegrationTests.Tui;

public sealed class TerminalBoundaryTests
{
    [Fact]
    public async Task Terminal_libraries_build_and_execute_without_GUI_assets_or_toolchain()
    {
        using var target = new Debuggee("terminal-boundary");
        await TuiFixture.Compile(target, "shared.fpas");
        foreach (var file in new[] { "Fpas.Tui.dll", "Fpas.Ui.dll", "Fpas.Mvu.dll", "Fpas.Processes.dll" })
        {
            using var stream = File.OpenRead(target.FilePath(file));
            using var pe = new PEReader(stream);
            var metadata = pe.GetMetadataReader();
            var references = metadata.AssemblyReferences.Select(handle =>
                metadata.GetString(metadata.GetAssemblyReference(handle).Name)).ToArray();
            Assert.DoesNotContain(references, name => name.Contains("Gui", StringComparison.OrdinalIgnoreCase)
                || name.Contains("Cui", StringComparison.OrdinalIgnoreCase));
        }
        Assert.DoesNotContain(Directory.GetFiles(target.DirectoryPath, "*", SearchOption.AllDirectories),
            path => Path.GetFileName(path).Contains("cui", StringComparison.OrdinalIgnoreCase));
        Assert.Equal("shared terminal contracts verified\n", await target.Run());
    }
}
