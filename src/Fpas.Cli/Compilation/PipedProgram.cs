// [Documentation](../../../docs/pascal/program-structure/project-commands.md#run-source-from-stdin)
using Fpas.Cli.Toolchain;
using Fpas.Compiler;

namespace Fpas.Cli.Compilation;

internal sealed class PipedProgram : IDisposable
{
    private readonly string directory;
    private readonly string core;
    internal string Assembly { get; }

    private PipedProgram(string directory)
    {
        this.directory = directory;
        Assembly = Path.Combine(directory, PipedSource.AssemblyName + ".dll");
        core = Path.Combine(directory, "Fpas.Core.dll");
    }

    internal static PipedProgram Prepare(CompiledProgram compiled)
    {
        var program = new PipedProgram(CreateScratch());
        try
        {
            OutputFiles.Publish(program.Assembly, compiled);
            File.Copy(FrameworkReferences.CorePath, program.core);
            return program;
        }
        catch
        {
            program.Dispose();
            throw;
        }
    }

    private static string CreateScratch()
    {
        for (var ancestor = new DirectoryInfo(Environment.CurrentDirectory); ancestor is not null; ancestor = ancestor.Parent)
        {
            var git = Path.Combine(ancestor.FullName, ".git");
            if (!Directory.Exists(git) && !File.Exists(git)) continue;
            try
            {
                return Directory.CreateDirectory(Path.Combine(ancestor.FullName, ".temp-data", "fpas-stdin-" + Guid.NewGuid().ToString("N"))).FullName;
            }
            catch (Exception error) when (error is IOException or UnauthorizedAccessException) { break; }
        }
        return Directory.CreateTempSubdirectory("fpas-stdin-").FullName;
    }

    public void Dispose()
    {
        try
        {
            var scratch = new DirectoryInfo(directory);
            if (!scratch.Exists || (scratch.Attributes & FileAttributes.ReparsePoint) != 0) return;
            File.Delete(Assembly);
            File.Delete(Path.ChangeExtension(Assembly, ".runtimeconfig.json"));
            File.Delete(core);
            // Remove only owned files. The program can create other files in its assembly directory.
            scratch.Delete();
        }
        catch (Exception error) when (error is IOException or UnauthorizedAccessException) { }
    }
}
