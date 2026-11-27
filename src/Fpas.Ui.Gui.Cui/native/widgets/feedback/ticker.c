// [Documentation](../../../../../docs/ui/gui/social-and-media/ticker.md)
#include "../../fpas_cui.h"

/** Keeps the native edge fade and refreshes when a first layout needs additional copies. */
static void fpas_ticker_edges(cui_ctx *ui, cui_node *node, void *data) {
    cui__paint_ticker_edges(ui, node, data);
    const cui_node *strip = &ui->nodes[node->abs_first];
    const float set = ui->nodes[strip->first].rect.w;
    const float visible = CUI_MIN(node->rect.w, ui->vw);
    if (set > 0 && set * strip->child_count + strip->tx + 0.5f < visible)
        cui_request_redraw(ui);
}

/** Builds and repeats CUI's native ticker entries to fill the visible strip. */
void fpas_cui_ticker(cui_ctx *ui, const char *id, const fpas_ticker_item *items, int count,
    float speed, float height) {
    const int first = ui->node_count;
    cui_ticker_item *native = cui__frame_alloc(ui, sizeof *native * (size_t)CUI_MAX(count, 1));
    for (int i = 0; i < count; ++i) {
        const fpas_ticker_item *item = &items[i];
        native[i] = (cui_ticker_item){ item->label, item->value, item->delta, item->lower_is_better != 0 };
    }
    const cui_ticker_opts options = { .speed = speed, .height = height };
    cui_ticker_ex(ui, id, native, count, &options);
    const cui_id key = cui_make_id(ui, id);
    /* CUI-073 (docs/future/cui-upstream-bugs.md#cui-073): three copies do
       not fill a wide ticker with a short set. Keep native items and fade. */
    if (count > 0) {
        ui->nodes[first].paint = fpas_ticker_edges;
        const int strip = ui->nodes[first].abs_first;
        cui__ticker_state *state = cui_state(ui, key, sizeof *state);
        if (state->set_w > 0) {
            if (state->off >= state->set_w) state->off = fmodf(state->off, state->set_w);
            ui->nodes[strip].tx = -state->off;
            const float previous = cui__prev_rect(ui, key).w;
            const float width = previous > 0 ? CUI_MIN(previous, ui->vw) : ui->vw;
            const int copies = (int)ceilf((width + state->off) / state->set_w);
            if (copies > 3) {
                ui->nodes[strip].w = CUI_PX(state->set_w * copies + 16);
                cui__open(ui, strip);
                for (int copy = 3; copy < copies; ++copy) cui__ticker_set(ui, 0, native, count);
                cui__close(ui);
            }
        }
    }
    fpas_creative_node(ui, first, key);
}
