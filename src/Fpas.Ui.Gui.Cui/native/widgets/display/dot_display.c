// [Documentation](../../../../../docs/ui/gui/instruments/dot-display.md)
#include "../../fpas_cui.h"

/** Requests a frame when a marquee overflows its current layout width. */
static void fpas_paint_dot_display(cui_ctx *ui, cui_node *node, void *data) {
    const cui__dmf_paint *paint = data;
    if (paint->n > 0 && (paint->n * 6 - 1) * paint->pitch > node->rect.w + 0.5f)
        cui_request_wakeup(ui, 1.0f / 30.0f);
    cui__paint_dot_display(ui, node, data);
}

/** Draws native dot-matrix text with a stable widget ID. */
void fpas_cui_dot_display(cui_ctx *ui, const char *id, const char *text, float pitch,
    int palette, uint32_t rgba, int align, int width_kind, float width, int flags) {
    const int node = ui->node_count;
    const cui_dot_display_opts options = { .pitch = pitch, .color = fpas_chart_color(palette, rgba),
        .accent = (flags & 1) != 0, .no_dim = (flags & 2) != 0,
        .align = align, .width = fpas_extent(width_kind, width) };
    cui_dot_display_ex(ui, text, &options);
    /* CUI-154 (docs/future/cui-upstream-bugs.md#cui-154): Use actual paint bounds
       for marquee wakeups, including the first frame and changes in width. */
    if ((flags & 4) && ui->theme.motion > 0) {
        cui__dmf_paint *paint = ui->nodes[node].paint_data;
        paint->scroll = (float)(cui_time(ui) * 24.0 * paint->pitch);
        ui->nodes[node].paint = fpas_paint_dot_display;
    }
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id = cui_make_id(ui, id);
}
