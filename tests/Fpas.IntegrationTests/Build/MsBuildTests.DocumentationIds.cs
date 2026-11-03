using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Documentation_ids_match_an_equivalent_CSharp_API()
    {
        using var project = sdk.CreateProject();
        var shared = XDocument.Load(project.File("Shared/Shared.csproj"));
        shared.Root!.Add(new XElement("PropertyGroup", new XElement("GenerateDocumentationFile", "true"), new XElement("NoWarn", "1591")));
        shared.Save(project.File("Shared/Shared.csproj"));
        File.WriteAllText(project.File("Shared/Oracle.cs"), """
            namespace Demo;
            /// <summary>Container.</summary>
            public static class Oracle
            {
                /// <summary>Box.</summary>
                public sealed class Box<T>
                {
                    /// <summary>Value.</summary>
                    public T Value => default!;
                    /// <summary>Copy.</summary>
                    public U Copy<U>(ref T item, out T previous, in U other) { previous = item; return other; }
                }
                /// <summary>Outcome.</summary>
                public abstract class Outcome<T>
                {
                    /// <summary>Empty.</summary>
                    public static Outcome<T> None => null!;
                    /// <summary>Present.</summary>
                    public static Outcome<T> Some(T value) => null!;
                }
                /// <summary>Read.</summary>
                public static int Read(int value) => value;
                /// <summary>Read.</summary>
                public static T Read<T>(T value) => value;
                /// <summary>Arguments.</summary>
                public static void Arguments(Box<int> box, int[,][] grid, string?[]? text, double? number, Func<int, string> callback) { }
            }
            """);
        var game = XDocument.Load(project.File("Game/Game.fpasproj"));
        game.Root!.Element("PropertyGroup")!.Add(new XElement("GenerateDocumentationFile", "true")); game.Save(project.File("Game/Game.fpasproj"));
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            // Container.
            unit Demo.Api;
            uses System as Sys, System.Collections.Generic as Collections;
            // Box.
            public type Box<T> = record
              // Value.
              Value: T;
              // Copy.
              function Copy<U>(ref item: T; out previous: T; in other: U): U;
              begin previous := item; return other; end function;
            end record;
            // Outcome.
            public type Outcome<T> = union
              // Empty.
              None;
              // Present.
              Some(value: T);
            end union;
            // Read.
            public function Read(value: int): int; begin return value; end function;
            // Read.
            public function Read<T>(value: T): T; begin return value; end function;
            // Arguments.
            public procedure Arguments(box: Box<int>; grid: int[][,]; text: string?[]?; number: double?; callback: function(int): string);
            begin end procedure;
            end unit;
            """);
        var build = await sdk.Run(project, "build", "Game/Game.fpasproj"); Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var actual = XDocument.Load(project.File("Game/bin/Debug/net10.0/Game.xml")).Descendants("member").Select(member => (string)member.Attribute("name")!).ToArray();
        var expected = XDocument.Load(project.File("Shared/bin/Debug/net10.0/Shared.xml")).Descendants("member")
            .Select(member => ((string)member.Attribute("name")!).Replace("Demo.Oracle", "Demo.Api", StringComparison.Ordinal)).Order(StringComparer.Ordinal).ToArray();
        Assert.Equal(expected, actual);
    }
}
