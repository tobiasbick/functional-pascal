using Fpas.Ui.Gui.Cui.Widgets.Desktop;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private readonly Dictionary<string, int> dockRestores = new(StringComparer.Ordinal);

    /// <summary>Applies a native layout once for a model revision before building the workspace.</summary>
    /// <param name="id">Stable workspace identity.</param>
    /// <param name="revision">Application restore revision.</param>
    /// <param name="layout">Saved native layout in dock1 format.</param>
    public void RestoreDockLayout(string id, int revision, string layout)
    {
        CheckFrame();
        if (dockRestores.TryGetValue(id, out int previous) && previous == revision) return;
        Utf8.Validate(layout, nameof(layout));
        NativeMethods.DockspaceLoad(context, Stable(id), layout);
        dockRestores[id] = revision;
    }

    /// <summary>Opens a native workspace. Submit every panel before closing it.</summary>
    /// <param name="id">Stable workspace identity.</param>
    /// <param name="options">Native allocation and layout.</param>
    /// <param name="enabled">Whether to process docking input this frame.</param>
    public void BeginDockspace(string id, CuiDockspaceOptions options, bool enabled)
    {
        CheckFrame();
        CheckExtent(options.WidthKind, options.Width, nameof(options));
        CheckExtent(options.HeightKind, options.Height, nameof(options));
        CheckFeedbackNumber(options.MinimumWidth, nameof(options));
        CheckFeedbackNumber(options.MinimumHeight, nameof(options));
        Utf8.Validate(options.Layout, nameof(options));
        NativeMethods.DockspaceBegin(context, Stable(id), Optional(options.Layout), options.Shortcuts ? 1 : 0,
            options.WidthKind, options.Width, options.HeightKind, options.Height,
            options.MinimumWidth, options.MinimumHeight, enabled ? 1 : 0);
    }

    /// <summary>Submits a panel and opens its content scope when visible.</summary>
    /// <param name="key">Stable panel key in the dock1 layout.</param>
    /// <param name="title">Visible title.</param>
    /// <param name="icon">Optional icon glyph.</param>
    /// <param name="dirty">Whether to show the unsaved-changes dot.</param>
    /// <param name="open">Model-owned panel presence.</param>
    /// <param name="closable">Whether the panel can close.</param>
    /// <param name="trackingId">Optional inspection identity for the tab.</param>
    /// <returns>Panel visibility and proposed presence.</returns>
    public CuiDockPanelResult BeginDockPanel(string key, string title, string icon, bool dirty,
        bool open, bool closable, string? trackingId)
    {
        CheckFrame();
        Utf8.Validate(key, nameof(key));
        Utf8.Validate(title, nameof(title));
        Utf8.Validate(icon, nameof(icon));
        if (key.Length == 0 || key.Contains("##", StringComparison.Ordinal) || Utf8.Encoding.GetByteCount(key) > 63)
            throw new ArgumentException("Expected a dock key of one to 63 UTF-8 bytes without the key delimiter.", nameof(key));
        if (title.Contains("##", StringComparison.Ordinal)
            || Utf8.Encoding.GetByteCount(title) + Utf8.Encoding.GetByteCount(key) > 124
            || Utf8.Encoding.GetByteCount(icon) > 7)
            throw new ArgumentException("Dock panel title or icon exceeds native storage.", nameof(title));
        int state = open || !closable ? 1 : 0;
        bool visible = NativeMethods.DockPanelBegin(context, title + "###" + key, Optional(icon),
            dirty ? 1 : 0, closable ? 1 : 0, ref state, out int node) != 0;
        if (node >= 0 && trackingId is not null) TrackControl(trackingId, node);
        return new(visible, state != 0, node >= 0);
    }

    /// <summary>Closes a visible panel's content scope.</summary>
    public void EndDockPanel()
    {
        CheckFrame();
        NativeMethods.DockPanelEnd(context);
    }

    /// <summary>Closes the workspace after all panels have been submitted.</summary>
    public void EndDockspace()
    {
        CheckFrame();
        NativeMethods.DockspaceEnd(context);
    }

    /// <summary>Serializes the native layout after building the workspace.</summary>
    /// <param name="id">Stable workspace identity.</param>
    /// <returns>Native dock1 layout text, or an empty string before any layout exists.</returns>
    public unsafe string ReadDockLayout(string id)
    {
        CheckFrame();
        string key = Stable(id);
        nuint size = NativeMethods.DockspaceSave(context, key, null, 0);
        if (size == 0) return "";
        byte[] data = new byte[checked((int)size + 1)];
        fixed (byte* buffer = data)
            NativeMethods.DockspaceSave(context, key, buffer, (nuint)data.Length);
        return Utf8.Encoding.GetString(data, 0, (int)size);
    }
}
