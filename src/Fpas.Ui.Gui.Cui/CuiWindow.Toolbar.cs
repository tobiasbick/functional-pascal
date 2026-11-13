using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Opens the native BeginToolbar operation.</summary>
    /// <param name="id">id supplied to the native operation.</param>
    /// <param name="style">style supplied to the native operation.</param>
    /// <param name="noBorder">noBorder supplied to the native operation.</param>
    /// <param name="enabled">enabled supplied to the native operation.</param>
    public unsafe void BeginToolbar(string id, int style, bool noBorder, bool enabled)
    {
        CheckFrame();
        NativeMethods.ToolbarBegin(context, Stable(id), style, noBorder ? 1 : 0, enabled ? 1 : 0);
        layouts.Add(new(Layout.Toolbar));
    }

    /// <summary>Closes the native EndToolbar operation.</summary>
    public unsafe void EndToolbar()
    {
        RequireLayout(Layout.Toolbar);
        NativeMethods.ToolbarEnd(context);
        layouts.RemoveAt(layouts.Count - 1);
    }

    /// <summary>Invokes the native ToolbarButton operation.</summary>
    /// <param name="icon">icon supplied to the native operation.</param>
    /// <param name="label">label supplied to the native operation.</param>
    /// <param name="tooltip">tooltip supplied to the native operation.</param>
    /// <param name="shortcut">shortcut supplied to the native operation.</param>
    /// <param name="toggle">toggle supplied to the native operation.</param>
    /// <param name="tone">tone supplied to the native operation.</param>
    /// <param name="flags">flags supplied to the native operation.</param>
    /// <returns>The native result.</returns>
    public unsafe int ToolbarButton(string? icon, string? label, string? tooltip, string? shortcut, int toggle, int tone, int flags)
    {
        CheckFrame();
        return NativeMethods.ToolbarButton(context, icon, label, tooltip, shortcut, toggle, tone, flags);
    }

    /// <summary>Invokes the native ToolbarRadio operation.</summary>
    /// <param name="id">id supplied to the native operation.</param>
    /// <param name="icons">icons supplied to the native operation.</param>
    /// <param name="labels">labels supplied to the native operation.</param>
    /// <param name="value">value supplied to the native operation.</param>
    /// <returns>The native result.</returns>
    public unsafe int ToolbarRadio(string id, string[] icons, string[] labels, int value)
    {
        CheckFrame();
        if (icons.Length != labels.Length) throw new ArgumentException("A radio option needs one label per icon.", nameof(labels));
        using var memory = new ChartMemory();
        return NativeMethods.ToolbarRadio(context, Stable(id), memory.Labels(icons), memory.Labels(labels), icons.Length, value);
    }

    /// <summary>Invokes the native ToolbarCycle operation.</summary>
    /// <param name="id">id supplied to the native operation.</param>
    /// <param name="labels">labels supplied to the native operation.</param>
    /// <param name="value">value supplied to the native operation.</param>
    /// <param name="tooltip">tooltip supplied to the native operation.</param>
    /// <returns>The native result.</returns>
    public unsafe int ToolbarCycle(string id, string[] labels, int value, string tooltip)
    {
        CheckFrame();
        using var memory = new ChartMemory();
        return NativeMethods.ToolbarCycle(context, Stable(id), memory.Labels(labels), labels.Length, value, tooltip);
    }

    /// <summary>Opens the native BeginToolbarGroup operation.</summary>
    public unsafe void BeginToolbarGroup()
    {
        CheckFrame();
        NativeMethods.ToolbarGroupBegin(context);
    }

    /// <summary>Closes the native EndToolbarGroup operation.</summary>
    public unsafe void EndToolbarGroup()
    {
        CheckFrame();
        NativeMethods.ToolbarGroupEnd(context);
    }

    /// <summary>Invokes the native ToolbarSeparator operation.</summary>
    public unsafe void ToolbarSeparator()
    {
        CheckFrame();
        NativeMethods.ToolbarSeparator(context);
    }

    /// <summary>Invokes the native ToolbarSpacer operation.</summary>
    public unsafe void ToolbarSpacer()
    {
        CheckFrame();
        NativeMethods.ToolbarSpacer(context);
    }
}
