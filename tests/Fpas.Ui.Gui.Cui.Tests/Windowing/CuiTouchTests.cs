// [Documentation](../../../docs/ui/gui/cui.md#touch-input)
using Fpas.Ui.Gui.Cui.Widgets.Canvas;

namespace Fpas.Ui.Gui.Cui.Tests.Windowing;

/// <summary>Checks raw multi-touch input through CUI's touch API.</summary>
public sealed class CuiTouchTests
{
    /// <summary>Tracks landing, movement, the first touch, lifting and cancellation per frame.</summary>
    [GuiFact]
    public void TouchesKeepPerFrameStateAndLandingOrder()
    {
        using var window = new CuiWindow("Touch input", 320, 200, CuiRenderer.Software, continuous: true);

        Frame(() =>
        {
            window.Touch(0, true, 10, 20);
            Assert.Equal(new CuiTouch(true, true, false, new CuiPoint(10, 20)), window.ReadTouch(0));
            Assert.Equal(1, window.TouchCount);
        });
        Frame(() =>
        {
            window.MoveTouch(0, 30, 40);
            window.Touch(1, true, 50, 60);
            Assert.Equal(new CuiTouch(true, false, false, new CuiPoint(30, 40)), window.ReadTouch());
            Assert.True(window.ReadTouch(1).Pressed);
            Assert.Equal(2, window.TouchCount);
        });
        Frame(() =>
        {
            window.Touch(0, false, 32, 42);
            Assert.Equal(new CuiTouch(false, false, true, new CuiPoint(32, 42)), window.ReadTouch(0));
            Assert.Equal(new CuiPoint(50, 60), window.ReadTouch().Position);
            Assert.Equal(1, window.TouchCount);
        });
        Frame(() =>
        {
            window.CancelTouch(1);
            Assert.True(window.ReadTouch(1).Released);
            Assert.Equal(0, window.TouchCount);
        });
        Frame(() =>
        {
            Assert.Equal(new CuiTouch(false, false, false, null), window.ReadTouch(1));
            Assert.Throws<ArgumentOutOfRangeException>(() => window.Touch(CuiWindow.TouchSlots, true, 0, 0));
            Assert.Throws<ArgumentOutOfRangeException>(() => window.MoveTouch(0, float.NaN, 0));
            Assert.Throws<ArgumentOutOfRangeException>(() => window.ReadTouch(-2));
        });

        void Frame(Action body)
        {
            Assert.True(window.NextFrame());
            body();
            window.Text("Touch");
            window.Present();
        }
    }
}
