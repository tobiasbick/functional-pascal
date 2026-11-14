#include "../../fpas_cui.h"

int fpas_cui_gradient_editor(cui_ctx *ui, const char *id, cui_gradient_stop *stops, int *count, int capacity, int *selected, int *added, int *removed, int width_kind, float width, float height, int enabled) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    fpas_own(ui, key, key ^ 0x6E5Du, 0, 0);
    const int before = *selected;
    const cui_gradient_opts options = { .width = fpas_extent(width_kind, width), .height = height, .selected = selected };
    cui_push_disabled(ui, !enabled);
    const cui_gradient_result result = cui_gradient_editor_ex(ui, id, stops, count, capacity, &options);
    cui_pop_disabled(ui);
    fpas_creative_node(ui, root, key);
    *added = result.added;
    *removed = result.removed;
    return enabled && (result.changed || *selected != before);
}
