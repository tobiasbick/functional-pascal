// [Documentation](../../../../../docs/ui/gui/capture/avatar-upload.md)
#include "../../fpas_cui.h"

int fpas_cui_avatar_upload(cui_ctx *ui, const char *id, const char *name, const char *path, float size,
    int enabled, int focus, int *node, int *has_image) {
    const cui_id key = cui_make_id(ui, id);
    fpas_avatar_image *state = fpas_avatar_image_for(ui, key, path);

    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) ui->focus_next_request = key;
    const cui_texture previous = state->texture;
    cui_push_disabled(ui, !enabled);
    const fpas_file_drag_state hidden = fpas_filter_file_drag(ui, key);
    const int event = cui_avatar_upload(ui, id, cui__frame_strdup(ui, name, -1), &state->texture, size);
    fpas_restore_file_drag(ui, hidden);
    cui_pop_disabled(ui);
    /* CUI-047 (docs/future/cui-upstream-bugs.md#cui-047):
       Preserve a successful drop when CUI reports a click from the same frame. */
    const bool dropped = event == 1 || state->texture != previous;
    if (dropped) fpas_avatar_path(ui, state, cui_dropped_file(ui, 0));
    *node = ui->last_node = fpas_find(ui, key);
    *has_image = state->texture != 0;
    return allowed ? (dropped ? 1 : event) : 0;
}
