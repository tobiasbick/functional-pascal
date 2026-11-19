// [Documentation](../coverage.md#native-gui)
using System.Runtime.InteropServices;
using Fpas.Tests.Support;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks that widgets copy call-scoped caller data before CUI paints a frame.</summary>
internal static class DeferredPaint
{
    /// <summary>Requires an identical capture after the caller's inputs change during the final frame.</summary>
    /// <param name="window">Window that presents the frames.</param>
    /// <param name="draw">Draws one frame; its argument requests mutation of all caller inputs before presentation.</param>
    internal static void AssertInputsCopied(CuiWindow window, Action<bool> draw)
    {
        using var directory = new TemporaryDirectory();
        string before = directory.File("before.png"), after = directory.File("after.png");
        for (int frame = 0; frame < 5; frame++) draw(false);
        window.CapturePng(before);
        draw(true);
        window.CapturePng(after);
        Assert.Equal(File.ReadAllBytes(before), File.ReadAllBytes(after));
    }

    /// <summary>Closes the root column and presents after reusing and overwriting freed native buffers.</summary>
    /// <param name="window">Window whose open frame is presented.</param>
    internal static void PresentOverFreedBuffers(CuiWindow window)
    {
        var blocks = new List<nint>();
        try
        {
            for (int bytes = 8; bytes <= 256; bytes += 8)
                for (int count = 0; count < 4; count++)
                {
                    nint block = Marshal.AllocHGlobal(bytes);
                    blocks.Add(block);
                    Marshal.Copy(Enumerable.Repeat((byte)'Z', bytes).ToArray(), 0, block, bytes);
                }
            window.EndColumn();
            GC.Collect();
            window.Present();
        }
        finally { foreach (nint block in blocks) Marshal.FreeHGlobal(block); }
    }
}
