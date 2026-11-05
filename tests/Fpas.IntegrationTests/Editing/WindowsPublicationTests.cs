using Fpas.IntegrationTests.Processes.Support;
using Fpas.Tests.Support;
using Fpas.Workspaces.Editing;
using System.Runtime.Versioning;
using System.Security.AccessControl;
using System.Security.Principal;
using System.Text;

namespace Fpas.IntegrationTests.Editing;

[SupportedOSPlatform("windows")]
public sealed class WindowsPublicationTests
{
    [WindowsFact]
    public async Task Source_replacement_and_rollback_preserve_explicit_Windows_access_rules()
    {
        using var directory = new TemporaryDirectory();
        var a = directory.File("a.fpas"); var b = directory.File("b.fpas");
        File.WriteAllText(a, "integer"); File.WriteAllText(b, "integer");
        var file = new FileInfo(a); var access = file.GetAccessControl();
        access.SetAccessRuleProtection(isProtected: true, preserveInheritance: true);
        access.AddAccessRule(new(new SecurityIdentifier(WellKnownSidType.WorldSid, null), FileSystemRights.Read, AccessControlType.Allow));
        file.SetAccessControl(access);
        var expected = file.GetAccessControl().GetSecurityDescriptorSddlForm(AccessControlSections.Access);
        SourceChange Change(string path, string before, string after) => new(path, Encoding.UTF8.GetBytes(before), Encoding.UTF8.GetBytes(after));
        await SourcePublication.ApplyAsync([Change(a, "integer", "int")]);
        Assert.Equal("int", File.ReadAllText(a));
        Assert.True(expected == file.GetAccessControl().GetSecurityDescriptorSddlForm(AccessControlSections.Access), "Source replacement changed the original Windows access rules.");
        await Assert.ThrowsAsync<FileNotFoundException>(() => SourcePublication.ApplyAsync([Change(a, "int", "long"), Change(b, "integer", "int")], _ =>
        {
            var staged = Directory.GetFiles(directory.Path, ".fpas-edit-*.new").Single(path => File.ReadAllText(path) == "int");
            File.Delete(staged); return Task.CompletedTask;
        }));
        Assert.Equal("int", File.ReadAllText(a)); Assert.Equal("integer", File.ReadAllText(b));
        Assert.True(expected == file.GetAccessControl().GetSecurityDescriptorSddlForm(AccessControlSections.Access), "Source rollback changed the original Windows access rules.");
        Assert.Empty(Directory.GetFiles(directory.Path, ".fpas-edit-*"));
    }
}
