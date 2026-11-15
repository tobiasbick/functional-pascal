#include "../../fpas_cui.h"

int fpas_cui_wheel_picker(cui_ctx *ui, const char *id, const char *const *items, int count,
    int index, float width, int rows, int loop, int join_left, int join_right, int enabled,
    int focus, int *node) {
    const cui_id key = cui_make_id(ui, id);
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) ui->focus_next_request = key;
    if (!allowed) {
        cui__wheel_state *state = cui__state_lazy(ui, key, sizeof *state, false);
        if (state) state->dragging = false;
    }

    /* CUI reads the captions during painting, after the managed call returns. */
    const char **captions = cui__frame_alloc(ui, (size_t)count * sizeof *captions);
    for (int i = 0; i < count; ++i) captions[i] = cui__frame_strdup(ui, items[i], -1);

    const cui_wheel_opts options = { .width = width > 0 ? CUI_PX(width) : CUI_FIT, .rows = rows,
        .loop = loop != 0, .join_left = join_left != 0, .join_right = join_right != 0 };
    int selected = index;
    cui_push_disabled(ui, !enabled);
    cui_wheel_picker_ex(ui, id, captions, count, &selected, &options);
    cui_pop_disabled(ui);
    *node = fpas_find(ui, key);
    ui->last_node = *node;
    fpas_own(ui, key, key ^ CUI__VERTICAL_WHEEL_STATE, 0, 0);
    return allowed ? selected : index;
}
