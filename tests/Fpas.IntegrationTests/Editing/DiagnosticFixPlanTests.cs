using Fpas.Compiler;
using Fpas.Tests.Support;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Loading;
using System.Text;

namespace Fpas.IntegrationTests.Editing;

public sealed class DiagnosticFixPlanTests
{
    private static FixSuggestion Suggest(string path, int offset, string oldText, string newText, DiagnosticApplicability applicability = DiagnosticApplicability.Automatic)
        => new("Project", "FP3005", new("Replace text", applicability, [new(new(path, 1, offset + 1, oldText.Length, Offset: offset), oldText, newText)]));

    [Fact]
    public async Task Linked_edits_are_deduplicated_and_UTF16_offsets_preserve_BOM_CRLF_and_permissions()
    {
        using var directory = new TemporaryDirectory(); var path = directory.File("source.fpas");
        var text = "\uFEFF// 😀\r\nunit Sample; public const X: integer := 1; end unit;\r\n";
        File.WriteAllBytes(path, Encoding.UTF8.GetBytes(text));
        UnixFileMode? mode = OperatingSystem.IsWindows() ? null : UnixFileMode.UserRead | UnixFileMode.UserWrite | UnixFileMode.UserExecute;
        if (!OperatingSystem.IsWindows()) File.SetUnixFileMode(path, mode!.Value);
        var suggestion = Suggest(path, text.IndexOf("integer", StringComparison.Ordinal), "integer", "int");
        var review = Suggest(path, 0, "\uFEFF", "", DiagnosticApplicability.Review);
        var plan = await DiagnosticFixPlan.CreateAsync([new(path, text)], [suggestion, suggestion with { Project = "LinkedOwner" }, review]);
        Assert.Equal(1, plan.EditCount); Assert.Equal(3, plan.Suggestions.Count);
        Assert.Equal(Encoding.UTF8.GetBytes(text), File.ReadAllBytes(path));
        await SourcePublication.ApplyAsync(plan.Changes);
        Assert.Equal(Encoding.UTF8.GetBytes(text.Replace("integer", "int", StringComparison.Ordinal)), File.ReadAllBytes(path));
        if (!OperatingSystem.IsWindows()) Assert.Equal(mode, File.GetUnixFileMode(path));
        Assert.Empty(Directory.GetFiles(directory.Path, ".fpas-edit-*"));
    }

    [Theory]
    [InlineData(0, "abc", "x", 0, "abc", "y")]
    [InlineData(0, "abc", "x", 1, "b", "y")]
    [InlineData(0, "abc", "x", 1, "", "y")]
    [InlineData(1, "", "x", 1, "", "y")]
    public async Task Competing_or_overlapping_corrections_never_write(int first, string oldFirst, string newFirst, int second, string oldSecond, string newSecond)
    {
        using var directory = new TemporaryDirectory(); var path = directory.File("source.fpas"); File.WriteAllText(path, "abc");
        var failure = await Assert.ThrowsAsync<ContextFailure>(() => DiagnosticFixPlan.CreateAsync([new(path, "abc")],
            [Suggest(path, first, oldFirst, newFirst), Suggest(path, second, oldSecond, newSecond)]));
        Assert.Equal("edit_conflict", failure.Code); Assert.Equal("abc", File.ReadAllText(path));
    }

    [Fact]
    public async Task Adjacent_edits_and_empty_range_insertions_use_original_offsets()
    {
        using var directory = new TemporaryDirectory(); var path = directory.File("source.fpas"); File.WriteAllText(path, "integer");
        var plan = await DiagnosticFixPlan.CreateAsync([new(path, "integer")], [Suggest(path, 0, "integer", "int"), Suggest(path, 7, "", ";")]);
        await SourcePublication.ApplyAsync(plan.Changes); Assert.Equal("int;", File.ReadAllText(path));
    }

    [Fact]
    public async Task Invalid_ranges_old_text_and_foreign_sources_are_rejected()
    {
        using var directory = new TemporaryDirectory(); var path = directory.File("source.fpas"); File.WriteAllText(path, "abc");
        foreach (var suggestion in new[] { Suggest(path, -1, "a", "b"), Suggest(path, 3, "a", "b"), Suggest(path, 0, "bad", "b"), Suggest(directory.File("foreign.fpas"), 0, "a", "b") })
            Assert.Equal("invalid_fix", (await Assert.ThrowsAsync<ContextFailure>(() => DiagnosticFixPlan.CreateAsync([new(path, "abc")], [suggestion]))).Code);
        Assert.Equal("abc", File.ReadAllText(path));
    }

    [Fact]
    public async Task Stale_bytes_context_changes_cancellation_and_read_only_files_preserve_the_batch()
    {
        using var directory = new TemporaryDirectory(); var a = directory.File("a.fpas"); var b = directory.File("b.fpas");
        File.WriteAllText(a, "integer"); File.WriteAllText(b, "integer");
        var plan = await DiagnosticFixPlan.CreateAsync([new(a, "integer"), new(b, "integer")], [Suggest(a, 0, "integer", "int"), Suggest(b, 0, "integer", "int")]);
        File.WriteAllText(b, "changed");
        Assert.Equal("inputs_changed", (await Assert.ThrowsAsync<ContextFailure>(() => SourcePublication.ApplyAsync(plan.Changes))).Code);
        Assert.Equal("integer", File.ReadAllText(a)); Assert.Equal("changed", File.ReadAllText(b));
        File.WriteAllText(b, "integer");
        await Assert.ThrowsAsync<ContextFailure>(() => SourcePublication.ApplyAsync(plan.Changes, _ => throw new ContextFailure("inputs_changed", "Project changed.")));
        using var cancelled = new CancellationTokenSource();
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => SourcePublication.ApplyAsync(plan.Changes, _ => { cancelled.Cancel(); return Task.CompletedTask; }, cancelled.Token));
        var attributes = File.GetAttributes(b);
        try
        {
            File.SetAttributes(b, attributes | FileAttributes.ReadOnly);
            await Assert.ThrowsAnyAsync<IOException>(() => SourcePublication.ApplyAsync(plan.Changes));
        }
        finally { File.SetAttributes(b, attributes); }
        Assert.Equal("integer", File.ReadAllText(a)); Assert.Equal("integer", File.ReadAllText(b));
        Assert.Empty(Directory.GetFiles(directory.Path, ".fpas-edit-*"));
    }

    [Fact]
    public async Task Review_only_and_no_op_proposals_do_not_write_and_edit_limits_fail_explicitly()
    {
        using var directory = new TemporaryDirectory(); var path = directory.File("source.fpas"); File.WriteAllText(path, "abc");
        var suggestion = Suggest(path, 0, "abc", "abc");
        var plan = await DiagnosticFixPlan.CreateAsync([new(path, "abc")], [suggestion, Suggest(path, 0, "abc", "def", DiagnosticApplicability.Review)]);
        Assert.Empty(plan.Changes); Assert.Empty(plan.Files);
        Assert.Equal("fix_limit", (await Assert.ThrowsAsync<ContextFailure>(() => DiagnosticFixPlan.CreateAsync([new(path, "abc")], Enumerable.Repeat(suggestion, 1001).ToArray()))).Code);
        File.WriteAllText(path, "stale");
        Assert.Equal("inputs_changed", (await Assert.ThrowsAsync<ContextFailure>(() => DiagnosticFixPlan.CreateAsync([new(path, "abc")], [suggestion]))).Code);
    }

    [Fact]
    public async Task A_missing_staged_file_rolls_back_already_replaced_sources()
    {
        using var directory = new TemporaryDirectory();
        Directory.CreateDirectory(directory.File("a")); Directory.CreateDirectory(directory.File("b"));
        var a = directory.File("a/source.fpas"); var b = directory.File("b/source.fpas");
        File.WriteAllText(a, "integer"); File.WriteAllText(b, "boolean");
        var plan = await DiagnosticFixPlan.CreateAsync([new(a, "integer"), new(b, "boolean")], [Suggest(a, 0, "integer", "int"), Suggest(b, 0, "boolean", "bool")]);
        await Assert.ThrowsAsync<FileNotFoundException>(() => SourcePublication.ApplyAsync(plan.Changes, _ =>
        {
            File.Delete(Directory.GetFiles(directory.File("b"), ".fpas-edit-*.new").Single());
            return Task.CompletedTask;
        }));
        Assert.Equal("integer", File.ReadAllText(a)); Assert.Equal("boolean", File.ReadAllText(b));
        Assert.Empty(Directory.GetFiles(directory.Path, ".fpas-edit-*", SearchOption.AllDirectories));
    }

    [Fact]
    public async Task Failed_staging_cleanup_reports_actual_written_files_and_retained_paths()
    {
        using var directory = new TemporaryDirectory(); var path = directory.File("source.fpas"); File.WriteAllText(path, "integer");
        var plan = await DiagnosticFixPlan.CreateAsync([new(path, "integer")], [Suggest(path, 0, "integer", "int")]);
        var failure = await Assert.ThrowsAsync<SourcePublicationFailure>(() => SourcePublication.ApplyAsync(plan.Changes, _ =>
        {
            var backup = Directory.GetFiles(directory.Path, ".fpas-edit-*.bak").Single();
            File.Delete(backup); Directory.CreateDirectory(backup);
            return Task.CompletedTask;
        }));
        Assert.Equal(new[] { path }, failure.Written); Assert.True(Directory.Exists(Assert.Single(failure.Backups)));
        Assert.Equal("int", File.ReadAllText(path));
    }

    [Fact]
    public async Task Links_are_rejected_before_the_target_is_changed()
    {
        if (OperatingSystem.IsWindows()) return;
        using var directory = new TemporaryDirectory(); var target = directory.File("target.fpas"); File.WriteAllText(target, "integer");
        var link = directory.File("link.fpas"); File.CreateSymbolicLink(link, target);
        Assert.Equal("invalid_argument", (await Assert.ThrowsAsync<ContextFailure>(() => DiagnosticFixPlan.CreateAsync([new(link, "integer")], [Suggest(link, 0, "integer", "int")]))).Code);
        Assert.Equal("integer", File.ReadAllText(target));
    }
}
