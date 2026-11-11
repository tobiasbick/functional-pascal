using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's breadcrumb trail with an informational final item.</summary>
    /// <param name="id">Stable trail identity.</param>
    /// <param name="labels">Nonempty positional path captions.</param>
    /// <param name="enabled">Whether ancestor links accept input.</param>
    /// <param name="focus">Zero or one plus the requested ancestor index.</param>
    /// <param name="trackingIds">Trail and path item tracking IDs, or null.</param>
    /// <returns>The activated ancestor index, or minus one.</returns>
    public unsafe int Breadcrumb(string id, string[] labels, bool enabled, int focus, string[]? trackingIds)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(labels);
        int count = labels.Length;
        if (count == 0) throw new ArgumentException("Supply at least one path item.", nameof(labels));
        if (focus < 0 || focus >= count) throw new ArgumentOutOfRangeException(nameof(focus));
        if (trackingIds is not null && trackingIds.Length != count + 1)
            throw new ArgumentException("Supply trail and item tracking IDs.", nameof(trackingIds));
        foreach (string label in labels)
        {
            Utf8.Validate(label, nameof(labels));
            ArgumentException.ThrowIfNullOrWhiteSpace(label, nameof(labels));
        }
        var nodes = new int[count + 1];
        int proposal;
        fixed (int* parts = nodes)
            proposal = NativeMethods.Breadcrumb(context, Stable(id), labels, count, enabled ? 1 : 0, focus, parts);
        if (trackingIds is not null)
            for (int index = 0; index < nodes.Length; index++) TrackControl(trackingIds[index], nodes[index]);
        return proposal;
    }
}
