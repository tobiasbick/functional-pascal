// [Documentation](../../../docs/ui/gui/app-chrome/window.md)
using System.Diagnostics;

namespace Fpas.Ui.Gui.Cui.Tests.Windowing;

/// <summary>Exercises window placement and state through the native binding.</summary>
public sealed class CuiPlacementTests
{
    /// <summary>Opens at a saved place, moves, and keeps the normal bounds while maximized.</summary>
    [GuiFact]
    [Trait("Category", "NativeGui")]
    public void PlacementOpensMovesAndOutlastsAMaximize()
    {
        var saved = new CuiWindowPlacement { X = 120, Y = 140, Width = 420, Height = 300, HasPosition = true };
        using var window = new CuiWindow("Placement", 640, 480, CuiRenderer.Software, continuous: true, placement: saved);
        Settle(window, () => window.Placement.Width == 420);
        var opened = window.Placement;
        Assert.Equal((420, 300, CuiWindowState.Normal), (opened.Width, opened.Height, opened.State));
        if (opened.HasPosition) Assert.Equal((120, 140), (opened.X, opened.Y));

        window.MoveTo(200, 180);
        Settle(window, () => !window.Placement.HasPosition || window.Placement.X == 200);
        var moved = window.Placement;
        if (moved.HasPosition) Assert.Equal((200, 180), (moved.X, moved.Y));

        // Window managers carry out state changes on X11 in their own time, and Xvfb runs none.
        if (!OperatingSystem.IsWindows()) return;
        window.Maximize();
        Settle(window, () => window.State == CuiWindowState.Maximized);
        var maximized = window.Placement;
        Assert.Equal((200, 180, 420, 300, CuiWindowState.Maximized),
            (maximized.X, maximized.Y, maximized.Width, maximized.Height, maximized.State));
        window.Minimize();
        Settle(window, () => window.State == CuiWindowState.Minimized);
        Assert.Equal(CuiWindowState.Maximized, window.Placement.State);
        window.Restore();
        Settle(window, () => window.State == CuiWindowState.Maximized);
        window.Restore();
        Settle(window, () => window.State == CuiWindowState.Normal);
        Assert.Equal(420f, window.Size.Width);
        Assert.Equal(300f, window.Size.Height);
    }

    /// <summary>A placement with an undefined state is rejected before a window opens.</summary>
    [Fact]
    public void UndefinedPlacementStateIsRejected()
    {
        var invalid = new CuiWindowPlacement { Width = 320, Height = 240, State = (CuiWindowState)7 };
        Assert.Throws<ArgumentOutOfRangeException>(() => new CuiWindow("Invalid", placement: invalid));
    }

    private static void Settle(CuiWindow window, Func<bool> done)
    {
        var deadline = Stopwatch.StartNew();
        do
        {
            Assert.True(window.NextFrame());
            window.BeginColumn();
            window.Heading("Placement");
            window.EndColumn();
        }
        while (!done() && deadline.Elapsed < TimeSpan.FromSeconds(3));
        for (int frame = 0; frame < 5; frame++)
        {
            Assert.True(window.NextFrame());
            window.BeginColumn();
            window.Heading("Placement");
            window.EndColumn();
        }
    }
}
