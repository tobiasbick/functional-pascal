// [Documentation](../../docs/build/msbuild.md#documentation-output)
using Fpas.Build.Tasks.Incremental;
using Fpas.Build.Tasks.Documentation;
using Microsoft.Build.Framework;

namespace Fpas.Build.Tasks;

/// <summary>Deletes XML documentation files recorded by the compiler cache.</summary>
public sealed class FpasCleanDocumentation : Microsoft.Build.Utilities.Task
{
    /// <summary>Gets or sets the compiler cache file that records generated documentation.</summary>
    [Required] public string CacheFile { get; set; } = "";

    /// <inheritdoc/>
    public override bool Execute()
    {
        try
        {
            var state = CompileFingerprint.Read(Path.GetFullPath(CacheFile));
            DocumentationOutputs.Publish([], state?.DocumentationFiles ?? []);
            return true;
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException)
        {
            Log.LogError(null, "FP5002", null, null, 0, 0, 0, 0, "Cannot clean generated XML documentation: " + exception.Message);
            return false;
        }
    }
}
