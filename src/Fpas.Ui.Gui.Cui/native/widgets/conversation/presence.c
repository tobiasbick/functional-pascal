// [Documentation](../../../../../docs/ui/gui/social-and-media/presence.md)
#include "../../fpas_cui.h"

void fpas_cui_presence(cui_ctx *ui, const char *id, const fpas_presence_user *users,
    int count, int maximum, float size) {
    const int first = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    if (maximum <= 0) maximum = 5;
    const int visible = CUI_MIN(count, maximum);
    fpas_avatar_image *images = fpas_avatar_cache_resize(ui, key, visible);
    cui_presence_user *native = cui__frame_alloc(ui, sizeof *native * (size_t)CUI_MAX(count, 1));
    const bool hover_allowed = !cui_is_disabled(ui) && ui->active == 0 &&
        !ui->mdown[CUI_MOUSE_LEFT] && !ui->mpressed[CUI_MOUSE_LEFT];
    const cui_id hot = ui->hot;
    int hovered = -1;
    for (int i = 0; i < count; ++i) {
        const fpas_presence_user *user = &users[i];
        cui_texture avatar = 0;
        if (i < visible) {
            fpas_avatar_image_update(ui, &images[i], user->avatar);
            avatar = images[i].texture;
            if (hover_allowed && hot == cui__hash(&i, sizeof i, key)) hovered = i;
        }
        native[i] = (cui_presence_user){ user->name, fpas_chart_color(user->palette, user->rgba),
            avatar, user->idle != 0 };
    }
    /* CUI-070: a visible avatar owns the hit, but the native spread tests the group.
       See docs/future/cui-upstream-bugs.md#cui-070. */
    const cui_id tooltip = ui->tooltip_id;
    const double hover_start = ui->hover_start;
    if (hovered >= 0) ui->hot = key;
    cui_presence(ui, id, native, count, maximum, size);
    if (hovered >= 0) {
        ui->hot = hot;
        ui->tooltip_id = tooltip;
        ui->hover_start = hover_start;
        ui->last_id = hot;
        const fpas_presence_user *user = &users[hovered];
        cui_tooltip(ui, user->idle ? cui__frame_printf(ui, "%s (away)", user->name) : user->name);
    }
    fpas_creative_node(ui, first, key);
}
