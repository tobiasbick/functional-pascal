// [Documentation](../../../docs/ui/gui/files/file-picker.md)
using System.Runtime.Versioning;
using System.Security.AccessControl;
using System.Security.Principal;
using Fpas.Tests.Support;
using Fpas.Ui.Gui.Cui.Tests.Packaging;

namespace Fpas.Ui.Gui.Cui.Tests.Windowing;

/// <summary>Checks the real native file dialogs and their owner/frame boundaries.</summary>
public sealed class CuiFileDialogTests
{
    /// <summary>Returns a Unicode folder and supports cancelling the owned Windows dialog.</summary>
    [WindowsGuiFact]
    public void WindowsFolderDialogSelectsAndCancels()
    {
        using var directory = new TemporaryDirectory();
        string selected = Directory.CreateDirectory(directory.File("Musik — été")).FullName;
        using var window = new CuiWindow("Folder dialog owner", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        window.Text("Owner");
        window.Present();
        Assert.Equal(selected, Choose(selected));
        Assert.Null(Choose(null));
        Assert.True(window.NextFrame());
        window.Text("Returned to owner");
        window.Present();

        string? Choose(string? path)
        {
            nint owner = window.NativeWindow;
            Task driver = Task.Run(() => WindowsFileDialogInput.Respond(owner, path));
            string? result = window.ChooseFolder("FPAS folder selection");
            driver.GetAwaiter().GetResult();
            return result;
        }
    }

    /// <summary>Accepts a proposed destination even where the caller cannot create a file.</summary>
    [WindowsGuiFact]
    [SupportedOSPlatform("windows")]
    public void WindowsSaveDialogReturnsDefaultWithoutTestingFileCreation()
    {
        using var directory = new TemporaryDirectory();
        var folder = new DirectoryInfo(directory.Path);
        string proposed = directory.File("notes — été.txt");
        var access = folder.GetAccessControl();
        var denyCreation = new FileSystemAccessRule(new SecurityIdentifier(WellKnownSidType.WorldSid, null),
            FileSystemRights.CreateFiles, AccessControlType.Deny);
        access.AddAccessRule(denyCreation);
        folder.SetAccessControl(access);
        try
        {
            Assert.Throws<UnauthorizedAccessException>(() => File.WriteAllText(proposed, "probe"));
            using var window = new CuiWindow("Save dialog owner", renderer: CuiRenderer.Software, continuous: true);
            nint owner = window.NativeWindow;
            Task driver = Task.Run(() => WindowsFileDialogInput.Respond(owner, ""));
            string? result = window.SaveFile("FPAS save destination", "Text", "txt", proposed);
            driver.GetAwaiter().GetResult();
            Assert.Equal(proposed, result);
            Assert.False(File.Exists(proposed));
        }
        finally
        {
            access.RemoveAccessRuleSpecific(denyCreation);
            folder.SetAccessControl(access);
        }
    }

    /// <summary>Forwards Unicode paths, default save names and cancellation to the CUI helper.</summary>
    /// <param name="kind">Native operation to invoke.</param>
    /// <param name="cancel">Whether to dismiss without selecting.</param>
    [LinuxFileDialogTheory]
    [InlineData(CuiFileDialogKind.Open, false)]
    [InlineData(CuiFileDialogKind.Save, false)]
    [InlineData(CuiFileDialogKind.Folder, false)]
    [InlineData(CuiFileDialogKind.Open, true)]
    [InlineData(CuiFileDialogKind.Save, true)]
    [InlineData(CuiFileDialogKind.Folder, true)]
    public void NativeDialogsSelectAndCancel(CuiFileDialogKind kind, bool cancel)
    {
        using var directory = new TemporaryDirectory();
        string path = kind == CuiFileDialogKind.Folder
            ? Directory.CreateDirectory(directory.File("Musik — été")).FullName
            : directory.File("notes — été.txt");
        if (kind == CuiFileDialogKind.Open)
        {
            File.WriteAllText(path, "Native open selection");
            File.WriteAllText(directory.File("ignored.bin"), "Excluded by the native filter");
        }
        using var window = new CuiWindow("File dialog owner", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame()); window.Text("Owner"); window.Present();
        string title = "FPAS chooser " + Guid.NewGuid().ToString("N");
        // Save confirms the provided default path without replacing it in the dialog.
        Task driver = Task.Run(() => LinuxFileDialogInput.Respond(title,
            cancel ? null : kind == CuiFileDialogKind.Save ? "" : path, kind == CuiFileDialogKind.Folder));
        string? result = kind switch
        {
            CuiFileDialogKind.Open => window.OpenFile(title, "Text", "txt"),
            CuiFileDialogKind.Save => window.SaveFile(title, "Text", "txt", path),
            _ => window.ChooseFolder(title)
        };
        driver.GetAwaiter().GetResult();
        Assert.Equal(cancel ? null : path, result);
        if (kind == CuiFileDialogKind.Save) Assert.False(File.Exists(path));
        Assert.True(window.NextFrame()); window.Text("Returned to owner"); window.Present();
    }

    /// <summary>Rejects dialog reentry, invalid strings, worker access and calls after disposal.</summary>
    [GuiFact]
    public void DialogsRespectFrameOwnerAndLifetime()
    {
        using var window = new CuiWindow("Dialog lifetime", renderer: CuiRenderer.Software, continuous: true);
        Assert.Throws<ArgumentException>(() => window.SaveFile("Save", "", "", "bad\0name"));
        Assert.Throws<ArgumentException>(() => window.ChooseFolder("bad\0title"));
        Task.Factory.StartNew(() =>
        {
            Assert.Throws<InvalidOperationException>(() => window.ChooseFolder("Worker"));
            Assert.Throws<InvalidOperationException>(() => window.SaveFile("Worker", "", "", ""));
        }, CancellationToken.None, TaskCreationOptions.LongRunning, TaskScheduler.Default).GetAwaiter().GetResult();
        Assert.True(window.NextFrame());
        Assert.Throws<InvalidOperationException>(() => window.SaveFile("Save", "", "", ""));
        Assert.Throws<InvalidOperationException>(() => window.ChooseFolder("Folder"));
        window.Present();
        window.Dispose();
        Assert.Throws<ObjectDisposedException>(() => window.SaveFile("Save", "", "", ""));
        Assert.Throws<ObjectDisposedException>(() => window.ChooseFolder("Folder"));
    }
}
