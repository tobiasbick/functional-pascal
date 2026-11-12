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


static fpas_avatar_image *fpas_avatar_image_for(cui_ctx *ui, cui_id key, const char *path) {
    const cui_id image_key = key ^ 0x46504149u;
    fpas_own(ui, key, image_key, 0, 0);
    fpas_avatar_image *state = cui_state(ui, image_key, sizeof *state);
    cui__state_set_dtor(ui, image_key, fpas_avatar_image_free);
    if (path ? !state->path || strcmp(path, state->path) != 0 : state->path != NULL) {
        if (state->texture) cui_image_destroy(ui, state->texture);
        state->texture = path ? cui_image_load(ui, path, NULL, NULL) : 0;
        fpas_avatar_path(ui, state, path);
    }
    return state;
}
