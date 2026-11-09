namespace Fpas.Gui.Cui.Tests;

/// <summary>Exercises native scroll and dialog scopes through the managed window.</summary>
public sealed class CuiContainerTests
{
    /// <summary>Checks that scrolling moves the contents while the following widget stays in place.</summary>
    [GuiFact]
    [Trait("Category", "NativeGui")]
    public void ScrollToMovesContentWithinTheFixedViewport()
    {
        using var window = new CuiWindow("Scroll container", renderer: CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        const string id = "Aktivität-🧭";
        const float height = 112;
        const float padding = 12;
        const float gap = 8;
        const float offset = 80;

        Draw();
        var initial = Draw(offset);
        var scrolled = Draw();

        Assert.True(initial.Item.Width > 0);
        Assert.Equal(padding + height + gap, initial.Following.Y);
        Assert.Equal(initial.Item.Y - offset, scrolled.Item.Y);
        Assert.Equal(initial.Following.Y, scrolled.Following.Y);

        (CuiRectangle Item, CuiRectangle Following) Draw(float? scrollOffset = null)
        {
            Assert.True(window.NextFrame());
            window.BeginColumn(padding, gap);
            if (scrollOffset is float y) window.ScrollTo(id, y);
            window.BeginScroll(id, height);
            CuiRectangle item = default;
            for (int index = 0; index < 8; index++)
            {
                window.Button($"Item {index}");
                if (index == 2) item = window.LastRectangle;
            }
            window.EndScroll();
            window.Button("After scrolling");
            CuiRectangle following = window.LastRectangle;
            window.EndColumn();
            return (item, following);
        }
    }

    /// <summary>Checks pointer routing and native dismissal of a host-opened modal dialog.</summary>
    [GuiFact]
    [Trait("Category", "NativeGui")]
    public void ModalDialogReceivesClicksAndDismissesWithoutActivatingTheBackground()
    {
        using var window = new CuiWindow("Modal dialog", renderer: CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        bool open = false;
        int backgroundClicks = 0;
        int dialogClicks = 0;
        CuiRectangle background = default, action = default;

        Assert.False(Draw());
        open = true;
        Assert.True(Draw());
        Assert.True(Draw());
        Click(action);
        Assert.Equal(1, dialogClicks);
        Assert.True(open);

        Click(background);
        Assert.False(open);
        Assert.Equal(0, backgroundClicks);

        Assert.False(Draw());
        Click(background);
        Assert.Equal(1, backgroundClicks);

        void Click(CuiRectangle target)
        {
            Assert.True(target.Width > 0 && target.Height > 0);
            Draw(target);
            Draw(target, down: true);
            Draw(target, down: false);
        }

        bool Draw(CuiRectangle? target = null, bool? down = null)
        {
            Assert.True(window.NextFrame());
            if (target is CuiRectangle rect)
                window.MovePointer(rect.X + rect.Width / 2, rect.Y + rect.Height / 2);
            if (down is bool pressed) window.SetLeftButton(pressed);
            window.BeginColumn(padding: 12);
            if (window.Button("Background")) backgroundClicks++;
            background = window.LastRectangle;
            window.EndColumn();

            if (!window.BeginDialog("Bestätigen###confirmation", ref open)) return false;
            if (window.Button("Confirm")) dialogClicks++;
            action = window.LastRectangle;
            window.EndDialog();
            return true;
        }
    }
}
