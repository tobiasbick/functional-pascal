using System.Text.Json;
using System.Text.Json.Serialization;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Hosting;

// Correlation and serialized payloads belong to the private worker channel, not the core interface.
internal sealed record WorkerMessage(long Id, string Kind, string Operation, JsonElement? Payload = null, DebugProblem? Error = null);
internal sealed record BrokerLaunch(string Dotnet, string Program, string Cwd, string[]? Arguments = null, TerminalLaunch? Terminal = null);
internal sealed record BrokerTarget(int ProcessId);
internal sealed record BrokerExit(int Code, int? Signal = null);

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase)]
[JsonSerializable(typeof(WorkerMessage))]
[JsonSerializable(typeof(LaunchTarget))]
[JsonSerializable(typeof(DebugSnapshot))]
[JsonSerializable(typeof(DebugBinding[]))]
[JsonSerializable(typeof(BindingPage))]
[JsonSerializable(typeof(DebugThread[]))]
[JsonSerializable(typeof(StackPage))]
[JsonSerializable(typeof(StackRequest))]
[JsonSerializable(typeof(FrameRequest))]
[JsonSerializable(typeof(ValueRequest))]
[JsonSerializable(typeof(StaticRequest))]
[JsonSerializable(typeof(EvaluationRequest))]
[JsonSerializable(typeof(EvaluationResult))]
[JsonSerializable(typeof(StopRequest))]
[JsonSerializable(typeof(ResumeRequest))]
[JsonSerializable(typeof(BreakpointsRequest))]
[JsonSerializable(typeof(BreakpointInfo[]))]
[JsonSerializable(typeof(ExceptionPolicy))]
[JsonSerializable(typeof(DebugExceptionInfo))]
[JsonSerializable(typeof(BrokerLaunch))]
[JsonSerializable(typeof(BrokerTarget))]
[JsonSerializable(typeof(BrokerExit))]
[JsonSerializable(typeof(TerminalSize))]
[JsonSerializable(typeof(byte[]))]
internal sealed partial class WorkerJson : JsonSerializerContext;
