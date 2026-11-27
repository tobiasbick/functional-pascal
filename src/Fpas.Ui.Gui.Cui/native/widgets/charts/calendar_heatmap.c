// [Documentation](../../../../../docs/ui/gui/more-charts/calendar-heatmap.md)
#include "../../fpas_cui.h"

int fpas_cui_calendar_heatmap(cui_ctx *ui, const char *id, int year, int month, int day,
    const float *values, int count, float cell, int palette, uint32_t rgba, const char *format,
    const char *unit, int enabled, int *node) {
    const cui_date start = {year, month, day};
    const cui_calendar_heatmap_opts o = { .cell = cell, .color = fpas_chart_color(palette, rgba),
        .format = fpas_viz_text(ui, format), .unit = fpas_viz_text(ui, unit) };
    const cui_id hot = ui->hot, key = cui_make_id(ui, id);
    // CUI-087 (docs/future/cui-upstream-bugs.md#cui-087): gaps belong to no date.
    if (hot == key) {
        const cui_rect r = cui__prev_rect(ui, key);
        const float size = cell > 0 ? cell : 11;
        if (fpas_viz_cell_gap(ui, r.x + CUI__CAL_LW, r.y + CUI__CAL_TOP, size, size >= 10 ? 3 : 2))
            ui->hot = 0;
    }
    *node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const int clicked = cui_calendar_heatmap_ex(ui, id, start, values, count, &o);
    cui_pop_disabled(ui);
    ui->hot = hot;
    ui->last_node = *node;
    /* CUI returns an index relative to the visible weeks. Resolve it against the input date. */
    const cui__cal_paint *paint = ui->nodes[*node].paint_data;
    return clicked < 0 ? -1 : paint->base_day - cui_date_to_days(start) + clicked;
}
