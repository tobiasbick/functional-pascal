// [Documentation](../../../../../docs/ui/gui/more-charts/candlestick.md)
#include "../../fpas_cui.h"

int fpas_cui_candlestick(cui_ctx *ui, const char *id, const cui_candle *candles, int count,
    const fpas_candle_options *options, int enabled, int focus, int *node) {
    const cui_id key = cui_make_id(ui, id);
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (!allowed && ui->focus == key) ui->focus = 0;
    if (allowed && focus) ui->focus_next_request = key;
    fpas_own(ui, key, key ^ 0xCA4D2u, 0, 0);
    const cui_candlestick_opts o = { .width = fpas_extent(options->width_kind, options->width), .height = options->height,
        .labels = options->labels, .format = options->format,
        .overlays = fpas_chart_series_of(ui, options->overlays, options->overlay_count),
        .overlay_count = options->overlay_count, .no_volume = (options->flags & 1) != 0, .ohlc = (options->flags & 2) != 0,
        .up = fpas_chart_color(options->up_palette, options->up_rgba), .down = fpas_chart_color(options->down_palette, options->down_rgba) };
    *node = ui->node_count;
    cui_push_disabled(ui, !allowed);
    const int cursor = cui_candlestick_ex(ui, id, candles, count, &o);
    cui_pop_disabled(ui); ui->last_node = *node;
    return cursor;
}
