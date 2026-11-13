#include "../fpas_cui.h"

int fpas_cui_curve_editor(cui_ctx *ui, const char *id, cui_curve_key *keys, int *count, int capacity, int *selected, int *added, int *removed, const fpas_curve_options *settings) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    fpas_own(ui, key, key ^ 0x5C0Eu, 0, 0);
    const bool enabled = (settings->flags & 1) && !cui_is_disabled(ui);
    const int before = *selected;
    const cui_curve_opts options = { .x_min = settings->x_min, .x_max = settings->x_max,
        .y_min = settings->y_min, .y_max = settings->y_max, .width = fpas_extent(settings->width_kind, settings->width),
        .height = settings->height, .x_label = settings->x_label, .y_label = settings->y_label,
        .selected = selected, .snap = (settings->flags & 2) != 0,
        .show_playhead = (settings->flags & 4) != 0, .playhead = settings->playhead };
    cui_push_disabled(ui, !enabled);
    const cui_curve_result result = cui_curve_editor_ex(ui, id, keys, count, capacity, &options);
    cui_pop_disabled(ui);
    fpas_creative_node(ui, root, key);
    *added = result.added;
    *removed = result.removed;
    return enabled && (result.changed || *selected != before);
}
