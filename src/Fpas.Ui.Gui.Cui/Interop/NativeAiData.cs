using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativePlanStep
{
    public byte* Title;
    public byte* Detail;
    public int Status;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeSource
{
    public byte* Title;
    public byte* Url;
    public byte* Snippet;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeSuggestion
{
    public byte* Icon;
    public byte* Title;
    public byte* Detail;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeModel
{
    public byte* Name;
    public byte* Detail;
    public byte* Badge;
    public byte* Icon;
}
