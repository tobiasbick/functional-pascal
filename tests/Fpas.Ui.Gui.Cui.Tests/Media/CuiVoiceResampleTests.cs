// [Documentation](../../../docs/ui/gui/voice/voice-waveform.md)
namespace Fpas.Ui.Gui.Cui.Tests.Media;

/// <summary>Checks native resampling without a window.</summary>
public sealed class CuiVoiceResampleTests
{
    /// <summary>Preserves peaks and silence when the number of columns changes.</summary>
    [Fact]
    public void ResampleUsesNativePeakBuckets()
    {
        Assert.Equal([-1f, .7f, .4f], CuiWindow.ResampleVoice([-1, float.NaN, .4f, .7f, 0, .4f], 3));
        Assert.Equal([.2f, .2f, .8f, .8f], CuiWindow.ResampleVoice([.2f, .8f], 4));
        Assert.Equal([-1f, -1f], CuiWindow.ResampleVoice([], 2));
        Assert.Empty(CuiWindow.ResampleVoice([.4f], 0));
    }
}
