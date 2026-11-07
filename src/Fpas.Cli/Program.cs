namespace Fpas.Cli;

internal static class Program
{
    private static Task<int> Main(string[] args) => Profiling.ProfileWorker.IsInvocation(args) ? Task.FromResult(Profiling.ProfileWorker.Run(args)) : Fpas.Debugging.Hosting.PrivateWorker.IsInvocation(args)
        ? Fpas.Debugging.Hosting.PrivateWorker.RunAsync(args[1])
        : Fpas.Debugging.Hosting.PrivateWorker.IsTargetInvocation(args)
        ? Fpas.Debugging.Hosting.PrivateWorker.RunTargetAsync(args[1]) : CliApplication.RunAsync(args);
}
