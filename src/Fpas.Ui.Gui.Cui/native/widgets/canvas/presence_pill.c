#include "../../fpas_cui.h"

typedef struct {
    fpas_avatar_image *images;
    int count, capacity;
} fpas_pill_images;

static void fpas_pill_images_free(cui_ctx *ui, void *data) {
    fpas_pill_images *cache = data;
    for (int i = 0; i < cache->count; ++i) fpas_avatar_image_free(ui, &cache->images[i]);
    cui__free(ui, cache->images);
}

void fpas_cui_presence_pill(cui_ctx *ui, const char *id, const cui_live_user *people,
    const char *const *avatars, int count, float size, int max_visible) {
    const cui_id key = cui_make_id(ui, id);
    const cui_id image_key = key ^ 0x50494C4Cu;
    fpas_own(ui, key, image_key, 0, 0);
    fpas_pill_images *cache = cui_state(ui, image_key, sizeof *cache);
    cui__state_set_dtor(ui, image_key, fpas_pill_images_free);
    const int visible = avatars ? CUI_MIN(count, max_visible > 0 ? max_visible : 5) : 0;
    if (visible > cache->capacity) {
        cache->images = cui__realloc(ui, cache->images, (size_t)visible * sizeof *cache->images);
        cache->capacity = visible;
    }
    for (int i = cache->count; i < visible; ++i) cache->images[i] = (fpas_avatar_image){0};
    for (int i = visible; i < cache->count; ++i) fpas_avatar_image_free(ui, &cache->images[i]);
    cache->count = visible;

    if (visible > 0) {
        cui_live_user *native = cui__frame_alloc(ui, (size_t)count * sizeof *native);
        memcpy(native, people, (size_t)count * sizeof *native);
        for (int i = 0; i < visible; ++i) {
            const char *path = avatars[i];
            fpas_avatar_image_update(ui, &cache->images[i], path && *path ? path : NULL);
            native[i].avatar = cache->images[i].texture;
        }
        people = native;
    }

    const int root = ui->node_count;
    const cui_presence_pill_opts options = { .size = size, .max_visible = max_visible };
    cui_presence_pill_ex(ui, id, people, count, &options);
    fpas_creative_node(ui, root, key);
}
