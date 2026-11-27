// [Documentation](../../../../../docs/ui/gui/editing-tools/gradient-editor.md)
#include "../../fpas_cui.h"

/** Samples CUI's sRGB ramp and returns packed straight RGBA across the ABI. */
CUI_API uint32_t fpas_cui_gradient_eval(const cui_gradient_stop *stops, int count, float position) {
    const cui_color color = cui_gradient_eval(stops, count, position);
    return color.r | ((uint32_t)color.g << 8) | ((uint32_t)color.b << 16) | ((uint32_t)color.a << 24);
}

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
    return enabled ? (result.changed ? 3 : *selected != before ? 1 : 0) : 0;
}
