// [Documentation](../../doc/media/audio.md#mixer-settings)
using System.Collections.Immutable;

namespace FpasAmp.Media.Audio;

/// <summary>Immutable volume and equalizer targets in decibels.</summary>
public sealed record Settings
{
    /// <summary>Centre frequencies in Hz, ordered from bass to treble.</summary>
    public static ImmutableArray<int> Frequencies { get; } = [31, 62, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];
    /// <summary>Unity output volume and flat equalization.</summary>
    public static Settings Flat { get; } = new(0, new float[Frequencies.Length]);
    /// <summary>Output gain between -60 and 0 dB.</summary>
    public float Volume { get; }
    /// <summary>Gain between -12 and +12 dB for each entry in Frequencies.</summary>
    public ImmutableArray<float> Gains { get; }

    /// <summary>Copies and validates one set of mixer targets.</summary>
    /// <param name="volume">Finite output gain between -60 and 0 dB.</param>
    /// <param name="gains">Ten finite gains between -12 and +12 dB, in frequency order.</param>
    public Settings(float volume, float[] gains)
    {
        ArgumentNullException.ThrowIfNull(gains);
        if (!float.IsFinite(volume) || volume is < -60 or > 0)
            throw new ArgumentOutOfRangeException(nameof(volume));
        Gains = [.. gains];
        if (Gains.Length != Frequencies.Length || Gains.Any(g => !float.IsFinite(g) || g is < -12 or > 12))
            throw new ArgumentException("Expected ten gains between -12 and +12 dB.", nameof(gains));
        Volume = volume;
    }
}
