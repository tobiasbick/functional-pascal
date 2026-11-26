// [Documentation](../../../../docs/ui/gui/cui.md#layout-and-control-inspection)
#include "../fpas_cui.h"

static bool fpas_box_has_space(cui_ctx *ui) {
    return ui->parent_depth < CUI_PARENT_STACK;
}

static int fpas_open_box(cui_ctx *ui, const cui_box *options) {
    if (!fpas_box_has_space(ui)) return -1;
    cui_box_begin(ui, options);
    return cui__current(ui);
}

/** Opens a full-width column and returns its frame-local node. */
int fpas_cui_column_begin(cui_ctx *ui, float padding, float gap) {
    const cui_box options = { .dir = CUI_COLUMN, .pad = padding, .gap = gap, .width = CUI_FILL };
    return fpas_open_box(ui, &options);
}

/** Opens a full-width row and returns its frame-local node. */
int fpas_cui_row_begin(cui_ctx *ui, float padding, float gap) {
    const cui_box options = { .dir = CUI_ROW, .pad = padding, .gap = gap, .width = CUI_FILL };
    return fpas_open_box(ui, &options);
}

void fpas_cui_request_focus(cui_ctx *ui, const char *id) {
    ui->focus_next_request = cui_make_id(ui, id);
}

void fpas_cui_move_pointer(cui_ctx *ui, float x, float y) {
    cui_input_mouse_move(ui, x, y);
    /* The normal frame-start hit test ran before this injected move. */
    cui__compute_hot(ui);
}

void fpas_cui_input_scroll(cui_ctx *ui, float x, float y) {
    cui_input_scroll(ui, x, y);
    /* Wheel routing depends on the direction of the newly injected input. */
    cui__compute_hot(ui);
}

/** Opens a scroll area and returns its frame-local node. */
int fpas_cui_scroll_begin(cui_ctx *ui, const char *id, float height, int horizontal) {
    if (!fpas_box_has_space(ui)) return -1;
    const cui_scroll_opts options = {
        .id = id, .width = CUI_FILL, .height = height > 0 ? CUI_PX(height) : CUI_FILL,
        .horizontal = horizontal != 0
    };
    cui_scroll_begin(ui, &options);
    return cui__current(ui);
}
