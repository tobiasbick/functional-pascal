#include "../fpas_cui.h"

int fpas_cui_canvas_controls(cui_ctx *ui, const char *id, cui_node_view *view, cui_rect content,
    const float *steps, int step_count, const fpas_canvas_toggle *toggles, bool *values, int count, int flags) {
    cui_canvas_toggle *native = cui__frame_alloc(ui, (size_t)CUI_MAX(1, count) * sizeof *native);
    for (int k = 0; k < count; ++k) native[k] = (cui_canvas_toggle){ toggles[k].icon, toggles[k].label, &values[k] };
    const int anchor = ui->last_node;
    const cui_id anchor_key = ui->last_id;
    const cui_canvas_controls_opts options = { .steps = steps, .step_count = step_count,
        .content = content, .toggles = native, .toggle_count = count,
        .no_fit = (flags & 2) != 0, .start_side = (flags & 4) != 0 };
    cui_push_disabled(ui, !(flags & 1));
    const int result = cui_canvas_controls_ex(ui, id, view, &options);
    cui_pop_disabled(ui);
    ui->last_node = anchor;
    ui->last_id = anchor_key;
    return flags & 1 ? result : 0;
}
