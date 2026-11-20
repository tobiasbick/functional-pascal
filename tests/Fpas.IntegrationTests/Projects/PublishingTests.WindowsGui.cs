// [Documentation](../../../docs/build/publishing.md)
using System.Buffers.Binary;
using System.Diagnostics;
using System.Reflection.PortableExecutable;
using System.Text;
using System.Xml.Linq;
using Fpas.IntegrationTests.Processes.Support;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Projects;

public sealed partial class PublishingTests
{
    /// <summary>Checks each Windows launcher header and executes the published FPAS entry point.</summary>
    [WindowsFact]
    public async Task WinExe_publishes_a_runnable_GUI_subsystem_in_each_executable_mode()
    {
        using var directory = new TemporaryDirectory();
        var project = await ProjectCliFixture.Init(directory, sdk);
        var icon = ConfigureWindowsResources(project);
        File.WriteAllText(directory.File("App/main.fpas"), """
            program App;
            uses System.IO.File as File;
            begin
              File.WriteAllText(args[0], 'started');
            end;
            end program;
            """);

        var build = await Cli(directory, "build", project);
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var check = await Cli(directory, "check", project);
        Assert.True(check.ExitCode == 0, check.Output + check.Error);

        foreach (string mode in new[] { "apphost", "self-contained", "single-file", "aot" })
        {
            string output = directory.File(mode);
            var published = await Cli(directory, "publish", project, "--mode", mode, "--output", output);
            Assert.True(published.ExitCode == 0, mode + ": " + published.Output + published.Error);
            string executable = Path.Combine(output, "App.exe");
            AssertWindowsResources(executable, icon, 13);
            var version = FileVersionInfo.GetVersionInfo(executable);
            Assert.Equal("2.5.8.13", version.FileVersion);
            Assert.Equal("2.5.8-beta.3", version.ProductVersion);

            string receipt = directory.File(mode + ".txt");
            var run = await ProcessRunner.RunExecutableAsync(executable, [receipt], workingDirectory: directory.Path);
            Assert.True(run.ExitCode == 0, mode + ": " + run.Output + run.Error);
            Assert.Equal("started", File.ReadAllText(receipt));
        }
    }

    [Fact]
    public async Task Windows_apphost_copies_and_refreshes_project_resources()
    {
        using var directory = new TemporaryDirectory();
        var project = await ProjectCliFixture.Init(directory, sdk);
        var icon = ConfigureWindowsResources(project);
        var iconPath = directory.File("App/app icon.ico");
        var output = directory.File("published");
        var executable = Path.Combine(output, "App.exe");

        var published = await Publish();
        Assert.True(published.ExitCode == 0, published.Output + published.Error);
        AssertWindowsResources(executable, icon, 13);

        // Change only the icon contents, keeping the project and icon path unchanged.
        icon = CreateIcon(72);
        File.WriteAllBytes(iconPath, icon);
        published = await Publish();
        Assert.True(published.ExitCode == 0, published.Output + published.Error);
        AssertWindowsResources(executable, icon, 13);

        var xml = XDocument.Load(project);
        xml.Root!.Descendants("FileVersion").Single().Value = "2.5.8.19";
        xml.Save(project);
        published = await Publish();
        Assert.True(published.ExitCode == 0, published.Output + published.Error);
        AssertWindowsResources(executable, icon, 19);

        var previous = File.ReadAllBytes(executable);
        File.WriteAllBytes(iconPath, icon[..10]);
        var invalid = await Publish();
        Assert.NotEqual(0, invalid.ExitCode);
        Assert.Contains("FP4005", invalid.Output + invalid.Error);
        Assert.Equal(previous, File.ReadAllBytes(executable));

        Task<ProcessResult> Publish() =>
            Cli(directory, "publish", project, "--mode", "apphost", "--runtime", "win-x64", "--output", output);
    }

    private static byte[] ConfigureWindowsResources(string project)
    {
        var xml = XDocument.Load(project);
        var output = xml.Root!.Descendants("OutputType").Single();
        output.Value = "WinExe";
        output.Parent!.Add(new XElement("ApplicationIcon", "app icon.ico"),
            new XElement("FileVersion", "2.5.8.13"), new XElement("InformationalVersion", "2.5.8-beta.3"),
            new XElement("AssemblyTitle", "FPAS Desktop – Example"), new XElement("Product", "FPAS Tests"));
        xml.Save(project);
        var icon = CreateIcon(55);
        File.WriteAllBytes(Path.Combine(Path.GetDirectoryName(project)!, "app icon.ico"), icon);
        return icon;
    }

    private static byte[] CreateIcon(byte red)
    {
        int[] widths = [16, 32];
        int offset = 6 + 16 * widths.Length;
        using var stream = new MemoryStream();
        using var writer = new BinaryWriter(stream);
        writer.Write((ushort)0);
        writer.Write((ushort)1);
        writer.Write((ushort)widths.Length);
        foreach (int width in widths)
        {
            int size = 40 + (4 * width + 4) * width;
            writer.Write((byte)width);
            writer.Write((byte)width);
            writer.Write((ushort)0);
            writer.Write((ushort)1);
            writer.Write((ushort)32);
            writer.Write(size);
            writer.Write(offset);
            offset += size;
        }
        foreach (int width in widths)
        {
            writer.Write(40); // BITMAPINFOHEADER.
            writer.Write(width);
            writer.Write(width * 2); // The XOR image and AND mask have equal heights.
            writer.Write((ushort)1);
            writer.Write((ushort)32);
            writer.Write(new byte[24]);
            for (int pixel = 0; pixel < width * width; pixel++)
                writer.Write(0xff000000u | ((uint)red << 16));
            writer.Write(new byte[4 * width]); // Row-aligned AND mask.
        }
        return stream.ToArray();
    }

    private static void AssertWindowsResources(string executable, byte[] icon, int revision)
    {
        using var image = new PEReader(File.OpenRead(executable));
        Assert.Equal(Subsystem.WindowsGui, image.PEHeaders.PEHeader!.Subsystem);
        var group = ReadResource(image, 14, 32512);
        Assert.Equal(icon[..6], group[..6]);
        for (int index = 0; index < 2; index++)
        {
            var directory = icon.AsSpan(6 + 16 * index, 16);
            var groupEntry = group.AsSpan(6 + 14 * index, 14);
            Assert.True(directory[..12].SequenceEqual(groupEntry[..12]));
            int id = BinaryPrimitives.ReadUInt16LittleEndian(groupEntry[12..]);
            int size = BinaryPrimitives.ReadInt32LittleEndian(directory[8..]);
            int offset = BinaryPrimitives.ReadInt32LittleEndian(directory[12..]);
            Assert.Equal(icon.AsSpan(offset, size).ToArray(), ReadResource(image, 3, id));
        }

        var version = ReadResource(image, 16, 1);
        int fixedInfo = version.AsSpan().IndexOf(new byte[] { 0xbd, 0x04, 0xef, 0xfe });
        Assert.True(fixedInfo >= 0);
        Assert.Equal(0x00020005, BinaryPrimitives.ReadInt32LittleEndian(version.AsSpan(fixedInfo + 8)));
        Assert.Equal(0x00080000 | revision, BinaryPrimitives.ReadInt32LittleEndian(version.AsSpan(fixedInfo + 12)));
        foreach (string value in new[] { $"2.5.8.{revision}", "2.5.8-beta.3", "FPAS Desktop – Example", "FPAS Tests", "App.exe" })
            Assert.True(version.AsSpan().IndexOf(Encoding.Unicode.GetBytes(value + "\0")) >= 0, value);
    }

    private static byte[] ReadResource(PEReader image, int type, int id)
    {
        var section = image.GetSectionData(image.PEHeaders.PEHeader!.ResourceTableDirectory.RelativeVirtualAddress);
        int typeDirectory = Entry(section, 0, type) & int.MaxValue;
        int languageDirectory = Entry(section, typeDirectory, id) & int.MaxValue;
        var data = section.GetReader(Entry(section, languageDirectory, 0), 16);
        int address = data.ReadInt32();
        int size = data.ReadInt32();
        return image.GetSectionData(address).GetContent(0, size).ToArray();

        static int Entry(PEMemoryBlock section, int offset, int id)
        {
            var directory = section.GetReader(offset, section.Length - offset);
            directory.Offset = 12;
            int count = directory.ReadUInt16() + directory.ReadUInt16();
            for (int index = 0; index < count; index++)
            {
                int candidate = directory.ReadInt32();
                int next = directory.ReadInt32();
                if (candidate == id) return next;
            }
            throw new InvalidDataException($"Windows resource directory is missing ID {id}.");
        }
    }
}
