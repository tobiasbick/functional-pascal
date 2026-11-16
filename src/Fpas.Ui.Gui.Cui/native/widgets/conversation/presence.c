#include "../../fpas_cui.h"

void fpas_cui_presence(cui_ctx *ui, const char *id, const fpas_presence_user *users,
    int count, int maximum, float size) {
    const int first = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const int visible = CUI_MIN(count, maximum);
    fpas_avatar_image *images = fpas_avatar_cache_resize(ui, key, visible);
    cui_presence_user *native = cui__frame_alloc(ui, sizeof *native * (size_t)CUI_MAX(count, 1));
    for (int i = 0; i < count; ++i) {
        const fpas_presence_user *user = &users[i];
        cui_texture avatar = 0;
        if (i < visible) {
            fpas_avatar_image_update(ui, &images[i], user->avatar);
            avatar = images[i].texture;
        }
        native[i] = (cui_presence_user){ user->name, fpas_chart_color(user->palette, user->rgba),
            avatar, user->idle != 0 };
    }
    cui_presence(ui, id, native, count, maximum, size);
    fpas_creative_node(ui, first, key);
}
