using Fpas.Compiler.Analysis;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Analysis;

public sealed class MetadataImageTests
{
    [Fact]
    public void Immutable_images_replace_disk_inputs_for_both_analysis_and_emission()
    {
        using var directory = new TemporaryDirectory(); var path = directory.File("Library.dll");
        string[] references = [.. TestEnvironment.References, typeof(Core.RecordAttribute).Assembly.Location];
        var library = FpasCompiler.Compile(new([new("library.fpas", "unit Library; public function Read(): int; begin return 42; end function; end unit;")], "Library", references, CompilationTarget.Library));
        Assert.True(library.Success);
        var bytes = library.Program!.AssemblyImage; var image = new MetadataImage(path, bytes); Array.Clear(bytes);
        const string consumer = "program Consumer; uses Library as L; begin discard L.Read(); end; end program;";
        var request = new AnalysisRequest([new("consumer.fpas", consumer)], "Consumer", [.. references, path], ReferenceImages: [image]);
        var analyzed = FpasCompiler.Analyze(request); Assert.True(analyzed.Success, string.Join(';', analyzed.Diagnostics));
        Assert.True(FpasCompiler.Compile(new(request.Sources, request.AssemblyName, request.References, ReferenceImages: request.ReferenceImages)).Success);
        Assert.Equal(analyzed.Snapshot!.Id, FpasCompiler.Analyze(request).Snapshot!.Id);
        Assert.Equal(image.ContentHash, image.WithPath(directory.File("other.dll")).ContentHash);
        Assert.Empty(Directory.GetFileSystemEntries(directory.Path));
        Assert.Contains(FpasCompiler.Analyze(request with { ReferenceImages = null }).Diagnostics, diagnostic => diagnostic.Code == "FP4001");
    }

    [Fact]
    public void Invalid_or_unrequested_images_are_rejected_without_loading_runtime_code()
    {
        using var directory = new TemporaryDirectory(); var path = directory.File("bad.dll");
        var request = new AnalysisRequest([new("a.fpas", "unit A; end unit;")], "A", [.. TestEnvironment.References, path], CompilationTarget.Library,
            ReferenceImages: [new(path, [1, 2, 3])]);
        Assert.Contains(FpasCompiler.Analyze(request).Diagnostics, diagnostic => diagnostic.Code == "FP4001");
        Assert.Throws<ArgumentException>(() => FpasCompiler.Analyze(request with { References = TestEnvironment.References }));
    }
}
