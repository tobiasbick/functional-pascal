using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Native data for LiveCursors.</summary>
/// <param name="Name">Name supplied to CUI.</param>
/// <param name="Tone">Tone supplied to CUI.</param>
/// <param name="X">X supplied to CUI.</param>
/// <param name="Y">Y supplied to CUI.</param>
/// <param name="Says">Says supplied to CUI.</param>
/// <param name="Speaking">Speaking supplied to CUI.</param>
/// <param name="Away">Away supplied to CUI.</param>
/// <param name="Rgba">Rgba supplied to CUI.</param>
public sealed record CuiLivePerson(string Name, int Tone, float X, float Y, string? Says, bool Speaking, bool Away, uint Rgba);

public sealed partial class CuiWindow
{
    /// <summary>Invokes the native LiveCursors operation.</summary>
    /// <param name="id">id supplied to the native operation.</param>
    /// <param name="people">people supplied to the native operation.</param>
    /// <param name="transform">transform supplied to the native operation.</param>
    /// <param name="view">view supplied to the native operation.</param>
    /// <param name="noGlide">noGlide supplied to the native operation.</param>
    public unsafe void LiveCursors(string id, CuiLivePerson[] people, bool transform, CuiNodeView view, bool noGlide)
    {
        CheckFrame();
        using var memory = new ChartMemory();
        NativeMethods.LiveCursors(context, Stable(id), PackPeople(memory, people), people.Length,
            transform ? 1 : 0, view, noGlide ? 1 : 0);
    }
    private static unsafe NativeLivePerson* PackPeople(ChartMemory memory, CuiLivePerson[] people)
    {
        var native = Array.ConvertAll(people, person => new NativeLivePerson
        {
            Name = memory.Text(person.Name), Rgba = person.Rgba, Tone = person.Tone,
            X = person.X, Y = person.Y, Says = memory.Text(person.Says),
            Speaking = person.Speaking ? (byte)1 : (byte)0, Away = person.Away ? (byte)1 : (byte)0
        });
        return memory.Copy<NativeLivePerson>(native);
    }

}
