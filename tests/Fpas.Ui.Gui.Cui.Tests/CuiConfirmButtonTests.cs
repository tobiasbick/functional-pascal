using Fpas.Tests.Support;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks the managed native bridge's confirmation and lifetime contracts.</summary>
public sealed class CuiConfirmButtonTests
{
    /// <summary>Retains identity through both captions, routes two activations and retires removed state.</summary>
    [GuiFact]
    public void ConfirmationPreservesFocusAndRetiresDisabledOrRemovedState()
    {
        using var window = new CuiWindow("Confirm button", 640, 480, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Assert.Equal(new(false, false), Draw(focus: true));
        float width = window.GetControlRectangle("confirm").Width;
        Assert.Equal(new(false, true), Draw(key: 2));
        Assert.True(window.ControlFocused("confirm"));
        Assert.Equal(new(true, false), Draw(key: 6, label: "Renamed ### literally"));
        Assert.Equal(new(false, false), Draw());
        Assert.Equal(width, window.GetControlRectangle("confirm").Width);
        Assert.Equal(new(false, true), Draw(key: 2));
        Assert.Equal(new(false, false), Draw(key: 3));
        Assert.Equal(new(false, true), Draw(key: 6));
        Assert.Equal(new(false, false), Draw(enabled: false));
        Assert.False(window.ControlFocused("confirm"));
        Assert.Equal(new(false, true), Draw(key: 2, focus: true));
        window.ForgetControl("confirm");
        Assert.Equal(new(false, false), Draw(focus: true));
        Assert.Equal(new(false, true), Draw(key: 2));

        CuiConfirmButtonResult Draw(int key = 0, bool enabled = true, bool focus = false,
            string label = "Delete ## literally")
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            if (key != 0) { window.SetKey(key, true); window.SetKey(key, false); }
            window.BeginColumn();
            var result = window.ConfirmButton("confirm", label, "Confirm ## literally", "", 3000, 2, 0, enabled, focus);
            window.TrackLastControl("confirm");
            window.EndColumn();
            window.Present();
            return result;
        }
    }

    /// <summary>Changing a control's kind retires companion state owned by its former kind.</summary>
    [GuiFact]
    public void ReusingIdentityForAnotherKindRetiresConfirmation()
    {
        using var window = new CuiWindow("Control replacement", 480, 320, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Assert.False(Draw().Armed);
        Assert.True(Draw(activate: true).Armed);
        Draw(status: true);
        Assert.False(Draw().Armed);
        Assert.True(Draw(activate: true).Armed);

        CuiConfirmButtonResult Draw(bool status = false, bool activate = false)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            if (activate) { window.SetKey(2, true); window.SetKey(2, false); }
            window.BeginColumn();
            var result = new CuiConfirmButtonResult(false, false);
            if (status)
                window.StatusButton("shared", "Save", 0, "Saving", "Saved", "Failed", "", 0, 0, true, false);
            else
                result = window.ConfirmButton("shared", "Delete", "Confirm", "", 3000, 2, 0, true, true);
            window.TrackLastControl("shared");
            window.EndColumn();
            window.Present();
            return result;
        }
    }

    /// <summary>Rejects invalid native options before entering CUI.</summary>
    [GuiFact]
    public void ConfirmationValidatesTimeoutVariantAndSize()
    {
        using var window = new CuiWindow("Confirm validation", 480, 320, CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        Assert.Throws<ArgumentOutOfRangeException>(() => Draw(timeout: 0));
        Assert.Throws<ArgumentOutOfRangeException>(() => Draw(timeout: -1));
        Assert.Throws<ArgumentOutOfRangeException>(() => Draw(variant: 3));
        Assert.Throws<ArgumentOutOfRangeException>(() => Draw(size: 3));
        Assert.Throws<ArgumentException>(() => window.ConfirmButton("id", "Delete", "Bad\0label", "", 3000, 2, 0, true, false));

        void Draw(int timeout = 3000, int variant = 2, int size = 0) =>
            window.ConfirmButton("id", "Delete", "Confirm", "", timeout, variant, size, true, false);
    }
}
