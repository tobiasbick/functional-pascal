// [Documentation](../../../docs/ui/gui/overlays/README.md)
using Fpas.Tests.Support;
using Fpas.Ui.Gui.Cui.Widgets.Actions;
using Fpas.Ui.Gui.Cui.Widgets.Canvas;
using Fpas.Ui.Gui.Cui.Widgets.Overlays;

namespace Fpas.Ui.Gui.Cui.Tests.Overlays;

/// <summary>Checks native avatar, tour-caption and count-only selection options.</summary>
public sealed class CuiWidgetOptionTests
{
    /// <summary>Replaces a decoded avatar without losing the open menu or its native focus.</summary>
    [GuiFact]
    public void UserMenuReplacesAndReleasesItsAvatar()
    {
        using var directory = new TemporaryDirectory();
        string blue = directory.File("blue.png"), red = directory.File("red.png");
        File.WriteAllBytes(blue, TestImages.Blue); File.WriteAllBytes(red, TestImages.Red);
        using var window = new CuiWindow("User menu avatar", 420, 300, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Draw(blue, focus: true); Draw(blue);
        uint first = Center("blue");
        Assert.True(Draw(blue, enter: true).Open);
        Assert.True(Draw(red).Open);
        uint second = Center("red");
        Assert.NotEqual(first, second);
        Assert.True(window.ControlFocused("account"));
        Assert.True(window.NextFrame()); window.Text("Account removed"); window.Present();
        File.Delete(red);
        Draw(red); Draw(red);
        Assert.NotEqual(second, Center("missing"));

        uint Center(string name)
        {
            var box = window.GetControlRectangle("account");
            string capture = directory.File(name + ".png"); window.CapturePng(capture);
            return PngAssertions.ReadPixels(capture, 420, 300)[(int)(box.Y + box.Height / 2) * 420 + (int)(box.X + box.Width / 2)];
        }
        CuiMenuResult Draw(string path, bool focus = false, bool enter = false)
        {
            Assert.True(window.NextFrame());
            if (enter) { window.SetKey(2, true); window.SetKey(2, false); }
            var result = window.UserMenu("account", "Example Person", "Account", "online", true,
                [new(CuiMenuEntryKind.Action, "profile", "Profile")], 0, 0, true, focus, ["account", "profile"], path);
            window.Present(); return result;
        }
    }

    /// <summary>Preserves CUI's blank user detail and gray status independently of omission.</summary>
    [GuiFact]
    public void UserMenuDistinguishesEmptyDetailAndStatusFromOmission()
    {
        const int width = 360, height = 240;
        using var window = new CuiWindow("Optional account details", width, height, CuiRenderer.Software, continuous: true);
        using var directory = new TemporaryDirectory();
        window.SetReducedMotion(true);
        uint[] omitted = Draw("omitted", null, null);
        Assert.False(omitted.SequenceEqual(Draw("blank-detail", "", null)));
        Assert.False(omitted.SequenceEqual(Draw("gray-status", null, "")));

        uint[] Draw(string key, string? detail, string? status)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            window.UserMenu("account", "Example Person", detail, status, false,
                [new(CuiMenuEntryKind.Action, "profile", "Profile")], 0, 0, true, false, ["account", "profile"]);
            window.Present();
            string file = directory.File(key + ".png");
            window.CapturePng(file);
            return PngAssertions.ReadPixels(file, width, height);
        }
    }

    /// <summary>CUI draws label-only and glyph-only actions and distinguishes a blank trigger from the default plus.</summary>
    [GuiFact]
    public void SpeedDialPreservesOptionalNativeParts()
    {
        const int width = 360, height = 260;
        using var window = new CuiWindow("Optional speed dial parts", width, height, CuiRenderer.Software, continuous: true);
        using var directory = new TemporaryDirectory();
        window.SetReducedMotion(true);
        Assert.False(Draw(null).Open);
        uint[] plus = Pixels("plus");
        Assert.False(Draw("").Open);
        Assert.False(plus.SequenceEqual(Pixels("blank")));

        Draw(focus: 1);
        var opened = Draw(enter: true);
        Assert.True(opened.Open);
        Assert.Equal([0, 1], opened.VisibleActions);
        Assert.True(window.GetControlRectangle("action-0").Width < window.GetControlRectangle("action-1").Width);
        Assert.True(Draw(focus: 2).Open);
        Assert.True(window.ControlFocused("action-0"));
        var chosen = Draw(enter: true);
        Assert.Equal(0, chosen.Choice);
        Assert.False(chosen.Open);

        CuiSpeedDialResult Draw(string? icon = null, int focus = 0, bool enter = false)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            if (enter) { window.SetKey(2, true); window.SetKey(2, false); }
            var result = window.SpeedDial("dial", [null, "Caption only"], ["\ue0cc", null],
                icon, null, 2, true, focus, index => "action-" + index);
            window.Present();
            return result;
        }

        uint[] Pixels(string name)
        {
            string file = directory.File(name + ".png");
            window.CapturePng(file);
            return PngAssertions.ReadPixels(file, width, height);
        }
    }

    /// <summary>Changing a translated caption retains the same native focus and navigation identity.</summary>
    [GuiFact]
    public void TourCaptionsKeepFocusAcrossTranslation()
    {
        using var window = new CuiWindow("Tour captions", 600, 420, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Draw("Continue ## literally", focus: 3);
        Draw("Weiter");
        Assert.True(window.ControlFocused("part-2"));
        Draw("###");
        Assert.True(window.ControlFocused("part-2"));
        var next = Draw("Continuer", enter: true);
        Assert.Equal(1, next.Event); Assert.Equal(1, next.Step);

        CuiTourResult Draw(string label, int focus = 0, bool enter = false)
        {
            Assert.True(window.NextFrame());
            if (enter) { window.SetKey(2, true); window.SetKey(2, false); }
            window.Text("Base");
            var result = window.Tour("tour", ["", ""], ["First", "Second"], ["Body", "Body"], [0, 0], 0,
                trackPart: slot => "part-" + slot, focusPart: focus,
                nextLabel: label, backLabel: "Previous", doneLabel: "Finish", countFormat: "Step %d / %d");
            window.Present(); return result;
        }
    }

    /// <summary>Native count-only selection clears by Escape and supports validated integer formats.</summary>
    [GuiFact]
    public void AggregateSelectionUsesNativeCountAndClear()
    {
        using var window = new CuiWindow("Aggregate selection", 640, 420, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Draw(); Draw();
        Assert.Equal(-2, Draw(escape: true));
        Assert.Equal(-1, Draw(escape: true, noEscape: true));
        Assert.True(window.NextFrame()); window.Text("Anchor");
        foreach (string format in new[] { "%s", "%n", "%d %d", "%", "%1$d" })
            Assert.Throws<ArgumentException>(() => window.SelectionBar("selection", null, [], true, false, 12, format));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.SelectionBar("selection", null, [], true, false, -1));
        Assert.Throws<ArgumentException>(() => window.Tour("tour", [""], ["Title"], ["Body"], [0], 0, countFormat: "%n"));
        Assert.Throws<ArgumentException>(() => window.Tour("tour", [""], ["Title"], ["Body"], [0], 0, countFormat: "%d %d %d"));
        window.Present();

        int Draw(bool escape = false, bool noEscape = false)
        {
            Assert.True(window.NextFrame());
            if (escape) { window.SetKey(3, true); window.SetKey(3, false); }
            window.Text("Anchor");
            int result = window.SelectionBar("selection", null, [new CuiSelectionAction("", "Delete", 0)],
                true, noEscape, 1200, "%d chosen (100%%)");
            window.Present(); return result;
        }
    }
}
