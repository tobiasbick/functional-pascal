// [Documentation](../../../../docs/ui/gui/canvas-and-bars/live-cursors.md)
namespace Fpas.Ui.Gui.Cui.Widgets.Canvas;

/// <summary>A collaborator shown in a canvas or presence pill.</summary>
/// <param name="Name">Display name.</param>
/// <param name="Tone">Fallback theme color.</param>
/// <param name="X">Horizontal canvas position.</param>
/// <param name="Y">Vertical canvas position.</param>
/// <param name="Says">Optional cursor speech bubble.</param>
/// <param name="Speaking">Whether to show a speaking indicator.</param>
/// <param name="Away">Whether to hide the cursor and dim the avatar.</param>
/// <param name="Rgba">Explicit color, or zero to use the tone.</param>
public sealed record CuiLivePerson(string Name, int Tone, float X, float Y, string? Says, bool Speaking, bool Away, uint Rgba)
{
    /// <summary>Creates a collaborator with a presence pill avatar.</summary>
    /// <param name="name">Display name.</param>
    /// <param name="tone">Fallback theme color.</param>
    /// <param name="x">Horizontal canvas position.</param>
    /// <param name="y">Vertical canvas position.</param>
    /// <param name="says">Optional cursor speech bubble.</param>
    /// <param name="speaking">Whether to show a speaking indicator.</param>
    /// <param name="away">Whether to hide the cursor and dim the avatar.</param>
    /// <param name="rgba">Explicit color, or zero to use the tone.</param>
    /// <param name="avatar">PNG or JPEG path, or an empty string to show initials.</param>
    public CuiLivePerson(string name, int tone, float x, float y, string? says, bool speaking, bool away,
        uint rgba, string avatar) : this(name, tone, x, y, says, speaking, away, rgba) => Avatar = avatar;

    /// <summary>Gets the PNG or JPEG path, or an empty string to show initials.</summary>
    public string Avatar { get; init; } = "";
}
