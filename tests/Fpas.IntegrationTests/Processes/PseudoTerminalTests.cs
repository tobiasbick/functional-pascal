// [Documentation](../../../docs/pascal/libraries/processes.md)
using Fpas.Processes.Terminal;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Processes;

/// <summary>Checks isolation between private terminal sessions.</summary>
public sealed class PseudoTerminalTests
{
    /// <summary>Verifies that a new Linux target cannot access an earlier terminal's masters.</summary>
    [Fact]
    public async Task Linux_terminal_does_not_inherit_the_master_descriptors_of_another_session()
    {
        if (!OperatingSystem.IsLinux()) return;

        using var fixture = new TemporaryDirectory();
        await using var first = PseudoTerminal.Start("/bin/sh", ["-c", "exec sleep 60"], fixture.Path);
        var descriptors = fixture.File("descriptors.txt");
        await using var second = PseudoTerminal.Start("/bin/sh",
            ["-c", "exec ls -l /proc/self/fd > \"$1\"", "sh", descriptors], fixture.Path);

        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(10));
        Assert.Equal(0, await second.Completion.WaitAsync(deadline.Token));
        var listing = await File.ReadAllTextAsync(descriptors, deadline.Token);
        Assert.Contains("/dev/pts/", listing);
        Assert.DoesNotContain("/ptmx", listing);
    }

    /// <summary>Verifies that Linux targets launched concurrently each open their own terminal slave.</summary>
    [Fact]
    public async Task Concurrent_linux_terminal_launches_use_distinct_slave_devices()
    {
        if (!OperatingSystem.IsLinux()) return;

        using var fixture = new TemporaryDirectory();
        var terminals = new PseudoTerminal?[16];
        var paths = Enumerable.Range(0, terminals.Length).Select(index => fixture.File($"terminal-{index}.txt")).ToArray();
        try
        {
            await Task.WhenAll(paths.Select((path, index) => Task.Run(() =>
            {
                terminals[index] = PseudoTerminal.Start("/bin/sh",
                    ["-c", "exec tty > \"$1\"", "sh", path], fixture.Path);
            })));

            using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(10));
            foreach (var terminal in terminals) Assert.Equal(0, await terminal!.Completion.WaitAsync(deadline.Token));
            var slaves = await Task.WhenAll(paths.Select(path => File.ReadAllTextAsync(path, deadline.Token)));
            Assert.All(slaves, slave => Assert.StartsWith("/dev/pts/", slave));
            Assert.Equal(slaves.Length, slaves.Distinct(StringComparer.Ordinal).Count());
        }
        finally
        {
            await Task.WhenAll(terminals.OfType<PseudoTerminal>().Select(terminal => terminal.DisposeAsync().AsTask()));
        }
    }
}
