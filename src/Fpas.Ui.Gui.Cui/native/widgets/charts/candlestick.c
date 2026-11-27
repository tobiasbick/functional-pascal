// [Documentation](../../../../../docs/ui/gui/more-charts/candlestick.md)
#include "../../fpas_cui.h"

/** Keeps the retained viewport and inspection index valid for the current series.
 * CUI-088 (docs/future/cui-upstream-bugs.md#cui-088). */
static void fpas_candlestick_resize(cui_ctx *ui, cui_id key, int count) {
    cui__cs_state *state = cui__state_lazy(ui, key, sizeof *state, false);
    if (!state || state->count == count) return;
    if (count <= 0) {
        state->keys = false;
        state->cursor = -1;
    } else if (state->count == 0) {
        state->init = false;
    } else if (count < state->count) {
        if (state->cursor >= count) state->cursor = count - 1;
        if (state->span > CUI_MAX(8.0, count * 1.1))
            state->span = CUI_MAX(1.0, (double)count);
    }
}

/** Paints the chart with its native data and then draws CUI's focus ring. */
static void fpas_paint_focused_candlestick(cui_ctx *ui, cui_node *node, void *data) {
    cui__paint_candles(ui, node, data);
    cui__draw_focus_ring(ui, node->rect, node->radius, 1.0f);
}

/** Draws CUI's retained candlestick and reports the current input index. */
int fpas_cui_candlestick(cui_ctx *ui, const char *id, const cui_candle *candles, int count,
    const fpas_candle_options *options, int enabled, int focus, int *node) {
    const cui_id key = cui_make_id(ui, id);
    const cui_id viewport = key ^ 0xCA4D2u;
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (!allowed && ui->focus == key) ui->focus = 0;
    if (allowed && focus) ui->focus_next_request = key;
    fpas_own(ui, key, viewport, 0, 0);
    fpas_candlestick_resize(ui, viewport, count);
    const cui_candlestick_opts o = { .width = fpas_extent(options->width_kind, options->width), .height = options->height,
        .labels = options->labels, .format = options->format,
        .overlays = fpas_chart_series_of(ui, options->overlays, options->overlay_count),
        .overlay_count = options->overlay_count, .no_volume = (options->flags & 1) != 0, .ohlc = (options->flags & 2) != 0,
        .up = fpas_chart_color(options->up_palette, options->up_rgba), .down = fpas_chart_color(options->down_palette, options->down_rgba) };
    *node = ui->node_count;
    cui_push_disabled(ui, !allowed);
    int cursor;
    if (allowed && fpas_drag_focus_ring(ui, key)) {
        /* CUI-095 (docs/future/cui-upstream-bugs.md#cui-095):
           Keep the single chart paint slot. A temporary cover prevents
           cui__interact from installing the ring while the chart builds. */
        const fpas_focus_cover cover = fpas_begin_focus_cover(ui);
        cursor = cui_candlestick_ex(ui, id, candles, count, &o);
        fpas_end_focus_cover(ui, cover);
        ui->nodes[*node].paint = fpas_paint_focused_candlestick;
    } else {
        cursor = cui_candlestick_ex(ui, id, candles, count, &o);
    }
    cui_pop_disabled(ui); ui->last_node = *node;
    return cursor;
}
