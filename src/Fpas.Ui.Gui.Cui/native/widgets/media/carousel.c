#include "../../fpas_cui.h"

int fpas_cui_carousel_begin(cui_ctx *ui, const char *id, int count, int index,
    float height, float gap, float advance, int flags, int focus) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), view = key ^ 0x71E3u;
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    fpas_own(ui, key, view, 0, 0);
    if (enabled && focus) ui->focus_next_request = view;
    if (!enabled) {
        cui__car_state *state = cui__state_lazy(ui, key, sizeof *state, false);
        if (state) { state->dragging = false; state->last_change = ui->time; }
    }
    const cui_carousel_opts options = { .height = height, .gap = gap,
        .auto_advance = enabled ? advance : 0, .no_arrows = (flags & 2) != 0,
        .no_dots = (flags & 4) != 0, .loop = (flags & 8) != 0 };
    int *selected = cui__frame_alloc(ui, sizeof *selected);
    *selected = index;
    cui_push_disabled(ui, !enabled);
    cui_carousel_begin(ui, id, count, selected, &options);
    cui_pop_disabled(ui);
    return root;
}

void fpas_cui_carousel_slide_begin(cui_ctx *ui) { cui_carousel_slide_begin(ui); }
void fpas_cui_carousel_slide_end(cui_ctx *ui) { cui_carousel_slide_end(ui); }

int fpas_cui_carousel_end(cui_ctx *ui, int root, int enabled, int *parts) {
    const struct cui_carousel_build *carousel = cui__car_top(ui);
    const cui_id key = carousel->id;
    int *selected = carousel->index;
    const int count = carousel->count;
    const bool dots = !carousel->o.no_dots && count > 1;
    cui_push_disabled(ui, !enabled);
    cui_carousel_end(ui);
    cui_pop_disabled(ui);
    parts[0] = fpas_find(ui, key ^ 0x71E3u);
    for (int index = 0; index < count; ++index)
        parts[index + 1] = dots ? fpas_find(ui, fpas_well_part(key ^ 0xD07u, index)) : -1;
    fpas_creative_node(ui, root, key);
    return *selected;
}
