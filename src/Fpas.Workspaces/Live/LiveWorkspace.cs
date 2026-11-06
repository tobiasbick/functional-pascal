using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Snapshots;

namespace Fpas.Workspaces.Live;

/// <summary>Owns document mutations, analysis generations and bounded project reuse. Never publishes files.</summary>
public sealed class LiveWorkspace : IAsyncDisposable
{
    private readonly object gate = new();
    private readonly object identity = new();
    private readonly ContextOptions options;
    private readonly LiveWorkspaceLimits limits;
    private readonly Dictionary<string, TextDocumentSnapshot> opened = new(WorkspaceTextSnapshot.Paths);
    private readonly HashSet<Task> owned = [];
    private readonly SemaphoreSlim analysisGate = new(1);
    private readonly ProjectAnalysisCache cache;
    private CancellationTokenSource generation = new();
    private Task<LiveSnapshot>? pending;
    private Task? disposal;
    private LiveSnapshot? current;
    private long revision;
    private bool disposed;
    public long Revision { get { lock (gate) return revision; } }
    public LiveSnapshot? Current { get { lock (gate) return current; } }
    public WorkspaceTextSnapshot OpenDocuments { get { lock (gate) { Check(); return new(opened.Values); } } }

    public LiveWorkspace(ContextOptions input, LiveWorkspaceLimits? limits = null)
    {
        options = input with { Context = Path.GetFullPath(input.Context), NoRestore = true, Preparation = ContextPreparation.SourceSnapshots };
        this.limits = limits ?? new();
        if (this.limits.DocumentBytes <= 0 || this.limits.OpenBytes <= 0 || this.limits.RetainedBytes <= 0 ||
            this.limits.WorkTimeout is { } timeout && timeout <= TimeSpan.Zero) throw new ArgumentOutOfRangeException(nameof(limits));
        cache = new(this.limits.RetainedBytes);
    }

    public void Open(string path, string text, int version)
    {
        var document = new TextDocumentSnapshot(path, text, version); DocumentChanges.CheckSize(text, limits);
        lock (gate)
        {
            Check(); if (opened.ContainsKey(document.Path)) throw new ContextFailure("invalid_document", "The document is already open.");
            CheckBudget(document); opened.Add(document.Path, document); Changed();
        }
    }

    public void Change(string path, int version, IReadOnlyList<TextChange> changes)
    {
        lock (gate)
        {
            Check(); var original = Document(path); var document = DocumentChanges.Apply(original, version, changes, limits);
            CheckBudget(document); opened[document.Path] = document; Changed();
        }
    }

    public void Save(string path) { lock (gate) { Check(); _ = Document(path); Changed(); } }
    public void Close(string path) { lock (gate) { Check(); var document = Document(path); opened.Remove(document.Path); Changed(); } }
    public void Invalidate() { lock (gate) { Check(); Changed(); } }

    public Task<LiveSnapshot> AnalyzeAsync(CancellationToken token = default)
    {
        lock (gate)
        {
            Check(); token.ThrowIfCancellationRequested();
            if (pending is not null) return pending.WaitAsync(token);
            if (owned.Count >= 64) throw new ContextFailure("workspace_limit", "Too many outstanding workspace operations.");
            var captured = new WorkspaceTextSnapshot(opened.Values); var selected = revision;
            var cancellation = CancellationTokenSource.CreateLinkedTokenSource(generation.Token, token);
            cancellation.CancelAfter(limits.WorkTimeout ?? TimeSpan.FromSeconds(30));
            var work = Task.Run(() => Analyze(selected, captured, cancellation)); pending = work; owned.Add(work);
            _ = work.ContinueWith(completed =>
            {
                lock (gate) { owned.Remove(completed); if (!completed.IsCompletedSuccessfully && ReferenceEquals(pending, completed)) pending = null; }
                _ = completed.Exception;
            }, CancellationToken.None, TaskContinuationOptions.ExecuteSynchronously, TaskScheduler.Default);
            return work;
        }
    }

    public Task VerifyAsync(LiveSnapshot snapshot, CancellationToken token = default)
    {
        lock (gate)
        {
            CheckSnapshot(snapshot); token.ThrowIfCancellationRequested();
            if (owned.Count >= 64) throw new ContextFailure("workspace_limit", "Too many outstanding workspace operations.");
            var cancellation = CancellationTokenSource.CreateLinkedTokenSource(generation.Token, token);
            cancellation.CancelAfter(limits.WorkTimeout ?? TimeSpan.FromSeconds(30));
            var work = VerifyInputs(snapshot, new(opened.Values), cancellation); owned.Add(work);
            _ = work.ContinueWith(completed => { lock (gate) owned.Remove(completed); _ = completed.Exception; },
                CancellationToken.None, TaskContinuationOptions.ExecuteSynchronously, TaskScheduler.Default);
            return work;
        }
    }

    private async Task VerifyInputs(LiveSnapshot snapshot, WorkspaceTextSnapshot overlays, CancellationTokenSource cancellation)
    {
        using (cancellation)
        {
            try
            {
                var context = await WorkspaceLoader.LoadAsync(options, cancellation.Token);
                if (context.InputIdentity(overlays.Documents.Select(document => document.Path)) != snapshot.InputIdentity)
                    throw new ContextFailure("content_modified", "Workspace files or project inputs changed after analysis.");
                lock (gate) CheckSnapshot(snapshot);
            }
            catch
            {
                lock (gate) if (!disposed && revision == snapshot.Revision) Changed();
                throw;
            }
        }
    }

    private async Task<LiveSnapshot> Analyze(long selected, WorkspaceTextSnapshot overlays, CancellationTokenSource cancellation)
    {
        using (cancellation)
        {
            var token = cancellation.Token;
            await analysisGate.WaitAsync(token);
            try
            {
                var context = await WorkspaceLoader.LoadAsync(options, token);
                var identity = context.InputIdentity(overlays.Documents.Select(document => document.Path));
                var byPath = overlays.Documents.ToDictionary(document => document.Path, WorkspaceTextSnapshot.Paths);
                var texts = new WorkspaceTextSnapshot(context.Texts.Documents.Select(document => byPath.GetValueOrDefault(document.Path) ?? document));
                long referenceBytes = 0;
                foreach (var reference in context.Projects.SelectMany(project => project.Request.References).Distinct(WorkspaceTextSnapshot.Paths))
                {
                    if (context.DependencyOutputs.Any(output => WorkspaceTextSnapshot.Paths.Equals(output.Path, reference) && context.Projects.Any(project => WorkspaceTextSnapshot.Paths.Equals(project.Path, output.Project)))) continue;
                    if ((referenceBytes += new FileInfo(reference).Length) > limits.RetainedBytes) throw new ContextFailure("workspace_limit", "Reference images exceed the retained workspace budget.");
                }
                var hits = cache.Hits;
                var analysis = await Task.Run(() => SourceGraphAnalysis.Analyze(context, texts, cache, token), token);
                context.VerifyFiles(token, overlays.Documents.Select(document => document.Path));
                var all = new WorkspaceTextSnapshot(texts.Documents.Concat(overlays.Documents.Where(document => !texts.Documents.Any(owned => WorkspaceTextSnapshot.Paths.Equals(owned.Path, document.Path)))));
                var unowned = overlays.Documents.Where(document => !texts.Documents.Any(owned => WorkspaceTextSnapshot.Paths.Equals(owned.Path, document.Path)))
                    .Select(document => new UnownedDocument(document, SyntaxAnalysis.Check(new(document.Path, document.Text), token))).ToArray();
                var retained = analysis.Context.Projects.SelectMany(project => project.Request.ReferenceImages ?? []).DistinctBy(image => image.Path).Sum(image => (long)image.Length) +
                    analysis.Projects.Sum(project => project.Result is { } result ? ProjectAnalysisCache.Measure(result, null) : 0) + all.Documents.Sum(document => 2L * document.Text.Length);
                if (retained > limits.RetainedBytes) { cache.Clear(); throw new ContextFailure("workspace_limit", "Retained workspace data exceeds its budget."); }
                cache.Trim(limits.RetainedBytes - retained); retained += cache.RetainedBytes;
                token.ThrowIfCancellationRequested();
                var result = new LiveSnapshot(this.identity, selected, identity, analysis, all, unowned, cache.Hits - hits, retained);
                lock (gate)
                {
                    Check(); if (revision != selected) throw new ContextFailure("content_modified", "Analysis belongs to a superseded document revision.");
                    current = result;
                }
                return result;
            }
            finally { analysisGate.Release(); }
        }
    }

    private void CheckBudget(TextDocumentSnapshot document)
    {
        if (opened.Where(pair => !WorkspaceTextSnapshot.Paths.Equals(pair.Key, document.Path)).Sum(pair => (long)DocumentChanges.Bytes(pair.Value.Text)) +
            DocumentChanges.Bytes(document.Text) > limits.OpenBytes) throw new ContextFailure("workspace_limit", "Open text exceeds the session budget.");
    }
    private TextDocumentSnapshot Document(string path) => opened.GetValueOrDefault(Path.GetFullPath(path)) ?? throw new ContextFailure("invalid_document", "The document is not open: " + path);
    private void Check() => ObjectDisposedException.ThrowIf(disposed, this);
    private void CheckSnapshot(LiveSnapshot snapshot)
    {
        Check(); if (!ReferenceEquals(snapshot.Owner, identity) || snapshot.Revision != revision || !ReferenceEquals(current, snapshot))
            throw new ContextFailure("content_modified", "The result is not current in this workspace.");
    }
    private void Changed()
    {
        revision++; current = null; pending = null;
        generation.Cancel(); generation.Dispose(); generation = new();
    }

    public ValueTask DisposeAsync()
    {
        lock (gate)
        {
            if (disposal is not null) return new(disposal);
            disposed = true; current = null; pending = null; generation.Cancel();
            return new(disposal = DisposeOwned(owned.ToArray()));
        }
    }
    private async Task DisposeOwned(Task[] work)
    {
        try { await Task.WhenAll(work); } catch (Exception) { /* Each operation retains its own failure. */ }
        lock (gate) { opened.Clear(); cache.Clear(); generation.Dispose(); analysisGate.Dispose(); }
    }
}
