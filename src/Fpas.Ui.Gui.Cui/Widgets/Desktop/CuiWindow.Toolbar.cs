// [Documentation](../../../../docs/ui/gui/canvas-and-bars/toolbar.md)
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Opens a toolbar, closed by EndToolbar after its tools and groups.</summary>
    /// <param name="id">Stable toolbar identity.</param>
    /// <param name="style">Plain strip=0, grouped pill=1, floating pill=2.</param>
    /// <param name="noBorder">Whether to omit the plain strip's bottom hairline.</param>
    /// <param name="enabled">Whether tools accept input.</param>
    public unsafe void BeginToolbar(string id, int style, bool noBorder, bool enabled)
    {
        CheckFrame();
        NativeMethods.ToolbarBegin(context, Stable(id), style, noBorder ? 1 : 0, enabled ? 1 : 0);
        layouts.Add(new(Layout.Toolbar));
    }

    /// <summary>Closes the innermost toolbar, builds its overflow menu and makes its root the last widget.</summary>
    public unsafe void EndToolbar()
    {
        RequireLayout(Layout.Toolbar);
        NativeMethods.ToolbarEnd(context);
        layouts.RemoveAt(layouts.Count - 1);
    }

    /// <summary>Adds an action or toggle to the current toolbar.</summary>
    /// <param name="icon">Optional CUI icon glyph, copied for deferred overflow construction.</param>
    /// <param name="label">Optional visible caption.</param>
    /// <param name="tooltip">Optional hover caption.</param>
    /// <param name="shortcut">Optional shortcut hint.</param>
    /// <param name="toggle">Minus one for an action, zero for an unchecked toggle, one for a checked toggle.</param>
    /// <param name="tone">Native semantic tone ordinal.</param>
    /// <param name="flags">Bits: 1 disables the tool, 2 shows a flyout marker, 4 draws a primary action.</param>
    /// <returns>Minus one when idle; otherwise the proposed toggle value, or zero for an activated action.</returns>
    public unsafe int ToolbarButton(string? icon, string? label, string? tooltip, string? shortcut, int toggle, int tone, int flags)
    {
        CheckFrame();
        return NativeMethods.ToolbarButton(context, icon, label, tooltip, shortcut, toggle, tone, flags);
    }

    /// <summary>Adds an exclusive choice of icon tools to the current toolbar.</summary>
    /// <param name="id">Stable choice identity.</param>
    /// <param name="icons">Option glyphs; CUI uses at most the first 32 entries.</param>
    /// <param name="labels">One tooltip per icon.</param>
    /// <param name="value">Current zero-based selection.</param>
    /// <returns>The proposed index, or minus one when unchanged or empty.</returns>
    public unsafe int ToolbarRadio(string id, string[] icons, string[] labels, int value)
    {
        CheckFrame();
        if (icons.Length != labels.Length) throw new ArgumentException("A radio option needs one label per icon.", nameof(labels));
        using var memory = new ChartMemory();
        return NativeMethods.ToolbarRadio(context, Stable(id), memory.Labels(icons), memory.Labels(labels), icons.Length, value);
    }

    /// <summary>Adds a tool that cycles through its captions when activated.</summary>
    /// <param name="id">Stable tool identity.</param>
    /// <param name="labels">Available captions in cycle order.</param>
    /// <param name="value">Current zero-based selection.</param>
    /// <param name="tooltip">Hover caption.</param>
    /// <returns>The proposed index, or minus one when unchanged.</returns>
    public unsafe int ToolbarCycle(string id, string[] labels, int value, string tooltip)
    {
        CheckFrame();
        using var memory = new ChartMemory();
        return NativeMethods.ToolbarCycle(context, Stable(id), memory.Labels(labels), labels.Length, value, tooltip);
    }

    /// <summary>Opens a native tool group; pair with EndToolbarGroup before closing the toolbar.</summary>
    public unsafe void BeginToolbarGroup()
    {
        CheckFrame();
        NativeMethods.ToolbarGroupBegin(context);
    }

    /// <summary>Closes the current native tool group.</summary>
    public unsafe void EndToolbarGroup()
    {
        CheckFrame();
        NativeMethods.ToolbarGroupEnd(context);
    }

    /// <summary>Adds a divider between tools in the current toolbar.</summary>
    public unsafe void ToolbarSeparator()
    {
        CheckFrame();
        NativeMethods.ToolbarSeparator(context);
    }

    /// <summary>Pushes subsequent tools to the far side of the current toolbar.</summary>
    public unsafe void ToolbarSpacer()
    {
        CheckFrame();
        NativeMethods.ToolbarSpacer(context);
    }
}
