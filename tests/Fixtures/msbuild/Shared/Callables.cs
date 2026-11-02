namespace Demo;

public delegate string Formatter(int value);

public static class CallableApi
{
    public static T Identity<T>(T value) => value;
    public static U Apply<T, U>(T value, Func<T, U> transform) => transform(value);
    public static string Format(Formatter formatter) => formatter(7);
    public static string Run(Action<string> action) { action("callback"); return "done"; }
    public static T Create<T>() where T : new() => new();
    public static int Dispose<T>(T value) where T : IDisposable { value.Dispose(); return 1; }
    public static T Require<T>(T value) where T : notnull => value;
    public static string Ambiguous(Func<int, bool> predicate) => "func";
    public static string Ambiguous(Predicate<int> predicate) => "predicate";
    public static string Overload(Formatter formatter) => "formatter:" + formatter(8);
    public static string Overload(Action<int> action) => "action";
    public static string NullCallbacks(Func<string?, string?> transform) => transform(null) ?? "null";
    public static Func<string, string> MakePrefix(string prefix) => value => prefix + value;
    public static string ReceiverLog { get; private set; } = "";
    public static VirtualTransform Receiver() { ReceiverLog += "receiver;"; return new DerivedTransform(); }
    public static Func<int, int> ReadTransform { get { ReceiverLog += "getter;"; return value => value + 10; } }
}

public sealed class MethodOwner<T>(T value)
{
    public U Convert<U>(Func<T, U> transform) => transform(value);
    public U Echo<U>(U value) => value;
}

public class VirtualTransform
{
    public virtual int Transform(int value) => value;
}
public sealed class DerivedTransform : VirtualTransform
{
    public override int Transform(int value) => value * 3;
}

public sealed class DelegateHolder(Func<int, int> transform)
{
    public Func<int, int> Transform { get; } = transform;
}
