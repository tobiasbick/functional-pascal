// [Documentation](../../../docs/ui/gui/desktop.md)
using Fpas.Ui.Gui.Cui.Widgets.Overlays;

namespace Fpas.Ui.Gui.Cui.Tests.Windowing;

/// <summary>Checks native application shortcuts after widgets, including repeat and input ownership.</summary>
public sealed class CuiShortcutTests
{
    /// <summary>Exact modifiers, native repeats and disabled state respect the configured activation policy.</summary>
    [GuiFact]
    public void RepeatAndEnablePoliciesUseNativeKeyLevels()
    {
        using var window = new CuiWindow("Application shortcuts", renderer: CuiRenderer.Software, continuous: true);
        Assert.Equal(new[] { 0 }, Draw(new(33, 2), () => Tap(33, 2)));
        Assert.Empty(Draw(new(33, 2), () => Tap(33, 3)));
        Assert.Equal(new[] { 0 }, Draw(new(33, 2), () => window.SetKey(33, true, 2)));
        Assert.Empty(Draw(new(33, 2), () => window.SetKey(33, true, 2)));
        Assert.Equal(new[] { 0 }, Draw(new(33, 2, Repeat: true), () => window.SetKey(33, true, 2)));
        Assert.Empty(Draw(new(33, 2), () => window.SetKey(33, false)));
        Assert.Empty(Draw(new(33, 2, Enabled: false), () => window.SetKey(33, true, 2)));
        Assert.Empty(Draw(new(33, 2), () => window.SetKey(33, true, 2)));
        Draw(new(33, 2), () => window.SetKey(33, false));
        Assert.Equal(new[] { 0 }, Draw(new(33, 2), () => Tap(33, 2)));
        Assert.Equal(new[] { 0 }, Draw(new(67), () => Tap(67, 0)));

        void Tap(int key, int mods) { window.SetKey(key, true, mods); window.SetKey(key, false); }
        int[] Draw(CuiShortcut binding, Action input)
        {
            Assert.True(window.NextFrame()); input(); window.Text("Native shortcuts");
            int[] result = window.ReadShortcuts([binding]);
            Assert.Throws<InvalidOperationException>(() => window.ReadShortcuts([binding]));
            window.Present(); return result;
        }
    }

    /// <summary>Text editing yields only explicitly allowed combinations; native editor shortcuts stay first.</summary>
    [GuiFact]
    public void EditorsOwnTypingNavigationAndNativeFind()
    {
        using var window = new CuiWindow("Shortcut editor ownership", renderer: CuiRenderer.Software, continuous: true);
        Draw(0, new(33, 2), focus: true);
        Assert.Empty(Draw(33, new(33, 2)));
        Assert.Equal(new[] { 0 }, Draw(33, new(33, 2, AllowWhileEditing: true)));
        Assert.Empty(Draw(20, new(20, 2, AllowWhileEditing: true))); // Ctrl+F opens the native find bar.
        Assert.Empty(Draw(7, new(7, 2, AllowWhileEditing: true)));

        int[] Draw(int key, CuiShortcut binding, bool focus = false)
        {
            Assert.True(window.NextFrame());
            if (key != 0) { window.SetKey(key, true, 2); window.SetKey(key, false); }
            window.CodeEditor("editor", "A native editor", 128, "text", "", 1, 0, 200, 0, 4, focus ? 17 : 1);
            int[] result = window.ReadShortcuts([binding]); window.Present(); return result;
        }
    }

    /// <summary>An Escape that dismisses a popup never also activates an application binding.</summary>
    [GuiFact]
    public void PopupsRetainOwnershipOnTheirDismissalFrame()
    {
        using var window = new CuiWindow("Popup shortcut ownership", renderer: CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Draw(0); Draw(0);
        Assert.True(Draw(2).Open);
        Assert.False(Draw(3).Open);
        Assert.True(window.ControlFocused("trigger"));

        CuiMenuResult Draw(int key)
        {
            Assert.True(window.NextFrame());
            if (key != 0) { window.SetKey(key, true); window.SetKey(key, false); }
            var result = window.Menu("menu", "Actions", "", [new(CuiMenuEntryKind.Action, "save", "Save")],
                0, 0, 0, false, true, true, ["trigger", "save"]);
            Assert.Empty(window.ReadShortcuts([new(3), new(8)]));
            window.Present(); return result;
        }
    }

    /// <summary>Modal dialogs own global keys, including the frame that removes the modal.</summary>
    [GuiFact]
    public void ModalsBlockGlobalKeysThroughTheirClosingFrame()
    {
        using var window = new CuiWindow("Modal shortcut ownership", renderer: CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Assert.Empty(Draw(true)); Assert.Empty(Draw(false));
        Assert.Equal(new[] { 0 }, Draw(false));

        int[] Draw(bool open)
        {
            Assert.True(window.NextFrame()); window.SetKey(56, true); window.SetKey(56, false);
            window.Text("Background");
            if ((window.BeginDialog("modal", "Dialog", "", 300, open, true, false, true, false, "close") & 1) != 0)
            { window.Text("Modal content"); window.EndDialog(); }
            int[] result = window.ReadShortcuts([new(56, AllowWhileEditing: true)]);
            window.Present(); return result;
        }
    }

    /// <summary>Ambiguous combinations and invalid native key/modifier values are rejected.</summary>
    [GuiFact]
    public void InvalidBindingsDoNotConsumeTheFrame()
    {
        using var window = new CuiWindow("Shortcut validation", renderer: CuiRenderer.Software, continuous: true);
        Assert.Throws<InvalidOperationException>(() => window.ReadShortcuts([]));
        Assert.True(window.NextFrame());
        Assert.Throws<ArgumentException>(() => window.ReadShortcuts([new(0)]));
        Assert.Throws<ArgumentException>(() => window.ReadShortcuts([new(68)]));
        Assert.Throws<ArgumentException>(() => window.ReadShortcuts([new(15, 16)]));
        Assert.Throws<ArgumentException>(() => window.ReadShortcuts([new(33, 2), new(33, 2)]));
        Assert.Empty(window.ReadShortcuts([new(33, 2), new(33, 2, Enabled: false)]));
        window.Present();
    }
}
