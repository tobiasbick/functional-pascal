using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's native FeatureFlags widget.</summary>
    /// <param name="Id">Id supplied to the widget.</param>
    /// <param name="Entries">Entries supplied to the widget.</param>
    /// <param name="Environments">Environments supplied to the widget.</param>
    /// <param name="Height">Height supplied to the widget.</param>
    /// <param name="WidthKind">Width Kind supplied to the widget.</param>
    /// <param name="Width">Width supplied to the widget.</param>
    /// <param name="Enabled">Enabled supplied to the widget.</param>
    /// <returns>Native edit or action proposals.</returns>
    public unsafe CuiFlagProposal? FeatureFlags(string Id, CuiDevFlag[] Entries, string[] Environments, float Height, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        if (Environments.Length is < 1 or > 8 || Entries.Any(entry => entry.On.Length != Environments.Length || entry.Rollout.Length != Environments.Length))
            throw new ArgumentException("Each feature flag needs one value per environment.", nameof(Entries));
        using var memory = new ChartMemory();
        var values = new NativeDevFlag[Entries.Length];
        fixed (NativeDevFlag* flags = values)
        {
            for (int i = 0; i < Entries.Length; ++i)
            {
                var entry = Entries[i];
                flags[i].Key = memory.Text(entry.Key);
                flags[i].Description = memory.Text(entry.Description);
                for (int environment = 0; environment < Environments.Length; environment++)
                {
                    flags[i].On[environment] = entry.On[environment] ? (byte)1 : (byte)0;
                    flags[i].Rollout[environment] = entry.Rollout[environment];
                }
            }
            NativeMethods.DevFeatureFlags(context, Stable(Id), flags, Entries.Length, memory.Labels(Environments),
                Environments.Length, Height, WidthKind, Width, Enabled ? 1 : 0, out var result);
            if (!Enabled || result.Index < 0 || result.Action < 0) return null;
            return new(result.Index, result.Action, flags[result.Index].On[result.Action] != 0, flags[result.Index].Rollout[result.Action]);
        }
    }
}
