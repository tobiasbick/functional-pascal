#include "../fpas_cui.h"

int fpas_cui_dot_chart(cui_ctx *ui, const char *id, const float *samples, const int *accent,
    int series, int count, const fpas_dot_options *o, int *index, double *x, float *value) {
    cui_dot_series bands[16];
    for (int i = 0; i < series; ++i)
        bands[i] = (cui_dot_series){ count ? samples + (size_t)i * count : NULL, NULL, accent[i] != 0 };
    const cui_chart_opts shared = fpas_chart_options_of(ui, &o->chart);
    const cui_dot_chart_opts options = { .width = shared.width, .height = shared.height,
        .min = shared.min, .max = shared.max, .format = shared.format, .no_grid = shared.no_grid,
        .annotations = shared.annotations, .annotation_count = shared.annotation_count,
        .x_min = o->x_min, .x_max = o->x_max, .x_step = o->x_step, .x_units = o->units,
        .x_format = o->x_format, .unit = o->unit, .pitch = o->pitch,
        .no_axis = (o->flags & 1) != 0, .no_readout = (o->flags & 2) != 0 };
    const bool enabled = (o->flags & 4) != 0;
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    if (enabled && (o->flags & 8) && !cui_is_disabled(ui)) ui->focus_next_request = key;
    const bool keyboard = (ui->focus == key || ui->focus_next_request == key) &&
        (cui_key_pressed(ui, CUI_KEY_ENTER) || cui_key_pressed(ui, CUI_KEY_SPACE));
    cui_push_disabled(ui, !enabled);
    cui_dot_chart_result result = cui_dot_chart_ex(ui, id, bands, series, count, &options);
    cui_pop_disabled(ui);
    /* CUI-028 (docs/future/cui-upstream-bugs.md#cui-028): before arrow input,
       keyboard activation leaves the result at zero instead of the center sample. */
    if (keyboard && result.clicked && result.hovered < 0 && count > 0) {
        const cui__dm_paint *paint = ui->nodes[node].paint_data;
        result.hovered = count / 2;
        result.x = count > 1 ? paint->x0 + (paint->x1 - paint->x0) * result.hovered / (count - 1) : paint->x0;
        result.value = cui__dm_total(paint->vals, series, count, result.hovered);
    }
    ui->last_node = node;
    *index = result.hovered;
    *x = result.x;
    *value = result.value;
    return enabled && result.clicked;
}
