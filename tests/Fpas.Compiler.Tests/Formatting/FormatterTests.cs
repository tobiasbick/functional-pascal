using Fpas.Compiler.Formatting;
using Fpas.Compiler.Tests.Support;
using System.Reflection.PortableExecutable;
using System.Reflection.Metadata;

namespace Fpas.Compiler.Tests.Formatting;

public sealed class FormatterTests
{
    [Theory]
    [InlineData("\n", "\n")]
    [InlineData("\r\n", "\r\n")]
    [InlineData("\r", "\r")]
    [InlineData("\r\n", "\r")]
    public void Formatting_normalizes_line_breaks_but_preserves_spaces_and_tabs_in_a_multiline_literal(string first, string second)
    {
        var source = $"program Example;begin const Text:string:='first{first}\t  // part of the literal  {second}last''line';discard Text;end;end program;";
        var formatted = Format(source);
        Assert.DoesNotContain('\r', formatted);
        Assert.Contains("  const Text: string := 'first\n\t  // part of the literal  \nlast''line';\n  discard Text;", formatted);
        Assert.Equal(formatted, Format(formatted));
        Assert.True(TestCompilation.Compile(formatted).Success);
    }

    [Fact]
    public void A_routine_prefix_formats_contracts_and_helpers_before_the_outer_begin()
    {
        const string source = "program Example;function Run(Value:int):int;requires Value>=0;ensures Value>=0;function Read():int;begin return Value;end function;begin return Read();end function;begin end;end program;";
        var formatted = Format(source);
        Assert.Contains("ensures Value >= 0;\n  function Read(): int;\n  begin", formatted);
        Assert.Contains("  end function;\nbegin\n  return Read();", formatted);
        Assert.Equal(formatted, Format(formatted));
    }

    [Fact]
    public void Formats_program_declarations_and_named_blocks()
    {
        const string input = "program Demo;uses System.Console as Console;function Twice(Value:int):int;begin return Value*2;end function;begin const X:int:=Twice(2);if X=4 then Console.WriteLine('ok');else Console.WriteLine('bad');end if;Console.WriteLine(X);end;end program;";
        const string expected = """
            program Demo;

            uses System.Console as Console;

            function Twice(Value: int): int;
            begin
              return Value * 2;
            end function;

            begin
              const X: int := Twice(2);
              if X = 4 then
                Console.WriteLine('ok');
              else
                Console.WriteLine('bad');
              end if;

              Console.WriteLine(X);
            end;
            end program;
            """;
        Assert.Equal(expected.Replace("\r\n", "\n", StringComparison.Ordinal) + "\n", Format(input));
    }

    [Fact]
    public void Preserves_comments_documentation_spelling_and_literal_tokens()
    {
        const string input = "\uFEFF// detached\r\n\r\n// Demo documentation\r\nprogram Demo; // header\r\nuses System.Console as Console;\r\nbegin // body\r\n// attached binding\r\nconst @event:int:=$F_F; // value\r\n\r\n// detached inner\r\n\r\nConsole.WriteLine('it''s // string '+#32+';'+1_000UL.ToString());\r\nend;end program; // last\r\n// eof\r\n";
        var output = Format(input);
        Assert.DoesNotContain('\r', output);
        Assert.DoesNotContain('\uFEFF', output);
        Assert.Contains("// attached binding\n  const @event: int := $F_F; // value", output);
        Assert.Contains("// detached inner\n\n  Console", output);
        Assert.Contains("'it''s // string ' + #32 + ';' + 1_000UL.ToString()", output);
        Assert.EndsWith("end program; // last\n// eof\n", output);
    }

    [Fact]
    public void Keeps_generics_distinct_from_comparison_operators()
    {
        var output = Format("program Demo;uses System.Collections.Generic.List as List;type Values=List<int?>;begin const X:int:=-1;const Y:bool:=X<2;const Z:bool:=(X+1)>0;end;end program;");
        Assert.Contains("type Values = List<int?>;", output);
        Assert.Contains("const X: int := -1;", output);
        Assert.Contains("X < 2", output);
        Assert.Contains("(X + 1) > 0", output);
    }

    [Fact]
    public void Retains_authored_argument_and_formal_parameter_groups()
    {
        var output = Format("program Demo;function Add(\n A:int;B:int;\n C:int\n):int;begin return A+B+C;end function;begin discard Add(\n1,2,\n3\n);end;end program;");
        Assert.Contains("Add(\n  A: int; B: int;\n  C: int\n): int;", output);
        Assert.Contains("discard Add(\n    1, 2,\n    3\n  );", output);
    }

    [Fact]
    public void Expands_long_calls_and_binary_conditions()
    {
        var argument = "'" + new string('a', 45) + "'";
        var output = Format($"program Demo;begin discard Combine({argument},{argument});if VeryLongConditionNameOne+VeryLongConditionNameTwo+VeryLongConditionNameThree+VeryLongConditionNameFour>0 then return;end if;end;end program;");
        Assert.Contains("Combine(\n    " + argument + ",\n    " + argument + "\n  )", output);
        Assert.Contains("\n  then\n", output);
        Assert.All(output.Split('\n'), line => Assert.True(line.Length <= 100, line));
    }

    [Theory]
    [InlineData("program Broken; begin return end; end program;")]
    [InlineData("program Broken; begin // missing endings")]
    [InlineData("PROGRAM Broken; begin end; end program;")]
    [InlineData("program Broken; begin const Text:string:='broken; end; end program;")]
    [InlineData("program Broken;begin const X:int:=1;if X > = 1 then return;end if;end;end program;")]
    public void Rejects_invalid_source_without_partial_output(string input)
    {
        var result = FpasFormatter.Format(new("input.fpas", input));
        Assert.False(result.Success);
        Assert.Null(result.Text);
        Assert.NotEmpty(result.Diagnostics);
        Assert.All(result.Diagnostics, diagnostic => Assert.Equal("input.fpas", diagnostic.Location?.Path));
    }

    [Fact]
    public void Formatting_is_syntax_only_and_cancellable()
    {
        Assert.Contains("Missing()", Format("program Demo;begin Missing();end;end program;"));
        using var cancellation = new CancellationTokenSource();
        cancellation.Cancel();
        Assert.Throws<OperationCanceledException>(() => FpasFormatter.Format(new("input.fpas", "program Demo;begin end;end program;"), cancellation.Token));
    }

    [Fact]
    public void Comments_at_each_token_boundary_keep_their_order_and_attachment()
    {
        var pieces = "program|Demo|;|uses|System.Console|as|Console|;|function|Value|(|)|:|int|;|begin|return|2|+|3|;|end|function|;|begin|Console.WriteLine|(|Value|(|)|)|;|end|;|end|program|;".Split('|');
        for (var boundary = 0; boundary <= pieces.Length; boundary++)
            foreach (var comment in new[] { "// inline\n", "\n// attached\n", "\n// detached\n\n" })
            {
                var input = string.Join(' ', pieces.Take(boundary)) + " " + comment + string.Join(' ', pieces.Skip(boundary));
                var output = Format(input);
                Assert.Contains(comment.Contains("inline", StringComparison.Ordinal) ? "// inline" : comment.Contains("detached", StringComparison.Ordinal) ? "// detached" : "// attached", output);
            }
    }

    [Fact]
    public void Long_authored_groups_gain_breaks_and_empty_closures_stay_compact()
    {
        var argument = "'" + new string('a', 46) + "'";
        var output = Format($"program Demo;begin discard Call(\n{argument},{argument}\n);const Empty:procedure():=procedure()begin end procedure;end;end program;");
        Assert.Contains(argument + ",\n    " + argument, output);
        Assert.Contains("procedure() begin end procedure;", output);
    }

    [Fact]
    public void Width_uses_display_cells_for_wide_characters_and_combining_sequences()
    {
        var wide = "'" + new string('界', 24) + "'";
        var combining = "'" + string.Concat(Enumerable.Repeat("e\u0301", 24)) + "'";
        Assert.Contains("Call(\n", Format($"program Demo;begin discard Call({wide},{wide});end;end program;"));
        Assert.Contains($"Call({combining}, {combining})", Format($"program Demo;begin discard Call({combining},{combining});end;end program;"));
        var tabs = "'" + new string('\t', 14) + "'";
        Assert.Contains("Call(\n", Format($"program Demo;begin discard Call({tabs},{tabs});end;end program;"));
    }

    [Fact]
    public void Comments_before_closers_remain_inside_their_bodies()
    {
        var output = Format("program Demo;begin\n// an empty entry body\nend;end program;");
        Assert.Contains("begin\n  // an empty entry body\nend;", output);
        output = Format("program Demo;begin discard Call(\n1\n// last argument\n);end;end program;");
        Assert.Contains("1\n    // last argument\n  );", output);
    }

    [Fact]
    public void A_failed_generic_probe_does_not_duplicate_inner_generic_layout()
    {
        var output = Format("program Demo;begin const Smaller:bool:=Value<Identity<\nint\n>(4);end;end program;");
        Assert.Contains("Value < Identity<\n    int\n  >(4)", output);
    }

    [Fact]
    public void Type_constraints_members_contracts_and_attributes_use_structural_lines()
    {
        var output = Format("unit Models;[Sys.Obsolete('sample')]public type Box<T> =where T:class;record public Value:T;public function Read():T;requires true;ensures true;begin return Self.Value;end function;end record;end unit;");
        Assert.Contains("[Sys.Obsolete('sample')]\npublic type Box<T> =\nwhere T: class;\nrecord\n  public Value: T;\n\n  public function Read(): T;\n  requires true;\n  ensures true;\n  begin\n    return Self.Value;\n  end function;\nend record;\nend unit;", output);
    }

    [Fact]
    public void Trailing_comments_keep_section_separators_and_do_not_wrap_short_code()
    {
        var comment = "// " + new string('x', 120);
        var output = Format($"program Demo; // header\nuses System.Console as Console; // imports\nbegin Console.WriteLine(1,2); {comment}\nend;end program;");
        Assert.Contains("program Demo; // header\n\nuses", output);
        Assert.Contains("as Console; // imports\n\nbegin", output);
        Assert.Contains("Console.WriteLine(1, 2); " + comment, output);
    }

    [Fact]
    public void Union_payload_lists_retain_formal_parameter_line_groups()
    {
        var output = Format("unit Shapes;type Shape=union Point(\nX:int;\nY:int\n);end union;end unit;");
        Assert.Contains("  Point(\n    X: int;\n    Y: int\n  );", output);
    }

    [Fact]
    public void Original_and_formatted_programs_compile_with_the_same_method_bodies()
    {
        const string input = "program Demo;uses System.Console as Console;function GetValue():int;begin return if true then (2+3)*4 else 0 end if;end function;begin Console.WriteLine(GetValue());end;end program;";
        var before = TestCompilation.Compile(input); var after = TestCompilation.Compile(Format(input));
        Assert.True(before.Success, string.Join('\n', before.Diagnostics));
        Assert.True(after.Success, string.Join('\n', after.Diagnostics));
        static string[] Bodies(byte[] image)
        {
            using var stream = new MemoryStream(image);
            using var pe = new PEReader(stream);
            var metadata = pe.GetMetadataReader();
            return metadata.MethodDefinitions.Select(handle => metadata.GetMethodDefinition(handle)).Select(method =>
                metadata.GetString(method.Name) + ":" + Convert.ToHexString(pe.GetMethodBody(method.RelativeVirtualAddress).GetILBytes()!)).ToArray();
        }
        Assert.Equal(Bodies(before.Program!.AssemblyImage), Bodies(after.Program!.AssemblyImage));
    }

    internal static string Format(string input)
    {
        var result = FpasFormatter.Format(new("input.fpas", input));
        Assert.True(result.Success, string.Join('\n', result.Diagnostics));
        var second = FpasFormatter.Format(new("input.fpas", result.Text!));
        Assert.True(second.Success, string.Join('\n', second.Diagnostics));
        Assert.Equal(result.Text, second.Text);
        return result.Text!;
    }
}
