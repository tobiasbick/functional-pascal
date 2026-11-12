#include "../fpas_cui.h"

int fpas_cui_card_begin(cui_ctx *ui, const char *id, int width_kind, float width,
    int height_kind, float height, float minimum, float maximum, float pad, float pad_x,
    float pad_y, float gap, int row, int align, int wrap, int muted, int selected,
    int hoverable, int enabled, int focus, int *node) {
    const cui_card_opts options = {
        .id = id, .width = fpas_extent(width_kind, width), .height = fpas_extent(height_kind, height),
        .min_width = minimum, .max_width = maximum, .pad = pad, .pad_x = pad_x, .pad_y = pad_y,
        .gap = gap, .dir = row ? CUI_ROW : CUI_COLUMN, .align = (cui_align)align, .wrap = wrap != 0,
        .muted = muted != 0, .selected = selected != 0, .hoverable = hoverable != 0
    };
    if (hoverable && enabled && focus && !cui_is_disabled(ui)) ui->focus_next_request = cui_make_id(ui, id);
    cui_push_disabled(ui, !enabled);
    const bool clicked = cui_card_begin(ui, &options);
    cui_pop_disabled(ui);
    *node = cui__current(ui);
    return clicked;
}

void fpas_cui_card_end(cui_ctx *ui, int node) {
    cui_card_end(ui);
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id;
}
