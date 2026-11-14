#include "../../fpas_cui.h"

void fpas_cui_presence(cui_ctx *ui, const char *id, const fpas_presence_user *users,
    int count, int maximum, float size) {
    const int first = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    cui_presence_user *native = cui__frame_alloc(ui, sizeof *native * (size_t)CUI_MAX(count, 1));
    for (int i = 0; i < count; ++i) {
        const fpas_presence_user *user = &users[i];
        const cui_id uid = cui__hash(&i, sizeof i, key);
        const cui_texture avatar = i < maximum
            ? fpas_avatar_image_for(ui, uid, user->avatar)->texture : 0;
        native[i] = (cui_presence_user){ user->name, fpas_chart_color(user->palette, user->rgba),
            avatar, user->idle != 0 };
    }
    cui_presence(ui, id, native, count, maximum, size);
    fpas_creative_node(ui, first, key);
}
