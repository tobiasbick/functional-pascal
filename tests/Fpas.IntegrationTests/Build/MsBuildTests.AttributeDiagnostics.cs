using System.Text.Json;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Invalid_attribute_usage_constants_and_named_members_fail_before_emission()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/SharedApi.cs"), """
            namespace Shared;
            [AttributeUsage(AttributeTargets.Method)] public class OnlyMethodAttribute : Attribute { }
            public sealed class InheritedUsageAttribute : OnlyMethodAttribute { }
            public sealed class PrivateConstructorAttribute : Attribute { private PrivateConstructorAttribute() { } }
            public sealed class DataAttribute(object? value) : Attribute
            {
                public object? Value { get; } = value;
                public static int Static;
                public readonly int Readonly = 0;
                public int PrivateSetter { get; private set; }
                public int GetterOnly => 1;
                public int this[int index] { get => index; set { } }
            }
            public sealed class TypeAttribute(Type type) : Attribute { public Type Type { get; } = type; }
            public sealed class DecimalAttribute(decimal value) : Attribute { public decimal Value { get; } = value; }
            public static class Source { public static object? Value => null; }
            """);
        var build = await sdk.Run(project, "build", "Shared/Shared.csproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        (string Source, string Code)[] cases =
        [
            ("[S.InheritedUsageAttribute] type R = record end record;", "FP3035"),
            ("[S.PrivateConstructorAttribute] procedure P(); begin end procedure;", "FP3003"),
            ("[S.DataAttribute(S.Source.Value)] procedure P(); begin end procedure;", "FP3035"),
            ("[S.DataAttribute(new Sys.Object())] procedure P(); begin end procedure;", "FP3035"),
            ("[S.DataAttribute(1.0M)] procedure P(); begin end procedure;", "FP3035"),
            ("[S.DataAttribute(cast<nint>(1))] procedure P(); begin end procedure;", "FP3035"),
            ("[S.DataAttribute(cast<int?>(1))] procedure P(); begin end procedure;", "FP3003"),
            ("[S.DataAttribute(cast<byte>(300))] procedure P(); begin end procedure;", "FP3035"),
            ("[S.DataAttribute(1 div 0)] procedure P(); begin end procedure;", "FP3035"),
            ("[S.DataAttribute(new int[2])] procedure P(); begin end procedure;", "FP3035"),
            ("[S.DataAttribute(1, Static := 1)] procedure P(); begin end procedure;", "FP3035"),
            ("[S.DataAttribute(1, Readonly := 1)] procedure P(); begin end procedure;", "FP3035"),
            ("[S.DataAttribute(1, PrivateSetter := 1)] procedure P(); begin end procedure;", "FP3035"),
            ("[S.DataAttribute(1, GetterOnly := 1)] procedure P(); begin end procedure;", "FP3035"),
            ("[S.DataAttribute(1, Item := 1)] procedure P(); begin end procedure;", "FP3035"),
            ("[S.TypeAttribute(typeof(T))] type R<T> = record public Value: T; end record;", "FP3035"),
            ("[S.TypeAttribute(typeof(C.List<T>))] procedure P<T>(); begin end procedure;", "FP3035"),
            ("[S.TypeAttribute(@type := typeof(int))] procedure P(); begin end procedure;", "FP3003"),
            ("[S.DecimalAttribute(1.0M)] procedure P(); begin end procedure;", "FP3003"),
            ("[Compiler.AsyncStateMachineAttribute(typeof(Sys.Object))] async function P(): Tasks.Task; begin end function;", "FP3035"),
            ("[Compiler.MethodImplAttribute(Compiler.MethodImplOptions.InternalCall)] procedure P(); begin end procedure;", "FP3035"),
            ("[Compiler.MethodImplAttribute(MethodCodeType := Compiler.MethodCodeType.Runtime)] procedure P(); begin end procedure;", "FP3035")
        ];
        foreach (var (source, code) in cases)
        {
            var text = "program Invalid; uses Shared as S, System as Sys, System.Collections.Generic as C, System.Threading.Tasks as Tasks, System.Runtime.CompilerServices as Compiler; " + source + " begin end; end program;";
            var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", "-", "--output", project.File("Invalid.dll"),
                "--reference-dir", TestEnvironment.ReferenceDirectory, "--reference", project.File("Shared/bin/Debug/net10.0/Shared.dll"), "--diagnostics", "json"], text);
            Assert.True(result.ExitCode != 0, source);
            using var document = JsonDocument.Parse(result.Output);
            var diagnostic = Assert.Single(document.RootElement.GetProperty("diagnostics").EnumerateArray());
            Assert.True(code == diagnostic.GetProperty("code").GetString(), source + ": " + diagnostic.GetProperty("message").GetString());
            Assert.False(File.Exists(project.File("Invalid.dll")));
        }
    }
}
