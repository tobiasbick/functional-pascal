namespace Fpas.Cli;

internal static class Program
{
    private static Task<int> Main(string[] args) => CliApplication.RunAsync(args);
}
