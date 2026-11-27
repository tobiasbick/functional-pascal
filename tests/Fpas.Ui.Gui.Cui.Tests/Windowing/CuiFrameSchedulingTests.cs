// [Documentation](../../../docs/ui/gui/host.md)
namespace Fpas.Ui.Gui.Cui.Tests.Windowing;

/// <summary>Checks CUI deadlines in the real event-driven native window loop.</summary>
public sealed class CuiFrameSchedulingTests
{
    /// <summary>Requests one extra frame from an idle native owner.</summary>
    /// <param name="delay">Zero requests an immediate frame; a positive value selects a deadline.</param>
    [GuiTheory]
    [InlineData(0.0f)]
    [InlineData(.05f)]
    public async Task FrameRequestWakesAnIdleWindowOnce(float delay)
    {
        var timeout = TimeSpan.FromSeconds(10);
        var ready = new TaskCompletionSource<CuiWindow>(TaskCreationOptions.RunContinuationsAsynchronously);
        var deadlineFrame = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        var stopped = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        int stage = 0, frames = 0, close = 0;
        var owner = new Thread(() =>
        {
            try
            {
                using var window = new CuiWindow("Native frame deadlines", renderer: CuiRenderer.Software);
                window.SetReducedMotion(true);
                ready.SetResult(window);
                while (window.NextFrame() && Volatile.Read(ref close) == 0)
                {
                    Interlocked.Increment(ref frames);
                    window.Text("An idle native view");
                    if (Volatile.Read(ref stage) == 1)
                    {
                        window.RequestFrame(30);
                        window.RequestFrame(delay);
                        Volatile.Write(ref stage, 2);
                    }
                    else if (Volatile.Read(ref stage) == 2)
                    {
                        Volatile.Write(ref stage, 3);
                        deadlineFrame.TrySetResult();
                    }
                    window.Present();
                }
                stopped.SetResult();
            }
            catch (Exception failure)
            {
                ready.TrySetException(failure);
                deadlineFrame.TrySetException(failure);
                stopped.TrySetException(failure);
            }
        }) { IsBackground = true };
        owner.Start();
        CuiWindow window = await ready.Task.WaitAsync(timeout);
        try
        {
            using var cancellation = new CancellationTokenSource(timeout);
            int before;
            do
            {
                before = Volatile.Read(ref frames);
                await Task.Delay(150, cancellation.Token);
            } while (before == 0 || before != Volatile.Read(ref frames));
            Volatile.Write(ref stage, 1);
            Assert.True(window.TryWake());
            await deadlineFrame.Task.WaitAsync(timeout);
            int after = Volatile.Read(ref frames);
            await Task.Delay(200, cancellation.Token);
            Assert.Equal(after, Volatile.Read(ref frames));
        }
        finally
        {
            Volatile.Write(ref close, 1);
            window.TryWake();
            await stopped.Task.WaitAsync(timeout);
            Assert.True(owner.Join(timeout));
        }
    }
}
