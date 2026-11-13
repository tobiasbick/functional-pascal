#include "../fpas_cui.h"

int fpas_cui_model_picker(cui_ctx *ui, const char *id, const cui_model *models, int count,
    int *selected, int placement, int flags) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), popup = key ^ 0x30DEu;
    fpas_own(ui, key, 0, 0, popup);
    const bool enabled = fpas_ai_enabled(ui, flags, key);
    if (!enabled) cui__popup_set(ui, popup, false);
    const cui_model_picker_opts opts = { .ghost = (flags & 4) != 0, .placement = (cui_placement)placement };
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_model_picker_ex(ui, id, models, count, selected, &opts);
    cui_pop_disabled(ui);
    fpas_creative_node(ui, root, key);
    return enabled && changed;
}
