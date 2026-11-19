// [Documentation](../../../../../docs/ui/gui/disclosure/avatar.md)
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
static void fpas_avatar_image_update(cui_ctx *ui, fpas_avatar_image *state, const char *path) {
    if (path ? !state->path || strcmp(path, state->path) != 0 : state->path != NULL) {
        if (state->texture) cui_image_destroy(ui, state->texture);
        state->texture = path ? cui_image_load(ui, path, NULL, NULL) : 0;
        fpas_avatar_path(ui, state, path);
    }
}

static fpas_avatar_image *fpas_avatar_image_at(cui_ctx *ui, cui_id image_key, const char *path) {
    fpas_avatar_image *state = cui_state(ui, image_key, sizeof *state);
    cui__state_set_dtor(ui, image_key, fpas_avatar_image_free);
    fpas_avatar_image_update(ui, state, path);
    return state;
}

static fpas_avatar_image *fpas_avatar_image_for(cui_ctx *ui, cui_id key, const char *path) {
    const cui_id image_key = key ^ 0x46504149u;
    fpas_own(ui, key, image_key, 0, 0);
    return fpas_avatar_image_at(ui, image_key, path);
}

typedef struct {
    fpas_avatar_image *images;
    int count, capacity;
} fpas_avatar_cache;

static void fpas_avatar_cache_free(cui_ctx *ui, void *data) {
    fpas_avatar_cache *cache = data;
    for (int i = 0; i < cache->count; ++i) fpas_avatar_image_free(ui, &cache->images[i]);
    cui__free(ui, cache->images);
}

static fpas_avatar_image *fpas_avatar_cache_resize(cui_ctx *ui, cui_id key, int visible) {
    visible = CUI_MAX(visible, 0);
    const cui_id image_key = key ^ 0x50494C4Cu;
    fpas_own(ui, key, image_key, 0, 0);
    fpas_avatar_cache *cache = cui_state(ui, image_key, sizeof *cache);
    cui__state_set_dtor(ui, image_key, fpas_avatar_cache_free);
    if (visible > cache->capacity) {
        cache->images = cui__realloc(ui, cache->images, (size_t)visible * sizeof *cache->images);
        cache->capacity = visible;
    }
    for (int i = cache->count; i < visible; ++i) cache->images[i] = (fpas_avatar_image){0};
    for (int i = visible; i < cache->count; ++i) fpas_avatar_image_free(ui, &cache->images[i]);
    cache->count = visible;
    return cache->images;
}
