using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Native checklist proposals and action availability.</summary>
/// <param name="Action">Index of the invoked step action, or minus one.</param>
/// <param name="Completed">Proposed completion flags.</param>
/// <param name="VisibleActions">Indices of rendered step action buttons.</param>
public readonly record struct CuiChecklistResult(int Action, int[] Completed, int[] VisibleActions);

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's native onboarding checklist.</summary>
    /// <param name="id">Stable checklist identity.</param>
    /// <param name="title">Optional card heading.</param>
    /// <param name="description">Optional replacement for the native progress caption.</param>
    /// <param name="titles">Ordered step captions.</param>
    /// <param name="descriptions">Supporting text per step.</param>
    /// <param name="actions">Action caption per step, empty when absent.</param>
    /// <param name="completed">Model-owned flags, zero or one.</param>
    /// <param name="enabled">Whether to accept input.</param>
    /// <param name="focus">One plus the part index to focus, zero for neither.</param>
    /// <param name="trackPart">Maps a circle or action slot to its inspection identity, or null.</param>
    /// <returns>Completion changes and the activated action.</returns>
    public unsafe CuiChecklistResult Checklist(string id, string title, string description, string[] titles,
        string[] descriptions, string[] actions, int[] completed, bool enabled, int focus, Func<int, string>? trackPart)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(titles);
        ArgumentNullException.ThrowIfNull(descriptions);
        ArgumentNullException.ThrowIfNull(actions);
        ArgumentNullException.ThrowIfNull(completed);
        int count = titles.Length, slots = checked(count * 2);
        if (descriptions.Length != count || actions.Length != count || completed.Length != count)
            throw new ArgumentException("Checklist arrays must have the same number of steps.");
        if (focus < 0 || focus > slots) throw new ArgumentOutOfRangeException(nameof(focus));
        Utf8.Validate(title, nameof(title));
        Utf8.Validate(description, nameof(description));
        for (int index = 0; index < count; ++index)
        {
            Utf8.Validate(titles[index], nameof(titles));
            Utf8.Validate(descriptions[index], nameof(descriptions));
            Utf8.Validate(actions[index], nameof(actions));
            if (string.IsNullOrWhiteSpace(titles[index])) throw new ArgumentException("A step needs a title.", nameof(titles));
            if (completed[index] is < 0 or > 1) throw new ArgumentOutOfRangeException(nameof(completed));
        }

        var proposed = (int[])completed.Clone();
        var nodes = trackPart is null ? [] : new int[slots];
        int action, node;
        fixed (int* flags = proposed, parts = nodes)
            action = NativeMethods.Checklist(context, Stable(id), Optional(title), Optional(description),
                titles, descriptions, actions, flags, count, enabled ? 1 : 0, focus, out node, parts);

        if (trackPart is null) return new(action, proposed, []);
        TrackControl(id, node);
        List<int> visible = [];
        for (int index = 0; index < count; ++index)
        {
            TrackControl(trackPart(index * 2), nodes[index * 2]);
            if (nodes[index * 2 + 1] < 0) continue;
            TrackControl(trackPart(index * 2 + 1), nodes[index * 2 + 1]);
            visible.Add(index);
        }
        return new(action, proposed, visible.ToArray());
    }
}
