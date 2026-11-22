// [Documentation](../../coverage.md#terminal-ui)
using System.Text.Json;
using static Fpas.IntegrationTests.Tui.TuiGallerySession;

namespace Fpas.IntegrationTests.Tui;

public sealed partial class TuiGalleryTests
{
    private static async Task CheckChoiceGroups(TuiGallerySession session)
    {
        await session.Click("Board");
        AssertChoice(await session.Request("snapshot"), "view/1", true);
        await session.Key("Right");
        var timeline = await session.Request("snapshot");
        AssertChoice(timeline, "view/1", false);
        AssertChoice(timeline, "view/2", true);
        Assert.Equal("Timeline", Focused(timeline));
        await session.Key("Right");
        Assert.Equal("Timeline", Focused(await session.Request("snapshot")));
        await session.Key("Tab");
        Assert.Equal("Day", Focused(await session.Request("snapshot")));
        await session.Key("End");
        AssertChoice(await session.Request("snapshot"), "range/3", true);

        await session.Click("Actions");
        await session.Key("Right");
        Assert.Equal("Bold", Focused(await session.Request("snapshot")));
        await session.Key("Right");
        var italic = await session.Request("snapshot");
        Assert.Equal("Italic", Focused(italic));
        AssertChoice(italic, "action-format/1", false);
        await session.Turn(new() { ["kind"] = "character", ["text"] = " " });
        var combined = await session.Request("snapshot");
        AssertChoice(combined, "action-format/0", true);
        AssertChoice(combined, "action-format/1", true);
        await session.Click("Bold");
        AssertChoice(await session.Request("snapshot"), "action-format/0", false);
        await session.Key("Tab");
        await session.Key("Right");
        var center = await session.Request("snapshot");
        Assert.Equal("Center", Focused(center));
        AssertChoice(center, "action-align/0", true);
        AssertChoice(center, "action-align/1", false);
        await session.Key("Enter");
        var aligned = await session.Request("snapshot");
        AssertChoice(aligned, "action-align/0", false);
        AssertChoice(aligned, "action-align/1", true);

        await session.Resize(40, 17);
        await session.Turn(new() { ["kind"] = "key", ["key"] = "Tab", ["modifiers"] = 1 });
        await session.Key("End");
        var narrow = await session.Request("snapshot");
        Assert.Equal("Code", Focused(narrow));
        Assert.Contains("Code", Preview(narrow, 13));
        AssertChoice(narrow, "action-format/3", false);
        await session.Resize(144, 36);
    }

    private static void AssertChoice(JsonElement frame, string id, bool selected) =>
        Assert.Equal(selected ? "true" : "false", Control(frame, id).GetProperty("value").GetString());
}
