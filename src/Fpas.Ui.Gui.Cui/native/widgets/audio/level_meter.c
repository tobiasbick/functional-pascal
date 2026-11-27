// [Documentation](../../../../../docs/ui/gui/audio/level-meter.md)
#include "../../fpas_cui.h"

/* [CUI-115](../../../../../docs/future/cui-upstream-bugs.md#cui-115):
 * Removed channels must not leave a clip or a press on a hidden indicator. */
static void forget_removed_meter_channels(cui_ctx *ui, cui_id meter, cui_id clip, int channels, float minimum) {
    cui__meter_state *state = cui__state_lazy(ui, meter, sizeof *state, false);
    if (!state) return;
    const int count = CUI_CLAMP(channels, 1, CUI__METER_MAXCH);
    const float floor_db = (minimum < 0 ? minimum : -60) - 1;
    for (int channel = count; channel < CUI__METER_MAXCH; ++channel)
        state->ch[channel] = (cui__meter_ch){ .shown = floor_db, .peak = floor_db };

    if (ui->active == clip) {
        for (int channel = 0; channel < count; ++channel) {
            if (state->ch[channel].clip) return;
        }
        ui->active = 0;
    }
}

int fpas_cui_level_meter(cui_ctx *ui, const char *id, const float *levels,
    int channels, float minimum_db, float maximum_db, float length, float thickness,
    int horizontal, int scale, int segmented, float hold, float decay, int enabled, int *control) {
    const cui_id key = cui_make_id(ui, id);
    const cui_id clip = key ^ 0xC11Bu;
    const bool allowed = enabled && !cui_is_disabled(ui);
    const cui_level_meter_opts options = {
        .min_db = minimum_db, .max_db = maximum_db, .length = length, .thickness = thickness,
        .horizontal = horizontal != 0, .scale = scale != 0, .segmented = segmented != 0,
        .hold = hold, .decay = decay
    };
    forget_removed_meter_channels(ui, key, clip, channels, minimum_db);
    cui_push_disabled(ui, !enabled);
    const bool cleared = cui_level_meter_ex(ui, id, levels, channels, &options);
    cui_pop_disabled(ui);
    *control = ui->last_node = fpas_find(ui, key);
    /* [CUI-117](../../../../../docs/future/cui-upstream-bugs.md#cui-117):
     * The meter's paint puts the loud end on the right, even in an RTL layout. */
    if (*control >= 0 && horizontal) cui__keep_ltr(ui, *control);
    fpas_own(ui, key, clip, 0, 0);
    return allowed && cleared;
}
