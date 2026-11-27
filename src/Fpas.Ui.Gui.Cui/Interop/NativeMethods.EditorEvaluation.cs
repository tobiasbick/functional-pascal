// [Documentation](../../../docs/ui/gui/editing-tools/README.md)
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Widgets.Editors;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "cui_curve_eval")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial float EvaluateCurve(CuiCurveKey* keys, int count, float x);

    [LibraryImport(Library, EntryPoint = "fpas_cui_gradient_eval")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial uint EvaluateGradient(CuiGradientStop* stops, int count, float position);
}
