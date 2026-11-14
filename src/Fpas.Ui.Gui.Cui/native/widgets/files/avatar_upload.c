#include "../../fpas_cui.h"

int fpas_cui_avatar_upload(cui_ctx *ui, const char *id, const char *name, const char *path, float size,
    int enabled, int focus, int *node, int *has_image) {
    const cui_id key = cui_make_id(ui, id);
    fpas_avatar_image *state = fpas_avatar_image_for(ui, key, path);

    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) ui->focus_next_request = key;
    cui_push_disabled(ui, !enabled);
    const int event = cui_avatar_upload(ui, id, cui__frame_strdup(ui, name, -1), &state->texture, size);
    cui_pop_disabled(ui);
    if (event == 1) fpas_avatar_path(ui, state, cui_dropped_file(ui, 0));
    *node = ui->last_node = fpas_find(ui, key);
    *has_image = state->texture != 0;
    return allowed ? event : 0;
}
