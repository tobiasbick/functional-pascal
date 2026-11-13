#include "../fpas_cui.h"

int fpas_cui_control_tile(cui_ctx *ui, const char *id, const char *label, const char *icon,
    const char *detail, int *on, int width_kind, float width, int palette, uint32_t rgba, int flags) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_tile_opts options = { .icon = icon, .detail = detail, .width = fpas_extent(width_kind, width),
        .color = fpas_chart_color(palette, rgba), .wide = (flags & 1) != 0 };
    bool active = *on != 0;
    if ((flags & 6) == 6 && !cui_is_disabled(ui)) ui->focus_next_request = key;
    cui_push_disabled(ui, (flags & 2) == 0);
    const bool changed = cui_control_tile_ex(ui, id, &active, &options);
    cui_pop_disabled(ui);
    fpas_text(ui, fpas_label_node(ui, node), label);
    fpas_motion_node(ui, node, key);
    *on = active;
    return changed;
}
