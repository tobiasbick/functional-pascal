// [Documentation](../../../../docs/ui/gui/ai/README.md)
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop.Ai;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativePlanStep
{
    internal byte* Title;
    internal byte* Detail;
    internal int Status;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeSource
{
    internal byte* Title;
    internal byte* Url;
    internal byte* Snippet;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeSuggestion
{
    internal byte* Icon;
    internal byte* Title;
    internal byte* Detail;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeModel
{
    internal byte* Name;
    internal byte* Detail;
    internal byte* Badge;
    internal byte* Icon;
}
