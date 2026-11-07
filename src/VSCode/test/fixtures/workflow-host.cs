using System;
using System.Diagnostics;
using System.IO;
using System.Reflection;
using System.Text;
using System.Text.Json;
using System.Threading;

Console.InputEncoding = Encoding.UTF8;
Console.OutputEncoding = Encoding.UTF8;
if (args[0] == "child")
{
    File.WriteAllText(args[1], Environment.ProcessId.ToString());
    Thread.Sleep(60000);
    return 0;
}
if (args[0] == "wait")
{
    var start = new ProcessStartInfo(Environment.ProcessPath!) { UseShellExecute = false };
    start.ArgumentList.Add(Assembly.GetExecutingAssembly().Location);
    start.ArgumentList.Add("child");
    start.ArgumentList.Add(args[1] + ".child");
    using var child = Process.Start(start)!;
    File.WriteAllText(args[1], Environment.ProcessId.ToString());
    Thread.Sleep(60000);
    return 0;
}
if (args[0] == "debug") File.WriteAllText(args[1], Environment.ProcessId.ToString());
var result = Engine.Compute(21);
if (args[0] == "debug")
{
    try { Engine.Fail(); }
    catch (InvalidOperationException error) { Console.WriteLine(error.Message); }
    return 0;
}
#if DEBUG
const string configuration = "Debug";
#else
const string configuration = "Release";
#endif
var receipt = JsonSerializer.Serialize(new { input = Console.ReadLine(), argument = args[2], cwd = Environment.CurrentDirectory, result, configuration });
File.WriteAllText(args[1], receipt);
Console.WriteLine(receipt);
return 7;
