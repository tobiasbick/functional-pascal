#include "../fpas_cui.h"

typedef struct { cui_texture texture; char *path; } fpas_avatar_image;

static void fpas_avatar_image_free(cui_ctx *ui, void *data) {
    fpas_avatar_image *state = data;
    if (state->texture) cui_image_destroy(ui, state->texture);
    cui__free(ui, state->path);
}

static void fpas_avatar_path(cui_ctx *ui, fpas_avatar_image *state, const char *path) {
    cui__free(ui, state->path);
    state->path = NULL;
    if (path) {
        const size_t size = strlen(path) + 1;
        state->path = cui__alloc(ui, size);
        memcpy(state->path, path, size);
    }
}

int fpas_cui_avatar_upload(cui_ctx *ui, const char *id, const char *name, const char *path, float size,
    int enabled, int focus, int *node, int *has_image) {
    const cui_id key = cui_make_id(ui, id), image_key = key ^ 0x46504149u;
    fpas_own(ui, key, image_key, 0, 0);
    fpas_avatar_image *state = cui_state(ui, image_key, sizeof *state);
    cui__state_set_dtor(ui, image_key, fpas_avatar_image_free);
    if (path ? !state->path || strcmp(path, state->path) != 0 : state->path != NULL) {
        if (state->texture) cui_image_destroy(ui, state->texture);
        state->texture = path ? cui_image_load(ui, path, NULL, NULL) : 0;
        fpas_avatar_path(ui, state, path);
    }

    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) ui->focus_next_request = key;
    /* CUI-015 (docs/future/cui-upstream-bugs.md#cui-015): native drop
       claiming bypasses disabled scopes. Preserve the event for other targets. */
    const bool claimed = ui->drop_claimed, drag = ui->drag_hover;
    if (!allowed) { ui->drop_claimed = true; ui->drag_hover = false; }
    cui_push_disabled(ui, !enabled);
    const int event = cui_avatar_upload(ui, id, cui__frame_strdup(ui, name, -1), &state->texture, size);
    cui_pop_disabled(ui);
    if (!allowed) { ui->drop_claimed = claimed; ui->drag_hover = drag; }
    if (event == 1) fpas_avatar_path(ui, state, cui_dropped_file(ui, 0));
    *node = ui->last_node = fpas_find(ui, key);
    *has_image = state->texture != 0;
    return allowed ? event : 0;
}
