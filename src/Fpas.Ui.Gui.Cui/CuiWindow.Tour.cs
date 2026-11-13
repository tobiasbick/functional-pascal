using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Native tour navigation and visible action slots.</summary>
/// <param name="Step">Proposed step, minus one when closed.</param>
/// <param name="Event">None zero, next one, back two, done three, skipped four.</param>
/// <param name="VisibleParts">Close zero, Back one and Next/Done two when present.</param>
public readonly record struct CuiTourResult(int Step, int Event, int[] VisibleParts);

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's spotlight tour, resolving targets through this frame's tracked controls.</summary>
    /// <param name="id">Stable tour identity.</param>
    /// <param name="targets">Tracking IDs; absent or empty entries center the card.</param>
    /// <param name="titles">Literal headings, one per step.</param>
    /// <param name="bodies">Literal supporting text, one per step.</param>
    /// <param name="placements">Native placement ordinals from zero through nine.</param>
    /// <param name="step">Model-owned step, minus one when closed.</param>
    /// <param name="width">Logical card width; zero uses the native default.</param>
    /// <param name="padding">Logical target padding; zero uses the native default.</param>
    /// <param name="blockTarget">Whether to block the highlighted control's pointer input.</param>
    /// <param name="trackPart">Maps a native action slot to its inspection identity.</param>
    /// <param name="focusPart">One plus the slot to focus, zero for no request.</param>
    /// <returns>The native proposal and rendered actions; the input model is unchanged.</returns>
    public unsafe CuiTourResult Tour(string id, string[] targets, string[] titles, string[] bodies,
        int[] placements, int step, float width = 0, float padding = 0, bool blockTarget = false,
        Func<int, string>? trackPart = null, int focusPart = 0)
    {
        CheckFrame();
        string identity = Stable(id);
        ArgumentNullException.ThrowIfNull(targets);
        ArgumentNullException.ThrowIfNull(titles);
        ArgumentNullException.ThrowIfNull(bodies);
        ArgumentNullException.ThrowIfNull(placements);
        int count = targets.Length;
        if (count == 0 || titles.Length != count || bodies.Length != count || placements.Length != count)
            throw new ArgumentException("A tour needs equally sized, nonempty step arrays.");
        if (step < -1 || step >= count) throw new ArgumentOutOfRangeException(nameof(step));
        if (focusPart is < 0 or > 3) throw new ArgumentOutOfRangeException(nameof(focusPart));
        CheckFeedbackNumber(width, nameof(width)); CheckFeedbackNumber(padding, nameof(padding));
        var keys = new uint[count];
        for (int index = 0; index < count; ++index)
        {
            Utf8.Validate(targets[index], nameof(targets));
            Utf8.Validate(titles[index], nameof(titles));
            Utf8.Validate(bodies[index], nameof(bodies));
            if (placements[index] is < 0 or > 9) throw new ArgumentOutOfRangeException(nameof(placements));
            if (trackedNodes.TryGetValue(targets[index], out int target) && target >= 0) keys[index] = NativeMethods.NodeKey(context, target);
        }
        if (step < 0) return new(step, 0, []);
        string[] identities = trackPart is null ? [] : Enumerable.Range(0, 3).Select(trackPart).ToArray();
        int proposed = step, node, action;
        uint key;
        var parts = new int[3];
        fixed (uint* targetKeys = keys)
        fixed (int* positions = placements, nodes = parts)
            action = NativeMethods.Tour(context, identity, targetKeys, titles, bodies, positions, count,
                ref proposed, width, padding, blockTarget ? 1 : 0, focusPart, out node, out key, nodes);
        TrackControl(id, node, key);
        List<int> visible = [];
        if (identities.Length > 0)
            for (int slot = 0; slot < 3; ++slot)
                if (parts[slot] >= 0) { TrackControl(identities[slot], parts[slot]); visible.Add(slot); }
        return new(proposed, action, visible.ToArray());
    }
}
