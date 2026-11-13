#include "../fpas_cui.h"

int fpas_cui_veil_begin(cui_ctx *ui, const char *id, int revealed, const char *label, const char *hint, float radius, uint32_t color, int width_kind, float width, int flags) {
    bool shown = revealed != 0;
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const cui_id key = cui_make_id(ui, id);
    if (enabled && (flags & 4)) ui->focus_next_request = key;
    const cui_veil_opts options = { .label = label, .hint = hint, .radius = radius,
        .toggle = (flags & 2) != 0, .color = fpas_chart_color(-1, color), .width = fpas_extent(width_kind, width) };
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_veil_begin_ex(ui, id, &shown, &options);
    if (!enabled) {
        /* CUI-038: both hit registrations bypass the native disabled stack. */
        cui_node *node = &ui->nodes[cui__current(ui)];
        node->flags &= ~(CUI_NF_INTERACTIVE | CUI_NF_FOCUSABLE);
        ((cui__veil_paint *)node->paint_data)->catch_clicks = false;
    }
    return (shown ? 1 : 0) | (enabled && changed ? 2 : 0);
}

void fpas_cui_veil_end(cui_ctx *ui) {
    const cui__veil_build *building = ui->building.veil;
    const cui_id key = building ? building->id : 0;
    cui_veil_end(ui);
    cui_pop_disabled(ui);
    ui->last_node = fpas_find(ui, key);
}
