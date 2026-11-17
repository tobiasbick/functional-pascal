using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    // CUI's public cui_key ordinals: F1=56 through F12=67, followed by COUNT.
    private const int ShortcutKeyCount = 68;
    private readonly byte[] shortcutHeld = new byte[ShortcutKeyCount];
    private int shortcutOwners;
    private bool shortcutsRead;

    private unsafe void SnapshotShortcutKeys()
    {
        fixed (byte* keys = shortcutHeld) NativeMethods.ShortcutKeys(context, keys, shortcutHeld.Length);
    }

    /// <summary>Reads application bindings once after all widgets, with native input owners taking priority.</summary>
    /// <param name="bindings">Model-owned bindings; enabled combinations must be unique.</param>
    /// <returns>Matching binding indices, each at most once in the current native frame.</returns>
    public int[] ReadShortcuts(CuiShortcut[] bindings)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(bindings);
        if (shortcutsRead) throw new InvalidOperationException("Read application shortcuts once after drawing the view.");
        var unique = new HashSet<(int, int)>();
        foreach (var binding in bindings)
        {
            if (binding.Key <= 0 || binding.Key >= ShortcutKeyCount || (binding.Modifiers & ~15) != 0)
                throw new ArgumentException("Use valid native keys and modifiers.", nameof(bindings));
            if (binding.Enabled && !unique.Add((binding.Key, binding.Modifiers)))
                throw new ArgumentException("Enabled application shortcut combinations must be unique.", nameof(bindings));
        }
        shortcutsRead = true;
        int owners = shortcutOwners | NativeMethods.ShortcutOwners(context);
        List<int> triggered = [];
        if ((owners & 2) != 0) return [];
        for (int index = 0; index < bindings.Length; index++)
        {
            var binding = bindings[index];
            if (!binding.Enabled || !binding.Repeat && shortcutHeld[binding.Key] != 0 ||
                !binding.AllowWhileEditing && (owners & 1) != 0) continue;
            // Focused native widgets own editing/navigation and unmodified non-function keys.
            if ((owners & 4) != 0 && (binding.Key <= 14 ||
                binding.Key < 56 && (binding.Modifiers & ~1) == 0)) continue;
            if (NativeMethods.TakeShortcut(context, binding.Modifiers, binding.Key) != 0) triggered.Add(index);
        }
        return triggered.ToArray();
    }
}
