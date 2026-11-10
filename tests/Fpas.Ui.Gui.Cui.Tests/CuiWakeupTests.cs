namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks wakeup of the real idle platform loop and its lifetime during shutdown.</summary>
public sealed class CuiWakeupTests
{
    /// <summary>Wakes a noncontinuous window from a worker and posts while its owner disposes it.</summary>
    [GuiFact]
    [Trait("Category", "NativeGui")]
    public async Task WorkerWakesAnIdleFrameAndCannotWakeADisposedWindow()
    {
        var timeout = TimeSpan.FromSeconds(10);
        var ready = new TaskCompletionSource<CuiWindow>(TaskCreationOptions.RunContinuationsAsynchronously);
        var resumed = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        var stopped = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        int frames = 0, observeWake = 0, close = 0;
        var owner = new Thread(() =>
        {
            try
            {
                using (var window = new CuiWindow("FPAS idle wakeup", renderer: CuiRenderer.Software))
                {
                    window.SetReducedMotion(true);
                    ready.SetResult(window);
                    while (window.NextFrame() && Volatile.Read(ref close) == 0)
                    {
                        Interlocked.Increment(ref frames);
                        if (Volatile.Read(ref observeWake) != 0) resumed.TrySetResult();
                        window.Text("Idle view");
                    }
                }
                stopped.SetResult();
            }
            catch (Exception failure)
            {
                ready.TrySetException(failure);
                resumed.TrySetException(failure);
                stopped.TrySetException(failure);
            }
        }) { IsBackground = true };
        owner.Start();
        CuiWindow window = await ready.Task.WaitAsync(timeout);
        try
        {
            using var deadline = new CancellationTokenSource(timeout);
            int before;
            do
            {
                before = Volatile.Read(ref frames);
                await Task.Delay(150, deadline.Token);
            }
            while (before == 0 || before != Volatile.Read(ref frames));

            Volatile.Write(ref observeWake, 1);
            Assert.True(window.TryWake());
            await resumed.Task.WaitAsync(timeout);

            var posting = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            var poster = Task.Run(async () =>
            {
                while (!stopped.Task.IsCompleted && !deadline.IsCancellationRequested)
                {
                    if (window.TryWake()) posting.TrySetResult();
                    await Task.Yield();
                }
            });
            await posting.Task.WaitAsync(timeout);
            Volatile.Write(ref close, 1);
            window.TryWake();
            await stopped.Task.WaitAsync(timeout);
            await poster.WaitAsync(timeout);
            Assert.False(window.TryWake());
        }
        finally
        {
            Volatile.Write(ref close, 1);
            window.TryWake();
        }
    }
}
