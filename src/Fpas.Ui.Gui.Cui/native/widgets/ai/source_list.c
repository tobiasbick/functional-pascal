// [Documentation](../../../../../docs/ui/gui/ai/source-list.md)
#include "../../fpas_cui.h"

typedef struct { cui_id card; float width; } fpas_source_focus;
typedef struct { fpas_source_focus *state; int card; bool requested; } fpas_source_reveal;

/* [CUI-112](../../../../../docs/future/cui-upstream-bugs.md#cui-112):
 * Focus order includes every source card, even when the viewport clips it. */
static void reveal_focused_source(cui_ctx *ui, cui_node *viewport, void *data) {
    const fpas_source_reveal *reveal = data;
    fpas_source_focus *state = reveal->state;
    const cui_node *card = &ui->nodes[reveal->card];
    const cui_rect view = viewport->rect, bounds = card->rect;
    const bool changed = reveal->requested || state->width != view.w ||
        (ui->focus_visible && state->card != card->id);
    state->card = card->id;
    state->width = view.w;
    if (!changed) return;

    const float left = bounds.x - view.x;
    const float leading = viewport->rtl ? view.w - left - bounds.w : left;
    const float shift = leading < 0 || bounds.w > view.w ? leading :
        CUI_MAX(0, leading + bounds.w - view.w);
    if (fabsf(shift) < 0.01f) return;
    cui_scroll_state *scroll = cui_state(ui, viewport->id, sizeof *scroll);
    const float target = cui__clampf(scroll->x + shift, 0, CUI_MAX(0, scroll->content_w - view.w));
    if (target != scroll->tx) {
        scroll->tx = target;
        cui_request_redraw(ui);
    }
}

/* [CUI-024](../../../../../docs/future/cui-upstream-bugs.md#cui-024):
 * Keep the URL for the card, but end the native tooltip at a character boundary. */
static void source_tooltip_boundary(cui_ctx *ui, cui_id card, const char *url) {
    if (!url || ui->tooltip_id != card) return;
    cui__tooltip_state *tip = cui__tooltip_get(ui);
    const int capacity = sizeof tip->text - 1;
    if (tip->anchor == card && tip->visible && strlen(tip->text) == (size_t)capacity && url[capacity])
        tip->text[cui__utf8_cut(url, capacity + 1, capacity)] = '\0';
}

/* [CUI-114](../../../../../docs/future/cui-upstream-bugs.md#cui-114):
 * CUI copies only the first byte of the domain into its initial glyph. */
static void source_favicon_letter(cui_ctx *ui, int row) {
    const int favicon = ui->nodes[row].first;
    const int letter = ui->nodes[favicon].first;
    const cui_node *domain = &ui->nodes[ui->nodes[favicon].next];
    const int length = cui__utf8_next(domain->text, domain->text_len, 0);
    if (length > 1) {
        ui->nodes[letter].text = cui__frame_strdup(ui, domain->text, length);
        ui->nodes[letter].text_len = length;
    }
}

int fpas_cui_source_list(cui_ctx *ui, const char *id, const cui_source *sources, int count, int flags, int focus, int *parts) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (enabled && focus >= 0 && focus < count) ui->focus_next_request = fpas_well_part(key ^ 0x50Cu, focus);
    cui_push_disabled(ui, !enabled);
    const int picked = cui_source_list(ui, id, sources, count);
    cui_pop_disabled(ui);
    int focused = -1;
    for (int i = 0; i < count; ++i) {
        const cui_id part = fpas_well_part(key ^ 0x50Cu, i);
        parts[i] = fpas_find(ui, part);
        if (parts[i] >= 0) source_favicon_letter(ui, ui->nodes[parts[i]].first);
        source_tooltip_boundary(ui, part, sources[i].url);
        if (enabled && ui->focus == part) focused = parts[i];
    }
    fpas_source_focus *state = cui__state_lazy(ui, key ^ 0xF0C5u, sizeof *state, focused >= 0);
    if (focused >= 0) {
        fpas_source_reveal *reveal = cui__frame_alloc(ui, sizeof *reveal);
        *reveal = (fpas_source_reveal){ state, focused, focus >= 0 && focus < count };
        ui->nodes[root].paint = reveal_focused_source;
        ui->nodes[root].paint_data = reveal;
    } else if (state) {
        state->card = 0;
    }
    fpas_creative_node(ui, root, key);
    return enabled ? picked : -1;
}
