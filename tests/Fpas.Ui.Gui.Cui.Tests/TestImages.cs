namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Small solid-color PNG images for native image decoding checks.</summary>
internal static class TestImages
{
    /// <summary>A 16 x 16 PNG whose pixels CUI captures as 0xffdc641e.</summary>
    internal static readonly byte[] Blue = Convert.FromBase64String(
        "iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAIAAACQkWg2AAAAI0lEQVR4nGOUS7nDQApgIkk1w6gG4gATkergYFQDMYDkUAIAYLIBfpuLCPgAAAAASUVORK5CYII=");

    /// <summary>A 16 x 16 PNG with a color distinct from <see cref="Blue"/>.</summary>
    internal static readonly byte[] Red = Convert.FromBase64String(
        "iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAIAAACQkWg2AAAAI0lEQVR4nGO8pKzLQApgIkk1w6gG4gATkergYFQDMYDkUAIAqr4BQidXde8AAAAASUVORK5CYII=");
}
