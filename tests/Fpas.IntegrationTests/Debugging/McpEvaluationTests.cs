using System.Globalization;
using System.Text.Json;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Debugging;

public sealed class McpEvaluationTests
{
    [NetCoreDbgFact]
    public async Task Paged_inspection_and_retained_output_answer_the_debugger_evaluation()
    {
        var fixtures = Path.Combine(AppContext.BaseDirectory, "Fixtures", "debugging");
        var source = File.ReadAllText(Path.Combine(fixtures, "evaluation.fpas"));
        using var program = new Debuggee("mcp-evaluation");
        await program.Compile(source);
        await using var probe = await McpProbe.Create();
        var entry = await probe.Call("debug_launch", new { program = program.Assembly, cwd = program.DirectoryPath });
        var id = entry.GetProperty("sessionId").GetString()!;
        var line = Array.FindIndex(source.Split('\n'), text => text.Contains("Console.WriteLine(Total);", StringComparison.Ordinal)) + 1;
        await probe.Call("debug_set_breakpoints", new { sessionId = id, source = program.Source, lines = new[] { line } });
        await probe.Continue(id, entry);
        var stop = await probe.Stopped(id, entry.GetProperty("stopId").GetInt64());
        var stopId = stop.GetProperty("stopId").GetInt64();
        var locals = (await probe.Locals(id, stop)).ToDictionary(item => item.GetProperty("name").GetString()!);
        async Task<string[]> ArrayValues(string name)
        {
            var values = new List<string>(); var more = true;
            while (more)
            {
                var page = await probe.Call("debug_variables", new { sessionId = id, stopId,
                    variablesHandle = locals[name].GetProperty("handle").GetString(), start = values.Count, count = 2 });
                var data = page.GetProperty("data");
                values.AddRange(data.GetProperty("variables").EnumerateArray().Select(item => item.GetProperty("value").GetString()!));
                more = data.GetProperty("truncated").GetBoolean();
            }
            return values.ToArray();
        }
        var names = (await ArrayValues("Names")).Select(value => JsonSerializer.Deserialize<string>(value)!).ToArray();
        var stock = (await ArrayValues("Stock")).Select(Number).ToArray();
        var demand = (await ArrayValues("Demand")).Select(Number).ToArray();
        var filled = (await ArrayValues("Filled")).Select(Number).ToArray();
        var prices = (await ArrayValues("Prices")).Select(Number).ToArray();
        var indices = Enumerable.Range(0, names.Length).ToArray();
        var output = await probe.Call("debug_output", new { sessionId = id });
        var chunks = output.GetProperty("data").GetProperty("items").EnumerateArray().ToArray();
        var caller = await probe.Call("debug_stack", new { sessionId = id, stopId, start = 1, count = 1 });
        var frame = caller.GetProperty("data").GetProperty("frames")[0].GetProperty("handle").GetString();
        var scopes = await probe.Call("debug_variables", new { sessionId = id, stopId, frameHandle = frame });
        var callerValues = await probe.Call("debug_variables", new { sessionId = id, stopId,
            variablesHandle = scopes.GetProperty("data").GetProperty("scopes")[0].GetProperty("handle").GetString() });
        var callerRegion = callerValues.GetProperty("data").GetProperty("variables").EnumerateArray()
            .Single(item => item.GetProperty("name").GetString() == "Region").GetProperty("value").GetString();
        var region = JsonSerializer.Deserialize<string>(locals["Region"].GetProperty("value").GetString()!)!;
        int Local(string name) => Number(locals[name].GetProperty("value").GetString()!);
        var answers = new Dictionary<string, string>
        {
            ["largest_shortage"] = names[indices.MaxBy(index => demand[index] - filled[index])],
            ["unfilled_total"] = (demand.Sum() - filled.Sum()).ToString(CultureInfo.InvariantCulture),
            ["third_line"] = filled[2].ToString(CultureInfo.InvariantCulture),
            ["largest_remainder"] = names[indices.MaxBy(index => stock[index] - filled[index])],
            ["line_revenue"] = indices.Sum(index => filled[index] * prices[index]).ToString(CultureInfo.InvariantCulture),
            ["net_subtotal"] = (Local("Subtotal") - Local("Discount")).ToString(CultureInfo.InvariantCulture),
            ["charged_total"] = Local("Total").ToString(CultureInfo.InvariantCulture),
            ["largest_line"] = names[indices.MaxBy(index => filled[index] * prices[index])],
            ["audit_category"] = chunks.Single(item => item.GetProperty("text").GetString()!.Contains("audit:", StringComparison.Ordinal)).GetProperty("category").GetString()!,
            ["region_consistency"] = (JsonSerializer.Deserialize<string>(callerRegion!) == region
                && chunks.Any(item => item.GetProperty("text").GetString()!.Contains(" / " + region + " / ", StringComparison.Ordinal))).ToString()
        };
        foreach (var question in XDocument.Load(Path.Combine(fixtures, "evaluation.xml")).Root!.Elements("qa_pair"))
            Assert.Equal(question.Element("answer")!.Value, answers[question.Attribute("id")!.Value]);
    }

    private static int Number(string value) => int.Parse(value, CultureInfo.InvariantCulture);
}
