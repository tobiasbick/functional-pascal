using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native confirmation panel anchored to the last button.</summary>
    /// <param name="id">Stable panel and snapshot identity.</param>
    /// <param name="title">Question above the actions.</param>
    /// <param name="description">Optional supporting text.</param>
    /// <param name="confirm">Affirmative button caption.</param>
    /// <param name="cancel">Cancellation button caption.</param>
    /// <param name="placement">CUI placement relative to the anchor.</param>
    /// <param name="danger">Use the danger variant for confirmation.</param>
    /// <param name="open">Model-owned visibility.</param>
    /// <param name="enabled">Whether the panel accepts input.</param>
    /// <param name="cancelControl">Snapshot identity of the Cancel button.</param>
    /// <param name="confirmControl">Snapshot identity of the Confirm button.</param>
    /// <returns>Confirmation for bit 1, dismissal for bit 2 and an open panel for bit 4.</returns>
    public unsafe int ConfirmPopover(string id, string title, string description, string confirm, string cancel,
        int placement, bool danger, bool open, bool enabled, string cancelControl, string confirmControl)
    {
        CheckFrame();
        Utf8.Validate(title, nameof(title));
        Utf8.Validate(description, nameof(description));
        Utf8.Validate(confirm, nameof(confirm));
        Utf8.Validate(cancel, nameof(cancel));
        ArgumentException.ThrowIfNullOrWhiteSpace(title);
        ArgumentException.ThrowIfNullOrWhiteSpace(confirm);
        ArgumentException.ThrowIfNullOrWhiteSpace(cancel);
        if (placement is < 0 or > 9) throw new ArgumentOutOfRangeException(nameof(placement));
        CheckPopupDepth();
        int* nodes = stackalloc int[3];
        int result = NativeMethods.ConfirmPopover(context, Stable(id), title,
            description.Length == 0 ? null : description, confirm, cancel, placement,
            danger ? 1 : 0, open ? 1 : 0, enabled ? 1 : 0, nodes, out uint owner);
        if (owner != 0) TrackModal(id, owner);
        if (nodes[0] >= 0) trackedNodes.Add(id, nodes[0]);
        if (nodes[1] >= 0) TrackControl(cancelControl, nodes[1]);
        if (nodes[2] >= 0) TrackControl(confirmControl, nodes[2]);
        return result;
    }
}
