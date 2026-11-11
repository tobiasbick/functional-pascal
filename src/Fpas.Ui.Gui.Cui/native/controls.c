#include "fpas_cui.h"

void fpas_cui_column_begin(cui_ctx *ui, float padding, float gap) {
    const cui_box options = { .pad = padding, .gap = gap, .width = CUI_FILL };
    (void)cui_column_begin(ui, &options);
}

void fpas_cui_row_begin(cui_ctx *ui, float padding, float gap) {
    const cui_box options = { .pad = padding, .gap = gap, .width = CUI_FILL };
    (void)cui_row_begin(ui, &options);
}

void fpas_cui_request_focus(cui_ctx *ui, const char *id) {
    ui->focus_next_request = cui_make_id(ui, id);
}

void fpas_cui_move_pointer(cui_ctx *ui, float x, float y) {
    cui_input_mouse_move(ui, x, y);
    /* The normal frame-start hit test ran before this injected move. */
    cui__compute_hot(ui);
}

void fpas_cui_scroll_begin(cui_ctx *ui, const char *id, float height, int horizontal) {
    const cui_scroll_opts options = {
        .id = id, .width = CUI_FILL, .height = height > 0 ? CUI_PX(height) : CUI_FILL,
        .horizontal = horizontal != 0
    };
    cui_scroll_begin(ui, &options);
}
