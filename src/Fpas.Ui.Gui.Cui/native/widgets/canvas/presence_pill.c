// [Documentation](../../../../../docs/ui/gui/canvas-and-bars/presence-pill.md)
#include "../../fpas_cui.h"

/** Uses the topmost visible avatar under the pointer for the native tooltip. */
static void fpas_presence_pill_tooltip(cui_ctx *ui, cui_id key, const cui_live_user *people, int shown) {
    const bool can_hover = ui->mouse_inside && !ui->mdown[CUI_MOUSE_LEFT] && !ui->active &&
        ui->hot_layer == ui->current_layer;
    for (int i = shown - 1; i >= 0; --i) {
        const cui_id avatar = cui__hash(&i, sizeof i, key);
        const char *name = people[i].name;
        if (can_hover && name && *name && cui__rect_contains(cui__prev_rect(ui, avatar), ui->mx, ui->my)) {
            const char *suffix = people[i].away ? " (away)" : "";
            const int length = cui__utf8_cut(name, (int)strlen(name), 255 - (int)strlen(suffix));
            cui__tooltip_passive(ui, avatar, cui__frame_printf(ui, "%.*s%s", length, name, suffix));
            return;
        }
        if (ui->tooltip_id == avatar) ui->tooltip_id = 0;
    }
}

void fpas_cui_presence_pill(cui_ctx *ui, const char *id, const cui_live_user *people,
    const char *const *avatars, int count, float size, int max_visible) {
    const cui_id key = cui_make_id(ui, id);
    const int shown = CUI_MIN(count, max_visible > 0 ? max_visible : 5);
    const int visible = avatars ? shown : 0;
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
    /* CUI-160 (docs/future/cui-upstream-bugs.md#cui-160): suppress unfiltered tooltip requests. */
    const cui_id hot_layer = ui->hot_layer;
    ui->hot_layer = ui->current_layer ^ 1u;
    cui_presence_pill_ex(ui, id, people, count, &options);
    ui->hot_layer = hot_layer;
    fpas_presence_pill_tooltip(ui, key, people, shown);
    fpas_creative_node(ui, root, key);
}
