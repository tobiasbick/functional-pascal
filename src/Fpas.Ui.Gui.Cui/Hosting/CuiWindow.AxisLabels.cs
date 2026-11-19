// [Documentation](../../../docs/ui/gui/charts/README.md)
using System.Runtime.CompilerServices;
using System.Runtime.ExceptionServices;
using System.Runtime.InteropServices;
using System.Text;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    // CUI invokes label callbacks while painting, after the widget call has returned.
    private readonly List<GCHandle> axisLabels = [];
    private bool formattingAxisLabel;

    private sealed class AxisLabel(CuiWindow owner, Func<double, string> format)
    {
        internal readonly CuiWindow Owner = owner;
        internal readonly Func<double, string> Format = format;
        internal ExceptionDispatchInfo? Failure;
    }

    private nint RetainAxisLabel(Func<double, string>? format)
    {
        if (format is null) return 0;
        var root = GCHandle.Alloc(new AxisLabel(this, format));
        axisLabels.Add(root);
        return GCHandle.ToIntPtr(root);
    }

    private ExceptionDispatchInfo? ReleaseAxisLabels()
    {
        ExceptionDispatchInfo? failure = null;
        foreach (var root in axisLabels)
        {
            failure ??= ((AxisLabel)root.Target!).Failure;
            root.Free();
        }
        axisLabels.Clear();
        return failure;
    }

    private static unsafe nint AxisLabelFunction(nint user) => user == 0 ? 0 :
        (nint)(delegate* unmanaged[Cdecl]<nint, double, byte*, nuint, void>)&WriteAxisLabel;

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static unsafe void WriteAxisLabel(nint user, double value, byte* output, nuint capacity)
    {
        if (output == null || capacity == 0) return;
        output[0] = 0;
        var callback = (AxisLabel)GCHandle.FromIntPtr(user).Target!;
        if (callback.Failure is not null) return;
        try
        {
            callback.Owner.formattingAxisLabel = true;
            string label = callback.Format(value);
            Utf8.Validate(label, nameof(label));
            var buffer = new Span<byte>(output, checked((int)capacity) - 1);
            int written = 0;
            foreach (Rune rune in label.EnumerateRunes())
            {
                if (rune.Utf8SequenceLength > buffer.Length - written) break;
                written += rune.EncodeToUtf8(buffer[written..]);
            }
            output[written] = 0;
        }
        catch (Exception error)
        {
            // Managed exceptions must never unwind through CUI's C frames.
            callback.Failure = ExceptionDispatchInfo.Capture(error);
        }
        finally { callback.Owner.formattingAxisLabel = false; }
    }
}
