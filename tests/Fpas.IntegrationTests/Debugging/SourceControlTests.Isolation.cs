using Fpas.Debugging.Core;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class SourceControlTests
{
    [Fact]
    public async Task An_invalid_foreign_pdb_does_not_break_a_valid_program_breakpoint()
    {
        using var program = new Debuggee("own-foreign-pdb");
        var source = program.FilePath("Foreign.fpas"); var library = program.FilePath("Foreign.dll");
        File.WriteAllText(source, "unit Foreign; public function Number(): int; begin return 40; end function; end unit;");
        await program.CompileFile(source, library, "--target", "library");
        File.WriteAllText(Path.ChangeExtension(library, ".pdb"), "This is not a portable PDB.");
        await program.Compile("""
            program ForeignSymbols;
            uses Foreign as F, System.Console as Console;
            function Work(): int;
            begin
              return F.Number() + 2;
            end function;
            begin
              const Answer: int := Work();
              Console.WriteLine(Answer);
            end;
            end program;
            """, "--reference", library);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, deadline.Token);
        await host.SetBreakpoints(new(program.Source, [new(9)], host.Snapshot.StopId), deadline.Token);
        await host.Resume(host.Snapshot.StopId!.Value, "continue", deadline.Token);
        var hit = await host.WaitForStop(deadline.Token);
        Assert.Equal("breakpoint", hit.Reason);
        Assert.Equal("invalid_symbols", Assert.Single(hit.ModuleIssues!, issue => issue.Module == library).Error.Code);
        Assert.Contains(await host.Bindings(hit.StopId!.Value, deadline.Token), item => item.Name == "Answer" && item.Scalar == "42");
        await host.Stop(deadline.Token);
    }

    [Fact]
    public async Task Loading_the_main_assembly_again_does_not_create_a_second_entry_stop()
    {
        using var program = new Debuggee("own-entry-once");
        await program.Compile("""
            program EntryOnce;
            uses System.Console as Console, System.IO.File as File, System.Reflection.Assembly as Assembly,
              System.Reflection.MethodInfo as MethodInfo, System.Runtime.Loader.AssemblyLoadContext as LoadContext;
            begin
              if File.Exists('called-again.txt') then
                Console.WriteLine('second');
              else
                File.WriteAllText('called-again.txt', 'yes');
                const Context: LoadContext := new LoadContext('second-main', true);
                const Loaded: Assembly := Context.LoadFromAssemblyPath(Assembly.GetExecutingAssembly().Location);
                const Entry: MethodInfo? := Loaded.EntryPoint;
                if Entry is MethodInfo then
                  const Arguments: object?[] := new object?[Entry.GetParameters().Length];
                  if Arguments.Length > 0 then Arguments[0] := new string[0]; end if;
                  discard Entry.Invoke(null, Arguments);
                end if;
                Console.WriteLine('done');
              end if;
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, deadline.Token);
        await host.SetBreakpoints(new(program.Source, [new(17)], host.Snapshot.StopId), deadline.Token);
        await host.Resume(host.Snapshot.StopId!.Value, "continue", deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        var ambiguous = Assert.Single(host.Snapshot.Breakpoints!);
        Assert.Equal("rejected", ambiguous.State);
        Assert.Equal("ambiguous_source", ambiguous.Error!.Code);
        await host.Stop(deadline.Token);
        Assert.Equal("second\ndone\n", string.Concat(host.Output().Items.Select(item => item.Text)).Replace("\r\n", "\n", StringComparison.Ordinal));
    }
}
