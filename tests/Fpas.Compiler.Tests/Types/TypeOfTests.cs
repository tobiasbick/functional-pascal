using Fpas.Compiler.Tests.ControlFlow;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Types;

public sealed class TypeOfTests
{
    [Theory]
    [InlineData("typeof()", "FP2001")]
    [InlineData("typeof(42)", "FP2001")]
    [InlineData("typeof(int, string)", "FP2001")]
    [InlineData("typeof(Missing)", "FP3005")]
    [InlineData("typeof(Value)", "FP3005")]
    [InlineData("typeof(Sys)", "FP3005")]
    [InlineData("typeof(Sys.Void)", "FP3005")]
    [InlineData("typeof(C.List)", "FP3001")]
    [InlineData("typeof(C.List<>)", "FP2001")]
    [InlineData("typeof(int??)", "FP2001")]
    public void Typeof_requires_a_supported_resolved_type_in_parentheses(string expression, string code)
    {
        var result = DiagnosticsTests.Compile($$"""
            program Invalid;
            uses System as Sys, System.Collections.Generic as C;
            begin const Value: int := 1; discard {{expression}}; end;
            end program;
            """);
        Assert.False(result.Success);
        Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Unconstrained_nullable_type_parameters_follow_the_existing_type_rules()
    {
        var result = DiagnosticsTests.Compile("""
            program Invalid;
            uses System as Sys;
            function Get<T>(): Sys.Type; begin return typeof(T?); end function;
            begin end;
            end program;
            """);
        Assert.False(result.Success); Assert.Equal("FP3026", Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Typeof_does_not_extend_the_restricted_contract_expression_subset()
    {
        var result = DiagnosticsTests.Compile("""
            program Invalid;
            procedure Check(); requires typeof(int) <> null; begin end procedure;
            begin end;
            end program;
            """);
        Assert.False(result.Success); Assert.Equal("FP3027", Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Typeof_is_not_an_optional_parameter_metadata_constant()
    {
        var result = DiagnosticsTests.Compile("""
            program Invalid;
            uses System as Sys;
            procedure Check(Value: Sys.Type := typeof(int)); begin end procedure;
            begin end;
            end program;
            """);
        Assert.False(result.Success); Assert.Equal("FP3028", Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Typeof_emits_type_tokens_and_the_CLR_handle_resolver_deterministically()
    {
        const string text = """
            unit Tokens;
            uses System as Sys;
            public function Concrete(): Sys.Type; begin return typeof(int); end function;
            public function Generic<T>(): Sys.Type; begin return typeof(T); end function;
            public function Nullable<T>(): Sys.Type; where T: struct; begin return typeof(T?); end function;
            """ + "end unit;";
        CompilationResult Compile(IEnumerable<string> references) => FpasCompiler.Compile(new([new("tokens.fpas", text)], "Tokens",
            references.ToArray(), CompilationTarget.Library, EmitDebugInformation: true));
        var first = Compile(TestEnvironment.References); var second = Compile(TestEnvironment.References.Reverse());
        Assert.True(first.Success, string.Join(';', first.Diagnostics));
        Assert.True(second.Success, string.Join(';', second.Diagnostics));
        Assert.Equal(first.Program!.AssemblyImage, second.Program!.AssemblyImage);
        Assert.Equal(first.Program.PortablePdb, second.Program.PortablePdb);
        using var pe = new PEReader(new MemoryStream(first.Program.AssemblyImage));
        var metadata = pe.GetMetadataReader();
        var concrete = metadata.MethodDefinitions.Select(metadata.GetMethodDefinition).Single(method => metadata.GetString(method.Name) == "Concrete");
        var body = pe.GetMethodBody(concrete.RelativeVirtualAddress).GetILBytes()!.SkipWhile(code => code == (byte)ILOpCode.Nop).ToArray();
        Assert.Equal((byte)ILOpCode.Ldtoken, body[0]);
        var token = MetadataTokens.EntityHandle(BitConverter.ToInt32(body, 1));
        Assert.Equal(HandleKind.TypeReference, token.Kind);
        Assert.Equal("Int32", metadata.GetString(metadata.GetTypeReference((TypeReferenceHandle)token).Name));
        Assert.Equal((byte)ILOpCode.Call, body[5]);
        var resolve = metadata.GetMemberReference((MemberReferenceHandle)MetadataTokens.EntityHandle(BitConverter.ToInt32(body, 6)));
        Assert.Equal("GetTypeFromHandle", metadata.GetString(resolve.Name));
        Assert.Contains(Enumerable.Range(1, metadata.GetTableRowCount(TableIndex.TypeSpec)).Select(MetadataTokens.TypeSpecificationHandle),
            handle => metadata.GetBlobBytes(metadata.GetTypeSpecification(handle).Signature).SequenceEqual(new byte[] { 0x1e, 0 }));
        Assert.DoesNotContain(metadata.AssemblyReferences, handle => metadata.GetString(metadata.GetAssemblyReference(handle).Name) == "Fpas.Core");
    }
}
