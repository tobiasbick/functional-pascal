using System.Collections;

namespace Demo;

public sealed class Probe<T>(string name, IEnumerable<T> values) : IEnumerable<T>
{
    public IEnumerator<T> GetEnumerator()
    {
        Resource.Note("acquire:" + name);
        if (name == "acquire-error") throw new InvalidOperationException(name);
        return new Enumerator(name, values.GetEnumerator());
    }

    IEnumerator IEnumerable.GetEnumerator() => GetEnumerator();

    private sealed class Enumerator(string name, IEnumerator<T> inner) : IEnumerator<T>
    {
        public T Current
        {
            get
            {
                Resource.Note("current:" + name);
                if (name == "current-error") throw new InvalidOperationException(name);
                return inner.Current;
            }
        }
        object? IEnumerator.Current => Current;
        public bool MoveNext()
        {
            Resource.Note("move:" + name);
            if (name == "move-error") throw new InvalidOperationException(name);
            return inner.MoveNext();
        }
        public void Reset() => throw new NotSupportedException();
        public void Dispose()
        {
            Resource.Note("dispose:" + name);
            inner.Dispose();
            if (name == "dispose-error") throw new InvalidOperationException(name);
        }
    }
}

public sealed class Legacy(bool disposable) : IEnumerable
{
    public IEnumerator GetEnumerator() => disposable ? new DisposableEnumerator() : new Enumerator();
    private class Enumerator : IEnumerator
    {
        private int index;
        public object? Current => index == 1 ? null : "legacy";
        public bool MoveNext() => ++index <= 2;
        public void Reset() => throw new NotSupportedException();
    }
    private sealed class DisposableEnumerator : Enumerator, IDisposable
    {
        public void Dispose() => Resource.Note("legacy-dispose");
    }
}

public class Box<T>(T value)
{
    public T Field = value;
    public T Value { get; set; } = value;
    public T[] Repeat() => [Field, Value];
    public T? Maybe() => default;
}

public sealed class DerivedBox<T>(T value) : Box<T>(value);
public sealed class NeedsClass<T> where T : class;
public sealed class NeedsValue<T> where T : struct;
public sealed class NeedsNew<T> where T : new();
public sealed class NeedsDisposable<T> where T : IDisposable;
public sealed class NeedsComparable<T> where T : IComparable<T>;

public static class CollectionApi
{
    public static List<string?>? NullableList(bool present) => present ? [null, "text"] : null;
    public static List<string?[]> Nested() => [[null, "text"]];
    public static string?[] NullableArray() => [null, "text"];
    public static List<int?> NullableNumbers() => [null, 5];
    public static Dictionary<string, List<string?[]>> Echo(Dictionary<string, List<string?[]>> value) => value;
    public static KeyValuePair<string, string?>? EchoPair(KeyValuePair<string, string?>? value) => value;
    public static string Overload(IEnumerable<string> value) => "sequence";
    public static string Overload(object value) => "object";
    public static int[] ArrayField = [1, 2];
    public static string[] NonNullArray() => ["known"];
}

public sealed class IndexProbe
{
    public static string Log { get; private set; } = "";
    public int this[int x, int y]
    {
        get => x + y;
        set => Log += "set:" + x + ":" + y + ":" + value + ";";
    }
    public static IndexProbe Receiver() { Log += "receiver;"; return new(); }
    public static int Index(int value) { Log += "index:" + value + ";"; return value; }
    public static int Value() { Log += "value;"; return 9; }
}
