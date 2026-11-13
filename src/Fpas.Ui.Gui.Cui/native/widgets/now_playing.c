#include "../fpas_cui.h"

static void fpas_player_forget(cui_ctx *ui, cui_id key) {
    fpas_forget_state(ui, (key ^ 0x91A7u) ^ 1u);
}

int fpas_cui_now_playing(cui_ctx *ui, const char *id, const char *title, const char *artist,
    const char *artwork, int palette, uint32_t rgba, float *position, float duration,
    int *playing, int width_kind, float width, int flags, int focus, int *parts) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const fpas_avatar_image *image = fpas_avatar_image_for(ui, key, artwork);
    cui_now_playing value = { title, artist, image->texture, fpas_chart_color(palette, rgba),
        *position, duration, *playing != 0 };
    const cui_now_playing_opts options = { .width = fpas_extent(width_kind, width), .compact = (flags & 1) != 0 };
    const bool enabled = (flags & 2) && !cui_is_disabled(ui);
    cui_push_id(ui, id);
    const cui_id keys[] = { key ^ 0x91A7u, cui__hash_str(CUI_ICON_SKIP_BACK, cui_make_id(ui, "Previous")),
        cui__hash_str(CUI_ICON_SKIP_FORWARD, cui_make_id(ui, "Next")), key ^ 0x5C8Bu };
    cui_pop_id(ui);
    if (enabled && focus >= 0 && focus < 3) ui->focus_next_request = keys[focus];
    if (!enabled) {
        cui__np_state *state = cui__state_lazy(ui, key, sizeof *state, false);
        if (state) state->dragging = false;
    }
    cui_push_disabled(ui, !enabled);
    const int changed = cui_now_playing_ex(ui, id, &value, &options);
    cui_pop_disabled(ui);
    *position = value.position;
    *playing = value.playing;
    for (int i = 0; i < 4; ++i) parts[i] = fpas_find(ui, keys[i]);
    fpas_on_forget(ui, key, fpas_player_forget);
    fpas_motion_node(ui, node, key);
    return enabled ? changed : 0;
}
