using System.Text;
using Fpas.Compiler;

namespace Fpas.Cli.Compilation;

internal static class OutputFiles
{
    internal static void Publish(string output, CompiledProgram program, string? documentation = null)
    {
        var artifacts = new List<(string Path, byte[]? Image)>
        {
            (Path.ChangeExtension(output, ".runtimeconfig.json"), program.RuntimeConfiguration is null ? null : new UTF8Encoding(false).GetBytes(program.RuntimeConfiguration)),
            (Path.ChangeExtension(output, ".pdb"), program.PortablePdb),
            (output, program.AssemblyImage)
        };
        if (documentation is not null) artifacts.Add((documentation, program.XmlDocumentation));
        var stages = new Dictionary<string, string>(OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal);
        var backups = new Dictionary<string, string>();
        var published = new List<string>();
        var pending = new List<(string Path, string Temporary)>();
        var preserveBackups = false;
        try
        {
            foreach (var (path, image) in artifacts)
            {
                var directory = Path.GetDirectoryName(path)!;
                Directory.CreateDirectory(directory);
                if (!stages.TryGetValue(directory, out var staging))
                {
                    staging = Path.Combine(directory, ".fpas-" + Guid.NewGuid().ToString("N"));
                    Directory.CreateDirectory(staging); stages.Add(directory, staging);
                }
                if (Directory.Exists(path)) throw new IOException($"Output path '{path}' is a directory.");
                if (image is not null)
                {
                    var temporary = Path.Combine(staging, "new-" + pending.Count);
                    File.WriteAllBytes(temporary, image); pending.Add((path, temporary));
                }
                if (File.Exists(path))
                {
                    var backup = Path.Combine(staging, "old-" + backups.Count);
                    File.Move(path, backup); backups.Add(path, backup);
                }
            }
            foreach (var (path, temporary) in pending) { File.Move(temporary, path); published.Add(path); }
        }
        catch
        {
            try
            {
                foreach (var path in published)
                    File.Delete(path);
                foreach (var (target, backup) in backups)
                    File.Move(backup, target);
            }
            catch (Exception exception) when (exception is IOException or UnauthorizedAccessException)
            {
                preserveBackups = true;
                throw new IOException("Output rollback failed; previous files are retained in " + string.Join(", ", stages.Values) + ".", exception);
            }
            throw;
        }
        finally
        {
            if (!preserveBackups) foreach (var stage in stages.Values) Directory.Delete(stage, recursive: true);
        }
    }
}
