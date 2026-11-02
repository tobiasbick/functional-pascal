namespace Demo;

public static class SharedApi
{
    public static string Decorate(string value) => string.Concat("C#:", value);
}
