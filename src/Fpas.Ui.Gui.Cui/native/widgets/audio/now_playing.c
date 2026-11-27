// [Documentation](../../../../../docs/ui/gui/motion/now-playing.md)
#include "../../fpas_cui.h"

int fpas_cui_now_playing(cui_ctx *ui, const char *id, const char *title, const char *artist,
    const fpas_media_source *artwork, int palette, uint32_t rgba, float *position, float duration,
    int *playing, int width_kind, float width, int flags, int focus, int *parts) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_id texture_key = artwork ? key ^ 0x17B41u : 0;
    fpas_own(ui, key, texture_key, (key ^ 0x91A7u) ^ 1u, 0);
    const cui_texture texture = artwork ? fpas_media_resolve(ui, texture_key, artwork)->texture : 0;
    cui_now_playing value = { title, artist, texture, fpas_chart_color(palette, rgba),
        *position, duration, *playing != 0 };
    const cui_now_playing_opts options = { .width = fpas_extent(width_kind, width), .compact = (flags & 1) != 0 };
    const bool enabled = (flags & 2) && !cui_is_disabled(ui);
    cui_push_id(ui, id);
    const cui_id keys[] = { key ^ 0x91A7u, cui__hash_str(CUI_ICON_SKIP_BACK, cui_make_id(ui, "Previous")),
        cui__hash_str(CUI_ICON_SKIP_FORWARD, cui_make_id(ui, "Next")), key ^ 0x5C8Bu };
    cui_pop_id(ui);
    if (enabled && focus >= 0 && focus < 3) ui->focus_next_request = keys[focus];
    cui__np_state *state = cui_state(ui, key, sizeof *state);
    if (!enabled || !ui->window_focused) {
        state->dragging = false;
    } else {
        float fraction;
        if (fpas_drag_release(ui, keys[3], state->dragging, &fraction)) {
            state->drag_pos = fraction * duration;
            state->dragging = true;
        }
    }
    cui_push_disabled(ui, !enabled);
    const int changed = cui_now_playing_ex(ui, id, &value, &options);
    cui_pop_disabled(ui);
    *position = value.position;
    *playing = value.playing;
    for (int i = 0; i < 4; ++i) parts[i] = fpas_find(ui, keys[i]);
    fpas_motion_node(ui, node, key);
    return enabled ? changed : 0;
}
