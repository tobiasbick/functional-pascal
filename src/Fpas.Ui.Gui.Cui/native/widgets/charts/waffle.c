// [Documentation](../../../../../docs/ui/gui/more-charts/waffle.md)
#include "../../fpas_cui.h"

int fpas_cui_waffle(cui_ctx *ui, const char *id, const fpas_viz_item *items, int count,
    float cell, int columns, int rows, const char *format, int enabled, int *node) {
    const cui_waffle_opts o = { .cell = cell, .columns = columns, .rows = rows, .format = fpas_viz_text(ui, format) };
    const cui_id hot = ui->hot, key = cui_make_id(ui, id);
    /* [CUI-089](../../../../../docs/future/cui-upstream-bugs.md#cui-089): a gutter belongs to no square. */
    if (hot == key) {
        const cui_rect r = cui__prev_rect(ui, key);
        const float size = cell > 0 ? cell : 18, gap = size >= 12 ? 3 : 2, step = size + gap;
        const int wide = columns > 0 ? CUI_MIN(columns, 60) : 10;
        const int high = rows > 0 ? CUI_MIN(rows, 60) : 10;
        if (ui->mx < r.x + wide * step && ui->my < r.y + high * step &&
            fpas_viz_cell_gap(ui, r.x, r.y, size, gap)) ui->hot = 0;
    }
    *node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const int clicked = cui_waffle_ex(ui, id, fpas_viz_items(ui, items, count), count, &o);
    cui_pop_disabled(ui);
    ui->hot = hot;
    ui->last_node = *node;
    return clicked;
}
