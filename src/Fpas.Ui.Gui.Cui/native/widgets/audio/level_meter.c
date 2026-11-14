#include "../../fpas_cui.h"

int fpas_cui_level_meter(cui_ctx *ui, const char *id, const float *levels,
    int channels, float minimum_db, float maximum_db, float length, float thickness,
    int horizontal, int scale, int segmented, float hold, float decay, int enabled, int *control) {
    const cui_id key = cui_make_id(ui, id);
    const bool allowed = enabled && !cui_is_disabled(ui);
    const cui_level_meter_opts options = {
        .min_db = minimum_db, .max_db = maximum_db, .length = length, .thickness = thickness,
        .horizontal = horizontal != 0, .scale = scale != 0, .segmented = segmented != 0,
        .hold = hold, .decay = decay
    };
    cui_push_disabled(ui, !enabled);
    const bool cleared = cui_level_meter_ex(ui, id, levels, channels, &options);
    cui_pop_disabled(ui);
    *control = ui->last_node = fpas_find(ui, key);
    fpas_own(ui, key, key ^ 0xC11Bu, 0, 0);
    return allowed && cleared;
}
