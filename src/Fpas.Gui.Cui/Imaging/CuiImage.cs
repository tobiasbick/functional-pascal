namespace Fpas.Gui.Cui.Imaging;

/// <summary>Owns a texture in its creating CUI window. Use and dispose it on the window's UI thread.</summary>
public sealed class CuiImage : IDisposable
{
    internal CuiImage(CuiWindow owner, int pixelWidth, int pixelHeight, nuint texture)
    {
        Owner = owner;
        PixelWidth = pixelWidth;
        PixelHeight = pixelHeight;
        Texture = texture;
    }

    internal CuiWindow Owner { get; }
    internal nuint Texture { get; private set; }
    internal long LastDrawnFrame { get; set; }

    /// <summary>Gets the uploaded image width in pixels.</summary>
    public int PixelWidth { get; }

    /// <summary>Gets the uploaded image height in pixels.</summary>
    public int PixelHeight { get; }

    /// <summary>Stops future draws and releases the texture when the window no longer needs it.</summary>
    public void Dispose() => Owner.ReleaseImage(this);

    internal void Invalidate() => Texture = 0;
}
