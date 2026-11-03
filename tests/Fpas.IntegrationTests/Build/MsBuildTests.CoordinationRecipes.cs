using Fpas.IntegrationTests.Documentation;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Theory]
    [InlineData("Debug")]
    [InlineData("Release")]
    public async Task Documented_coordination_recipes_release_permits_and_join_pipeline_workers(string configuration)
    {
        using var project = sdk.CreateProject();
        var root = Path.Combine(AppContext.BaseDirectory, "Fixtures", "specification", "libraries");
        File.WriteAllText(project.File("Game/Gates.fpas"), MarkdownExamples.Read(Path.Combine(root, "task-coordination.md"))
            .Single(example => example.Library).Source);
        File.WriteAllText(project.File("Game/Bridge.fpas"), MarkdownExamples.Read(Path.Combine(root, "channels.md"))
            .Single(example => example.Library).Source);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Coordination = Recipes.Coordination;
            using Pipeline = Recipes.ChannelPipeline;

            static void Check(bool condition, string message)
            {
                if (!condition) throw new Exception(message);
            }
            static async Task<T> Fails<T>(Func<Task> action) where T : Exception
            {
                try { await action(); }
                catch (T failure) { return failure; }
                throw new Exception("Expected " + typeof(T).Name);
            }
            using (var gate = new SemaphoreSlim(1, 1))
            {
                await gate.WaitAsync();
                using var stop = new CancellationTokenSource();
                var calls = 0;
                var waiting = Coordination.UnderGate<int>(gate, token =>
                {
                    calls++;
                    return Task.FromResult(1);
                }, stop.Token);
                Check(!waiting.IsCompleted && calls == 0, "Work entered an occupied gate");
                stop.Cancel();
                await Fails<OperationCanceledException>(() => waiting.WaitAsync(TimeSpan.FromSeconds(5)));
                Check(calls == 0 && gate.CurrentCount == 0, "Canceled wait released an unacquired permit");
                gate.Release();

                var entered = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
                var release = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
                var holding = Coordination.UnderGate<int>(gate, async token =>
                {
                    entered.SetResult();
                    await release.Task.WaitAsync(token);
                    return 7;
                }, CancellationToken.None);
                Task<int>? second = null;
                try
                {
                    await entered.Task.WaitAsync(TimeSpan.FromSeconds(5));
                    Check(gate.CurrentCount == 0, "Permit released before awaited work finished");
                    second = Coordination.UnderGate<int>(gate, _ => Task.FromResult(9), CancellationToken.None);
                    Check(!second.IsCompleted, "Contending work bypassed the gate");
                    release.TrySetResult();
                    var results = await Task.WhenAll(holding, second).WaitAsync(TimeSpan.FromSeconds(5));
                    Check(results.SequenceEqual(new[] { 7, 9 }) && gate.CurrentCount == 1, "Serialized results or permit changed");
                }
                finally
                {
                    release.TrySetResult();
                    await holding.WaitAsync(TimeSpan.FromSeconds(5));
                    if (second is not null) await second.WaitAsync(TimeSpan.FromSeconds(5));
                }
                var fault = await Fails<InvalidOperationException>(() => Coordination.UnderGate<int>(gate,
                    _ => Task.FromException<int>(new InvalidOperationException("work fault")), CancellationToken.None));
                Check(fault.Message == "work fault" && gate.CurrentCount == 1, "Fault lost its permit or cause");
                using var inside = new CancellationTokenSource();
                var started = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
                var canceledWork = Coordination.UnderGate<int>(gate, async token =>
                {
                    Check(token == inside.Token, "Work did not receive the caller token");
                    started.SetResult();
                    await Task.Delay(Timeout.Infinite, token);
                    return 0;
                }, inside.Token);
                try
                {
                    await started.Task.WaitAsync(TimeSpan.FromSeconds(5));
                    inside.Cancel();
                    await Fails<OperationCanceledException>(() => canceledWork.WaitAsync(TimeSpan.FromSeconds(5)));
                    Check(gate.CurrentCount == 1, "Canceled work kept its acquired permit");
                }
                finally
                {
                    inside.Cancel();
                    try { await canceledWork.WaitAsync(TimeSpan.FromSeconds(5)); }
                    catch (OperationCanceledException) { }
                }
            }

            Check(await Pipeline.Sum(Array.Empty<int>(), 1, CancellationToken.None) == 0, "Empty pipeline did not complete");
            Check(await Pipeline.Sum(Enumerable.Range(1, 1000), 2, CancellationToken.None).WaitAsync(TimeSpan.FromSeconds(5)) == 500500,
                "Bounded pipeline lost values");
            var disposed = 0;
            IEnumerable<int> BrokenProducer()
            {
                try
                {
                    yield return 1;
                    throw new IOException("producer fault");
                }
                finally { Interlocked.Increment(ref disposed); }
            }
            var producerFault = await Fails<IOException>(() => Pipeline.Sum(BrokenProducer(), 1, CancellationToken.None)
                .WaitAsync(TimeSpan.FromSeconds(5)));
            Check(producerFault.Message == "producer fault" && disposed == 1, "Producer failure was not joined");

            var enumerating = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            IEnumerable<int> Endless(int first)
            {
                try
                {
                    enumerating.TrySetResult();
                    yield return first;
                    while (true) yield return 1;
                }
                finally { Interlocked.Increment(ref disposed); }
            }
            await Fails<InvalidDataException>(() => Pipeline.Sum(Endless(-1), 1, CancellationToken.None)
                .WaitAsync(TimeSpan.FromSeconds(5)));
            Check(disposed == 2, "Consumer failure left its producer running");
            await Fails<OverflowException>(() => Pipeline.Sum(new[] { int.MaxValue, 1 }, 1, CancellationToken.None)
                .WaitAsync(TimeSpan.FromSeconds(5)));

            enumerating = new(TaskCreationOptions.RunContinuationsAsynchronously);
            using var cancellation = new CancellationTokenSource();
            var pending = Task.Run(() => Pipeline.Sum(Endless(1), 1, cancellation.Token));
            try
            {
                await enumerating.Task.WaitAsync(TimeSpan.FromSeconds(5));
                cancellation.Cancel();
                await Fails<OperationCanceledException>(() => pending.WaitAsync(TimeSpan.FromSeconds(5)));
                Check(disposed == 3, "Canceled pipeline returned before its producer stopped");
            }
            finally
            {
                cancellation.Cancel();
                try { await pending.WaitAsync(TimeSpan.FromSeconds(5)); }
                catch (OperationCanceledException) { }
            }
            await Fails<OperationCanceledException>(() => Pipeline.Sum(BrokenProducer(), 1, cancellation.Token));
            Check(disposed == 3, "Pre-canceled pipeline enumerated input");
            Console.WriteLine("Coordination recipes verified");
            """);
        var build = await sdk.Run(project, "build", "Game.slnx", "--configuration", configuration, "-warnaserror");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await ProcessRunner.RunAsync([project.File($"Host/bin/{configuration}/net10.0/Host.dll")],
            workingDirectory: project.Path, timeLimit: TimeSpan.FromSeconds(40));
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("Coordination recipes verified" + Environment.NewLine, run.Output);
        Assert.Empty(run.Error);
    }
}
