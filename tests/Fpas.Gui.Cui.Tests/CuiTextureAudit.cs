using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Gui.Cui.Tests;

internal sealed unsafe partial class CuiTextureAudit : IDisposable
{
    private const string Library = "fpas_cui";
    private const int Rgba8 = 4;
    private readonly Dictionary<nuint, int> liveTextures = [];
    private readonly Renderer backend;
    private readonly GCHandle handle;
    private nint software;
    private bool rejectNextImage;

    internal CuiTextureAudit()
    {
        software = CreateSoftwareRenderer();
        if (software == 0) throw new InvalidOperationException("CUI could not create the audit renderer.");
        try
        {
            backend = GetSoftwareInterface(software);
            handle = GCHandle.Alloc(this);
        }
        catch
        {
            DestroySoftwareRenderer(software);
            throw;
        }
    }

    internal int ImageCount => liveTextures.Values.Count(format => format == Rgba8);
    internal int TextureCount => liveTextures.Count;
    internal int RejectedUploads { get; private set; }
    internal Exception? CallbackFailure { get; private set; }

    internal void Attach(CuiWindow window)
    {
        // Forward CUI uploads to a real software renderer. The window still runs its normal frame loop.
        var observer = new Renderer
        {
            User = GCHandle.ToIntPtr(handle), Create = &Create, Update = &Update, Destroy = &Destroy
        };
        SetRenderer(GetContext(window), &observer);
    }

    internal void RejectNextImage() => rejectNextImage = true;

    /// <summary>Destroys the audit renderer after the attached window has closed.</summary>
    public void Dispose()
    {
        if (software == 0) return;
        DestroySoftwareRenderer(software);
        software = 0;
        handle.Free();
    }

    private static CuiTextureAudit FromUser(nint user) => (CuiTextureAudit)GCHandle.FromIntPtr(user).Target!;

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static nuint Create(nint user, int width, int height, int format, void* pixels)
    {
        var audit = FromUser(user);
        if (format == Rgba8 && audit.rejectNextImage)
        {
            audit.rejectNextImage = false;
            audit.RejectedUploads++;
            return 0;
        }

        nuint texture = audit.backend.Create(audit.backend.User, width, height, format, pixels);
        if (texture == 0) return 0;
        try
        {
            audit.liveTextures.Add(texture, format);
            return texture;
        }
        catch (Exception error)
        {
            audit.CallbackFailure ??= error;
            audit.backend.Destroy(audit.backend.User, texture);
            return 0;
        }
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static void Update(nint user, nuint texture, int x, int y, int width, int height, void* pixels, int stride)
    {
        var audit = FromUser(user);
        audit.backend.Update(audit.backend.User, texture, x, y, width, height, pixels, stride);
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static void Destroy(nint user, nuint texture)
    {
        var audit = FromUser(user);
        try
        {
            if (!audit.liveTextures.Remove(texture))
                throw new InvalidOperationException("CUI released a texture that is not live.");
            audit.backend.Destroy(audit.backend.User, texture);
        }
        catch (Exception error) { audit.CallbackFailure ??= error; }
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct Renderer
    {
        internal nint User;
        internal delegate* unmanaged[Cdecl]<nint, int, int, int, void*, nuint> Create;
        internal delegate* unmanaged[Cdecl]<nint, nuint, int, int, int, int, void*, int, void> Update;
        internal delegate* unmanaged[Cdecl]<nint, nuint, void> Destroy;
    }

    [UnsafeAccessor(UnsafeAccessorKind.Field, Name = "context")]
    private static extern ref readonly nint GetContext(CuiWindow window);

    [LibraryImport(Library, EntryPoint = "cui_sw_create")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    private static partial nint CreateSoftwareRenderer();

    [LibraryImport(Library, EntryPoint = "cui_sw_destroy")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    private static partial void DestroySoftwareRenderer(nint renderer);

    [LibraryImport(Library, EntryPoint = "cui_sw_iface")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    private static partial Renderer GetSoftwareInterface(nint renderer);

    [LibraryImport(Library, EntryPoint = "cui_set_renderer")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    private static partial void SetRenderer(nint context, Renderer* renderer);
}
