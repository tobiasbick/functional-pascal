using Fpas.Compiler;
using Fpas.LanguageServer.Protocol;
using Fpas.Workspaces.Live;
using Fpas.Workspaces.Loading;
using OmniSharp.Extensions.LanguageServer.Protocol.Models;

namespace Fpas.LanguageServer.Hosting;

internal sealed partial class ServerSession
{
    private async Task DiagnosticsLoop()
    {
        try
        {
            await foreach (var change in changes.Reader.ReadAllAsync(lifetime.Token))
            {
                while (changes.Reader.TryRead(out _)) { }
                long selected;
                using (gate.EnterScope()) { if (state != SessionState.Running || desynchronized.Count != 0) continue; selected = revision; }
                LiveSnapshot? snapshot = null;
                try
                {
                    using var deadline = CancellationTokenSource.CreateLinkedTokenSource(lifetime.Token); deadline.CancelAfter(TimeSpan.FromSeconds(30));
                    snapshot = await workspace!.AnalyzeAsync(deadline.Token);
                    await workspace.VerifyAsync(snapshot, deadline.Token);
                    using (gate.EnterScope())
                    {
                        if (!IsCurrent(selected) || desynchronized.Count != 0) continue;
                        var diagnostics = snapshot.Analysis.Projects.SelectMany(project => project.Result?.Diagnostics ?? []).Concat(snapshot.Unowned.SelectMany(document => document.Diagnostics)).Distinct().ToArray();
                        foreach (var document in workspace.OpenDocuments.Documents)
                        {
                            var path = document.Path; var version = document.Version;
                            var values = diagnostics.Where(diagnostic => diagnostic.Location is { } location && PathComparer.Equals(location.Path, path))
                                .Select(diagnostic => DiagnosticProjection.Convert(diagnostic, snapshot.Documents)).ToArray();
                            outgoing.Enqueue("textDocument/publishDiagnostics", new PublishDiagnosticsParams { Uri = FileUris.Uri(path), Version = version, Diagnostics = values },
                                () => IsCurrent(selected));
                        }
                        foreach (var issue in snapshot.Analysis.Context.Issues) Log(issue.Code + ": " + issue.Project + ": " + issue.Message);
                        foreach (var diagnostic in diagnostics.Where(diagnostic => diagnostic.Location is null)) Log(diagnostic.Code + ": " + diagnostic.Message);
                    }
                }
                catch (OperationCanceledException)
                {
                    using (gate.EnterScope()) if (IsCurrent(selected)) { ClearDiagnostics(selected); Log("Analysis was cancelled or exceeded its 30-second work deadline."); }
                }
                catch (ContextFailure error)
                {
                    using (gate.EnterScope())
                    {
                        if (!IsCurrent(selected)) continue;
                        ClearDiagnostics(selected);
                        if (error.Code is "content_modified" or "inputs_changed") { workspace!.Invalidate(); Changed(); }
                        else Log(error.Code + ": " + error.Message);
                    }
                }
                catch (Exception error) when (error is IOException or UnauthorizedAccessException or ArgumentException)
                { using (gate.EnterScope()) if (IsCurrent(selected)) { ClearDiagnostics(selected); Log("Analysis failed: " + error.Message); } }
                finally { snapshot = null; }
            }
        }
        catch (OperationCanceledException) when (lifetime.IsCancellationRequested) { }
        catch (ObjectDisposedException) when (lifetime.IsCancellationRequested) { }
        catch (Exception error)
        { await errors.WriteLineAsync("LSP analysis failed: " + error.Message); Fail(); }
    }

    private void ClearDiagnostics(long selected)
    {
        foreach (var document in workspace!.OpenDocuments.Documents)
            outgoing.Enqueue("textDocument/publishDiagnostics", new PublishDiagnosticsParams { Uri = FileUris.Uri(document.Path), Version = document.Version, Diagnostics = [] }, () => IsCurrent(selected));
    }
}
