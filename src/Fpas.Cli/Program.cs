namespace Fpas.Cli;

internal static class Program
{
    private static Task<int> Main(string[] args) => Profiling.ProfileWorker.IsInvocation(args) ? Task.FromResult(Profiling.ProfileWorker.Run(args)) : Fpas.Debugging.Hosting.PrivateWorker.IsInvocation(args)
        ? Fpas.Debugging.Hosting.PrivateWorker.RunAsync(args[1])
        : Fpas.Debugging.Hosting.PrivateWorker.IsTargetInvocation(args)
        ? Fpas.Debugging.Hosting.PrivateWorker.RunTargetAsync(args[1])
        : args is ["--editor-control", .. var command] ? EditorControl.RunAsync(command)
        : args is ["--editor-run", var payload] ? EditorControl.RunTerminalAsync(payload) : CliApplication.RunAsync(args);
}
