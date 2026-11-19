// [Documentation](../../../docs/ui/gui/cui.md#theme-and-resources)
namespace Fpas.Ui.Gui.Cui.Resources;

/// <summary>A font copied into one CUI context; the owning window releases its data on disposal.</summary>
public sealed class CuiFontFace
{
    internal CuiFontFace(CuiWindow owner, int id) => (Owner, Id) = (owner, id);
    internal CuiWindow Owner { get; }
    internal int Id { get; }
    internal HashSet<int> Fallbacks { get; } = [];
}
