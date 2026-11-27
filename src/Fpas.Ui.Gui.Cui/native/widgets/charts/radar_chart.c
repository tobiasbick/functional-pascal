// [Documentation](../../../../../docs/ui/gui/more-charts/radar-chart.md)
#include "../../fpas_cui.h"

static float fpas_radar_legend_extra(cui_ctx *ui, const cui__radar_paint *paint, float width) {
    const float fs = cui__fs(ui, ui->theme.text_xs);
    float total = -16;
    for (int i = 0; i < paint->count; ++i) {
        const char *name = paint->series[i].name;
        total += 30 + cui__text_width(ui, CUI_FONT_BODY, fs, name ? name : "", -1, false);
    }
    if (total <= width) return 0;
    const char **names = cui__frame_alloc(ui, sizeof *names * (size_t)paint->count);
    for (int i = 0; i < paint->count; ++i) names[i] = paint->series[i].name;
    return cui__viz_legend(ui, 0, 0, width + 1, names, NULL, paint->count, -1, false) - 18;
}

static void fpas_paint_radar(cui_ctx *ui, cui_node *node, void *data) {
    const cui__radar_paint *paint = data;
    const float height = paint->size + 52 + 30 + fpas_radar_legend_extra(ui, paint, node->rect.w);
    if (fabsf(height - node->h.value) > 0.5f) cui_request_redraw(ui);
    cui__paint_radar(ui, node, data);
}

int fpas_cui_radar_chart(cui_ctx *ui, const char *id, const fpas_chart_series *series, int count,
    const char *const *axes, int axis_count, float size, float maximum, const char *format, int legend,
    int enabled, int *node) {
    cui_series *copy = fpas_chart_series_of(ui, series, count);
    for (int i = 0; i < count; ++i) copy[i].name = fpas_viz_text(ui, copy[i].name);
    const cui_radar_opts o = { .size = size, .max = maximum, .format = fpas_viz_text(ui, format), .no_legend = !legend };
    *node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const int clicked = cui_radar_chart_ex(ui, id, copy, count, fpas_viz_labels(ui, axes, axis_count), axis_count, &o);
    cui_pop_disabled(ui);
    // CUI-093 (docs/future/cui-upstream-bugs.md#cui-093): reserve the wrapped legend's extra rows.
    if (legend && count >= 2) {
        cui_node *chart = &ui->nodes[*node];
        const cui__radar_paint *paint = chart->paint_data;
        const float previous = cui__prev_rect(ui, chart->id).w;
        chart->h.value += fpas_radar_legend_extra(ui, paint, previous > 0 ? previous : paint->size + 180);
        chart->paint = fpas_paint_radar;
    }
    ui->last_node = *node;
    return clicked;
}
