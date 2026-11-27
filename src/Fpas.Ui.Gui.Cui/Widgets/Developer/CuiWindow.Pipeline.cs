// [Documentation](../../../../docs/ui/gui/developer-tools/pipeline.md)
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native Pipeline widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Stages">Stages supplied to CUI.</param>
    /// <param name="Jobs">Jobs supplied to CUI.</param>
    /// <param name="Selection">Selected index, minus one for no selection, or null to omit selection styling.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <param name="Focus">Whether to focus the selected job or first input job.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult Pipeline(string Id, string[] Stages, CuiDevJob[] Jobs, int? Selection, int WidthKind, float Width,
        bool Enabled, bool Focus = false)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        if (Selection is { } selected && (selected < -1 || selected >= Jobs.Length))
            throw new ArgumentOutOfRangeException(nameof(Selection));
        foreach (string stage in Stages) Utf8.Validate(stage, nameof(Stages));
        foreach (var job in Jobs)
        {
            Utf8.Validate(job.Name, nameof(Jobs));
            if (job.Stage < 0 || job.Stage >= Stages.Length || job.Status is < 0 or > 5 || !float.IsFinite(job.Duration))
                throw new ArgumentOutOfRangeException(nameof(Jobs));
        }
        using var memory = new ChartMemory();
        int flags = (Enabled ? 1 : 0) | (Focus ? 2 : 0);
        NativeMethods.DevPipeline(context, Stable(Id), memory.Labels(Stages), Stages.Length, PackJob(memory, Jobs), Jobs.Length,
            Selection ?? -2, WidthKind, Width, flags, out var result);
        return result.ToManaged();
    }
}
