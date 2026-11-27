// [Documentation](../../../../../docs/ui/gui/instruments/dot-chart.md)
#include "../../fpas_cui.h"

int fpas_cui_dot_chart(cui_ctx *ui, const char *id, const float *samples, const int *accent,
    int series, int count, const fpas_dot_options *o, int *index, double *x, float *value) {
    cui_dot_series bands[16];
    for (int i = 0; i < series; ++i) {
        const char *name = o->names ? o->names[i] : NULL;
        bands[i] = (cui_dot_series){ count ? samples + (size_t)i * count : NULL,
            name && *name ? name : NULL, accent[i] != 0 };
    }
    const cui_chart_opts shared = fpas_chart_options_of(ui, &o->chart);
    const cui_dot_chart_opts options = { .width = shared.width, .height = shared.height,
        .min = shared.min, .max = shared.max, .format = shared.format, .no_grid = shared.no_grid,
        .annotations = shared.annotations, .annotation_count = shared.annotation_count,
        .x_min = o->x_min, .x_max = o->x_max, .x_step = o->x_step, .x_units = o->units,
        .x_format = o->x_format, .unit = o->unit, .pitch = o->pitch,
        .x_label = o->x_label, .x_label_user = o->x_label_user,
        .value_label = o->value_label, .value_label_user = o->value_label_user,
        .no_axis = (o->flags & 1) != 0, .no_readout = (o->flags & 2) != 0 };
    const bool enabled = (o->flags & 4) != 0;
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    if (enabled && (o->flags & 8) && !cui_is_disabled(ui)) ui->focus_next_request = key;
    cui_push_disabled(ui, !enabled);
    /* CUI-152 (docs/future/cui-upstream-bugs.md#cui-152): limit pointer activation to the plot. */
    const bool released = ui->mreleased[CUI_MOUSE_LEFT];
    if (released && ui->active == key && cui__has_prev_rect(ui, key)) {
        const float head = cui__ann_headroom(ui, options.annotations, options.annotation_count);
        const cui_rect plot = cui__dm_plot(cui__prev_rect(ui, key), head, !options.no_axis);
        ui->mreleased[CUI_MOUSE_LEFT] = cui__rect_contains(plot, ui->mx, ui->my);
    }
    cui_dot_chart_result result = cui_dot_chart_ex(ui, id, bands, series, count, &options);
    ui->mreleased[CUI_MOUSE_LEFT] = released;
    cui_pop_disabled(ui);
    ui->last_node = node;
    *index = result.hovered;
    *x = result.x;
    *value = result.value;
    return enabled && result.clicked;
}
