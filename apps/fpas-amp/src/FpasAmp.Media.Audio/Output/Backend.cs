// [Documentation](../../../doc/media/audio.md#output-devices)
using System.Runtime.InteropServices;
using SoundFlow.Backends.MiniAudio;
using SoundFlow.Backends.MiniAudio.Enums;

namespace FpasAmp.Media.Audio.Output;

internal static class Backend
{
    // SoundFlow 1.4.2 casts its managed enum directly to a different native enum.
    [DllImport("miniaudio", EntryPoint = "ma_get_backend_from_name", CallingConvention = CallingConvention.Cdecl)]
    private static extern int Resolve([MarshalAs(UnmanagedType.LPUTF8Str)] string name, out MiniAudioBackend backend);

    internal static MiniAudioEngine Open(bool silent)
    {
        string[] names = silent ? ["Null"] : OperatingSystem.IsWindows() ? ["WASAPI"]
            : OperatingSystem.IsLinux() ? ["PulseAudio", "ALSA"] : throw new PlatformNotSupportedException();
        return new MiniAudioEngine(names.Select(name => Resolve(name, out var backend) == 0
            ? backend : throw new NotSupportedException($"Unknown miniaudio backend '{name}'.")));
    }
}
