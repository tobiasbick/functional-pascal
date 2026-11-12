namespace Fpas.Compiler.Tests.Formatting;

public sealed class RecordUpdateFormattingTests
{
    [Theory]
    [InlineData("Id:=Value;", "        Id := Value;")]
    [InlineData("Id:=Value;Name:=NewName;Active:=true;", "        Id := Value;\n        Name := NewName;\n        Active := true;")]
    public void Record_updates_in_case_arms_always_use_one_structural_line_per_field(string fields, string expectedFields)
    {
        var input = "program Demo;begin case Key of when 'id':Item:=Item with " + fields + "end with;end case;end;end program;";
        var expected = "program Demo;\n\nbegin\n  case Key of\n    when 'id':\n      Item := Item with\n" + expectedFields +
            "\n      end with;\n  end case;\nend;\n\nend program;\n";
        Assert.Equal(expected, FormatterTests.Format(input));
    }

    [Fact]
    public void Nested_updates_indent_fields_and_keep_closing_comments_inside_their_blocks()
    {
        const string input = """
            program Demo;begin
            Item:=Item with // outer update
            Child:=Item.Child with Id:=7; // child identifier
            // child end
            end with;
            Name:='new';
            // outer end
            end with;
            end;end program;
            """;
        const string expected = """
            program Demo;

            begin
              Item := Item with // outer update
                Child := Item.Child with
                  Id := 7; // child identifier
                  // child end
                end with;
                Name := 'new';
                // outer end
              end with;
            end;

            end program;
            """;
        Assert.Equal(expected.Replace("\r\n", "\n", StringComparison.Ordinal) + "\n", FormatterTests.Format(input));
    }

    [Fact]
    public void Updates_expand_their_containing_argument_list()
    {
        var output = FormatterTests.Format("program Demo;begin discard Save(Item with Id:=7;Name:='new';end with);end;end program;");
        Assert.Contains("Save(\n    Item with\n      Id := 7;\n      Name := 'new';\n    end with\n  );", output);
    }
}
