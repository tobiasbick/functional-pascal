// [Documentation](../../../doc/media/audio.md)
namespace FpasAmp.Media.Audio.Tests.Support;

internal static class Wave
{
    internal static void Write(string path, int frames, Func<int, (short Left, short Right)> sample)
    {
        using var writer = new BinaryWriter(File.Create(path));
        writer.Write("RIFF"u8);
        writer.Write(36 + frames * 4);
        writer.Write("WAVEfmt "u8);
        writer.Write(16);
        writer.Write((short)1);
        writer.Write((short)2);
        writer.Write(44100);
        writer.Write(44100 * 4);
        writer.Write((short)4);
        writer.Write((short)16);
        writer.Write("data"u8);
        writer.Write(frames * 4);
        for (int frame = 0; frame < frames; frame++)
        {
            var (left, right) = sample(frame);
            writer.Write(left);
            writer.Write(right);
        }
    }
}
