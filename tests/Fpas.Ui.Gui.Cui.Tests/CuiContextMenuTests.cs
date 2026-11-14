using Fpas.Ui.Gui.Cui.Widgets.Actions;
using Fpas.Ui.Gui.Cui.Widgets.Overlays;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks context menus delegated to CUI on existing target nodes.</summary>
public sealed class CuiContextMenuTests
{
    /// <summary>Uses native right-click opening, keyboard selection, disabled rows and lifecycle.</summary>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void ContextCommandsUseTheExistingTarget(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Native context menu", 760, 540, renderer, continuous: true);
        window.SetReducedMotion(true);
        CuiMenuEntry[] entries = [new(CuiMenuEntryKind.Label, "", "Commands"),
            new(CuiMenuEntryKind.Action, "disabled", "Disabled", Enabled: false),
            new(CuiMenuEntryKind.Separator, "", ""),
            new(CuiMenuEntryKind.Action, "copy", "Copy ## literally", Shortcut: "Ctrl C", Checked: true),
            new(CuiMenuEntryKind.Action, "delete", "Delete", Danger: true)];
        Draw();
        var target = window.GetControlRectangle("context");
        var point = new CuiRectangle { X = target.X + 20, Y = target.Y + 20 };
        Assert.True(Draw(point: point, right: true).Open);
        Assert.True(Draw(point: point, right: false).Open);
        Assert.Equal(target, window.GetControlRectangle("context"));
        Assert.Equal(1, Draw(key: 10).Highlighted);
        Assert.Equal(-1, Draw(key: 2).Choice);
        Assert.Equal(3, Draw(key: 10).Highlighted);
        Assert.Equal(3, Draw(key: 2).Choice);
        Assert.False(Draw().Open);
        Assert.Throws<KeyNotFoundException>(() => window.GetControlRectangle("copy"));
        Assert.True(Draw(point: point, right: true).Open);
        Draw(point: point, right: false);
        Assert.False(Draw(key: 3).Open);
        Assert.True(Draw(point: point, right: true).Open);
        Assert.False(Draw(enabled: false, point: point, right: true).Open);
        Assert.False(Draw(point: point, right: false).Open);
        Assert.True(Draw(point: point, right: true).Open);
        Draw(attach: false, right: false);
        Assert.False(Draw().Open);
        Assert.Equal(target, window.GetControlRectangle("context"));

        CuiMenuResult Draw(bool attach = true, bool enabled = true, CuiRectangle? point = null,
            bool? right = null, int key = 0)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(point?.X ?? -1, point?.Y ?? -1);
            if (right.HasValue) window.SetRightButton(right.Value);
            if (key != 0) { window.SetKey(key, true); window.SetKey(key, false); }
            window.BeginColumn();
            window.BeginGroup(false, 28, 0, 0, false);
            window.Text("Right click this area");
            window.EndGroup();
            window.StyleLast(7);
            var result = attach ? window.ContextMenu("context", entries, enabled,
                ["context", "header", "disabled", "separator", "copy", "delete"]) : new CuiMenuResult(-1, false, -1);
            window.EndColumn();
            window.Present();
            return result;
        }
    }

    /// <summary>Retiring the context menu leaves the target's native confirmation state intact.</summary>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void ContextRetirementPreservesTargetState(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Context target ownership", 500, 320, renderer, continuous: true);
        window.SetReducedMotion(true);
        Assert.False(Draw(focus: true).Armed);
        Assert.True(Draw(key: 2).Armed);
        Assert.True(Draw(attach: false).Armed);
        Assert.True(Draw(attach: false, key: 2).Confirmed);

        CuiConfirmButtonResult Draw(bool attach = true, bool focus = false, int key = 0)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            if (key != 0) { window.SetKey(key, true); window.SetKey(key, false); }
            window.BeginColumn();
            var result = window.ConfirmButton("target", "Delete", "Confirm", "", 3000, 4, 0, true, focus);
            window.TrackLastControl("target");
            if (attach) window.ContextMenu("commands", [new(CuiMenuEntryKind.Action, "copy", "Copy")], true, ["context", "copy"]);
            window.EndColumn();
            window.Present();
            return result;
        }
    }
}
