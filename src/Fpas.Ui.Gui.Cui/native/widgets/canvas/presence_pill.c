#include "../../fpas_cui.h"

void fpas_cui_presence_pill(cui_ctx *ui, const char *id, const cui_live_user *people,
    const char *const *avatars, int count, float size, int max_visible) {
    const cui_id key = cui_make_id(ui, id);
    const int visible = avatars ? CUI_MIN(count, max_visible > 0 ? max_visible : 5) : 0;
    fpas_avatar_image *images = fpas_avatar_cache_resize(ui, key, visible);

    if (visible > 0) {
        cui_live_user *native = cui__frame_alloc(ui, (size_t)count * sizeof *native);
        memcpy(native, people, (size_t)count * sizeof *native);
        for (int i = 0; i < visible; ++i) {
            const char *path = avatars[i];
            fpas_avatar_image_update(ui, &images[i], path && *path ? path : NULL);
            native[i].avatar = images[i].texture;
        }
        people = native;
    }

    const int root = ui->node_count;
    const cui_presence_pill_opts options = { .size = size, .max_visible = max_visible };
    cui_presence_pill_ex(ui, id, people, count, &options);
    fpas_creative_node(ui, root, key);
}
