// [Documentation](../../../docs/pascal/tools/debugging.md#private-terminal-and-tui-control)
using System.Buffers.Binary;
using System.Text;
using System.Text.Json;
using Fpas.Debugging.Dap;
using Fpas.Debugging.Hosting;
using Fpas.Processes.Terminal;

namespace Fpas.IntegrationTests.Debugging;

/// <summary>Checks private terminal output transport and VT rendering.</summary>
public sealed class TerminalOutputTests
{
    /// <summary>Checks the default cursor distance and clamps large movements to the screen.</summary>
    /// <param name="direction">The relative VT cursor command.</param>
    /// <param name="defaultRow">The row after the default movement.</param>
    /// <param name="defaultColumn">The column after the default movement.</param>
    /// <param name="edgeRow">The row after clamping a large movement.</param>
    /// <param name="edgeColumn">The column after clamping a large movement.</param>
    [Theory]
    [InlineData('A', 1, 2, 0, 2)]
    [InlineData('B', 3, 2, 4, 2)]
    [InlineData('C', 2, 3, 2, 4)]
    [InlineData('D', 2, 1, 2, 0)]
    public void Relative_cursor_movement_defaults_and_stays_on_screen(char direction, int defaultRow, int defaultColumn, int edgeRow, int edgeColumn)
    {
        Check("", defaultRow, defaultColumn);
        Check("0", defaultRow, defaultColumn);
        Check("1000", edgeRow, edgeColumn);

        void Check(string parameter, int row, int column)
        {
            var display = new TerminalDisplay(5, 5);
            display.Read(Encoding.UTF8.GetBytes($"\u001b[3;3H\u001b[{parameter}{direction}X"));
            var lines = display.Lines();
            Assert.Single(lines, line => line.Contains('X'));
            Assert.Equal('X', lines[row][column]);
        }
    }

    /// <summary>Verifies that output survives a controller read delay longer than the former send deadline.</summary>
    [Fact]
    public async Task Slow_terminal_reader_receives_output_after_catching_up()
    {
        var expected = Encoding.UTF8.GetBytes("terminal output after a slow read");
        await using var terminal = new FakeTerminal(expected);
        await using var stream = new DelayedStream();
        await using var channel = new WorkerChannel(stream);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(10));
        var forwarding = TerminalTarget.ReadOutput(terminal, channel, deadline.Token);
        await stream.Started.Task.WaitAsync(deadline.Token);
        await Task.Delay(TimeSpan.FromMilliseconds(2300), deadline.Token);
        Assert.False(forwarding.IsCompleted);
        stream.Release.TrySetResult();
        await forwarding.WaitAsync(deadline.Token);

        var frame = stream.ToArray();
        Assert.Equal(frame.Length - 4, BinaryPrimitives.ReadInt32BigEndian(frame));
        using var document = JsonDocument.Parse(frame.AsMemory(4));
        Assert.Equal("output", document.RootElement.GetProperty("operation").GetString());
        Assert.Equal(expected, document.RootElement.GetProperty("payload").GetBytesFromBase64());
    }

    /// <summary>Verifies that session shutdown interrupts a stalled terminal output write.</summary>
    [Fact]
    public async Task Cancelling_session_interrupts_blocked_terminal_output()
    {
        await using var terminal = new FakeTerminal([1]);
        await using var stream = new DelayedStream();
        await using var channel = new WorkerChannel(stream);
        using var lifetime = new CancellationTokenSource();
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(10));
        var forwarding = TerminalTarget.ReadOutput(terminal, channel, lifetime.Token);
        await stream.Started.Task.WaitAsync(deadline.Token);
        lifetime.Cancel();
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => forwarding.WaitAsync(deadline.Token));
    }

    private sealed class DelayedStream : MemoryStream
    {
        internal TaskCompletionSource Started { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);
        internal TaskCompletionSource Release { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);

        /// <inheritdoc/>
        public override async ValueTask WriteAsync(ReadOnlyMemory<byte> buffer, CancellationToken cancellationToken = default)
        {
            Started.TrySetResult();
            await Release.Task.WaitAsync(cancellationToken);
            await base.WriteAsync(buffer, cancellationToken);
        }
    }

    private sealed class FakeTerminal(byte[] output) : PseudoTerminal
    {
        /// <inheritdoc/>
        public override Stream Input => Stream.Null;
        /// <inheritdoc/>
        public override Stream Output { get; } = new MemoryStream(output);
        /// <inheritdoc/>
        public override int Id => 0;
        /// <inheritdoc/>
        public override Task<int> Completion => Task.FromResult(0);
        /// <inheritdoc/>
        public override void Resize(int columns, int rows) => throw new NotSupportedException();
        /// <inheritdoc/>
        public override Task TerminateAsync() => Task.CompletedTask;
        /// <inheritdoc/>
        public override async ValueTask DisposeAsync() => await Output.DisposeAsync();
    }
}
