using System.Runtime.CompilerServices;
using System.Diagnostics.CodeAnalysis;

namespace Demo;

public class Resource : IDisposable
{
    public static string Log { get; private set; } = "";
    public string Name { get; }
    public string? Optional { get; set; }
    public string? OptionalField;
    public int Number { get; set; }
    public int Field;
    public static int StaticField;

    public Resource(string name)
    {
        Name = name;
        Note("open:" + name);
        if (name == "bad-open") throw new InvalidOperationException("acquisition");
    }

    public virtual string Label() => Name;
    public Resource Next() => this;
    public static Resource? Maybe(bool present) => present ? new Resource("maybe") : null;
    public static Counter? MaybeCounter(bool present) => present ? new Counter { Value = 4 } : null;
    public static void Reset() => Log = "";
    public static void Note(string message) => Log += message + ";";
    public static bool Filter(string label) { Note("filter:" + label); return true; }
    public static bool ExplodingFilter() { Note("filter-throw"); throw new Exception("filter"); }
    public static string Overload(object? value) => "object";
    public static string Overload(Resource? value) => "resource";
    public static Resource Require(Resource value) => value;
    [return: MaybeNull]
    public static string MaybeContract() => null!;
    public static void Accept([AllowNull] string value) => Note(value ?? "null");

    [MethodImpl(MethodImplOptions.NoInlining)]
    public static void ThrowOriginal() => throw new InvalidOperationException("original");

    public void Dispose()
    {
        Note("close:" + Name);
        if (Name == "bad-close") throw new InvalidOperationException("cleanup");
    }
}

public sealed class DerivedResource(string name) : Resource(name)
{
    public override string Label() => "derived:" + Name;
}

public struct Counter : IDisposable
{
    public int Value;
    public void Increment() => Value++;
    public void Dispose() => Resource.Note("counter:" + Value);
}
