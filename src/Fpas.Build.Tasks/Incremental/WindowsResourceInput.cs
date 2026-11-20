// [Documentation](../../../docs/build/msbuild.md#sdk-task-and-incremental-state)
using System.Text.Json;
using Microsoft.Build.Framework;

namespace Fpas.Build.Tasks.Incremental;

internal sealed record WindowsResourceInput(string Descriptor, string Icon)
{
    internal static WindowsResourceInput? Read(ITaskItem? item)
    {
        if (item is null) return null;
        var icon = item.GetMetadata("Icon");
        return new(JsonSerializer.Serialize(new[]
        {
            item.GetMetadata("FileVersion"), item.GetMetadata("ProductVersion"), item.GetMetadata("Title"),
            item.GetMetadata("Product"), item.GetMetadata("Company"), item.GetMetadata("Copyright"), icon
        }), icon);
    }
}
