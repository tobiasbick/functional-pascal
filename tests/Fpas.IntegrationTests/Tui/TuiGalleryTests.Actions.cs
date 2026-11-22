// [Documentation](../../coverage.md#terminal-ui)
using System.Text.Json;
using static Fpas.IntegrationTests.Tui.TuiGallerySession;

namespace Fpas.IntegrationTests.Tui;

public sealed partial class TuiGalleryTests
{
    private static async Task CheckKeyboardPreview(TuiGallerySession session)
    {
        await session.Key("Down");
        await session.Key("Enter");
        Assert.Equal("Buttons", Selected(await session.Request("snapshot")));
        await session.Key("F6");
        var top = await session.Request("snapshot");
        Assert.True(Control(top, "preview-Buttons").GetProperty("focused").GetBoolean());
        Assert.DoesNotContain("Segmented control", Preview(top));
        await session.Key("End");
        var bottom = await session.Request("snapshot");
        Assert.True(Control(bottom, "preview-Buttons").GetProperty("focused").GetBoolean());
        Assert.Contains("Segmented control", Preview(bottom));

        await session.Key("Tab");
        for (int step = 0; step < 30 && Focused(await session.Request("snapshot")) != "List"; step++)
            await session.Key("Tab");
        Assert.Equal("List", Focused(await session.Request("snapshot")));
        await session.Key("Right");
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

        await session.Key("Escape");
        await session.Key("Home");
        await session.Key("Enter");
    }

    private static void AssertChoice(JsonElement frame, string id, bool selected) =>
        Assert.Equal(selected ? "true" : "false", Control(frame, id).GetProperty("value").GetString());
}
