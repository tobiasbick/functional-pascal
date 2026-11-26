// [Documentation](../../../docs/pascal/program-structure/cli.md#lint-a-project-or-solution)
using System.Text.Json;

namespace Fpas.IntegrationTests.Workspaces;

public sealed partial class LintTests
{
    /// <summary>Checks the warning boundary, branch depth and independent lambda scope.</summary>
    [Fact]
    public async Task Deep_nesting_warns_once_on_each_path()
    {
        const string source = """
            unit Rules;
            uses System.Exception as Failure;

            public function Read(Flag: bool): int;
            begin
              var Value: int := 0;
              const Items: array of int := [1];
              if Flag then
                while Flag do
                  for Item: int in Items do
                    try
                      const Compute: function(): int := function(): int => if Flag then 1 else 0 end if;
                      begin
                        Value := Compute();
                      end;
                      Value := if Flag then 1 else 2 end if;
                    catch Problem: Failure do
                      case Item of
                        when 1:
                          if Flag then
                            Value := 3;
                          end if;
                        else
                          Value := 4;
                      end case;
                    finally
                      repeat
                        Value := 5;
                      until Flag;
                    end try;
                  end for;
                  break;
                end while;
              elsif Value < 0 then
                Value := 6;
              else
                Value := 7;
              end if;
              return Value;
            end function;
            end unit;
            """;
        using var directory = CreateRulesProject();
        Write(directory, "Rules/Rules.fpas", source);

        var expectedOffsets = new[]
        {
            source.IndexOf("if Flag then 1 else 2", StringComparison.Ordinal),
            source.IndexOf("case Item of", StringComparison.Ordinal),
            source.IndexOf("repeat", StringComparison.Ordinal),
        };
        Assert.Equal(expectedOffsets, await WarningOffsets());
        Assert.Equal([source.IndexOf("if Flag then", expectedOffsets[1], StringComparison.Ordinal)],
            await WarningOffsets("--existing-inputs", "--max-nesting", "5"));
        Assert.Empty(await WarningOffsets("--existing-inputs", "--max-nesting", "6"));

        async Task<int[]> WarningOffsets(params string[] options)
        {
            var result = await Lint(directory, ["Rules/Rules.fpasproj", "--report", "json", .. options]);
            Assert.True(result.ExitCode == 0, result.Output + result.Error);
            using var json = JsonDocument.Parse(result.Output);
            Assert.Equal("ok", json.RootElement.GetProperty("status").GetString());
            return json.RootElement.GetProperty("findings").EnumerateArray().Select(finding =>
            {
                Assert.Equal("deep-nesting", Rule(finding));
                Assert.Equal("warning", finding.GetProperty("severity").GetString());
                Assert.Equal(directory.File("Rules/Rules.fpas"), PathAt(finding));
                return finding.GetProperty("location").GetProperty("offset").GetInt32();
            }).ToArray();
        }
    }
}
