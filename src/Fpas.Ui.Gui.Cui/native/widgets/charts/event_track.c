// [Documentation](../../../../../docs/ui/gui/instruments/event-track.md)
#include "../../fpas_cui.h"

/** Applies a queued move after CUI has processed a fresh press at its original position. */
static void fpas_event_track_release(cui_ctx *ui, cui_id key, cui__ev_state *state,
    cui__ev_paint *paint, double *times, int count, double step, cui_event_track_result *result) {
    const int from = state->drag;
    if (from >= 0 && from < count &&
        (state->moved || fabsf(ui->release_x - state->press_x) > 2)) {
        const double time = cui__ev_time(paint->lo, paint->hi, step, cui__prev_rect(ui, key), ui->release_x);
        if (time != times[from]) {
            times[from] = time;
            const int to = cui__ev_settle(times, count, from);
            memcpy(paint->t, times, sizeof *times * (size_t)count);
            result->selected = paint->sel = to;
            if (result->added == from) result->added = to;
            result->changed = true;
        }
    }
}

void fpas_cui_event_track(cui_ctx *ui, const char *id, double *times, int *count,
    int capacity, const fpas_event_options *o, int *result) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    cui__ev_state *state = cui__state_lazy(ui, key, sizeof *state, false);
    const bool initialized = state && state->own_init;
    const bool dragging = initialized && state->drag >= 0 && state->moved;
    const bool native_selection = (o->flags & 32) != 0;
    int selected = result[0];
    const int prior_selection = native_selection ? (initialized ? state->sel : -1) : selected;
    const bool enabled = (o->flags & 8) != 0 && !cui_is_disabled(ui);
    if (enabled && (o->flags & 16)) ui->focus_next_request = key;
    if (!enabled && state) state->drag = -1;
    const cui_event_track_opts options = { .width = fpas_extent(o->width_kind, o->width),
        .step = o->step, .label_step = o->label_step, .units = o->units, .format = o->format,
        .label = o->label, .label_user = o->label_user,
        .selected = native_selection ? NULL : &selected,
        .no_add = (o->flags & 1) != 0, .no_remove = (o->flags & 2) != 0,
        .no_axis = (o->flags & 4) != 0 };

    // CUI-100 (docs/future/cui-upstream-bugs.md#cui-100): use the pointer position at release.
    const bool pressed = ui->mpressed[CUI_MOUSE_LEFT];
    const bool continued = !pressed && initialized && state->drag >= 0 && ui->active == key;
    const bool released = enabled && (pressed || continued) && fpas_drag_release(ui, key, continued, NULL);
    const float mouse_x = ui->mx, mouse_y = ui->my;
    const cui_id hot = ui->hot;
    if (released) {
        ui->mx = pressed ? ui->press_x : ui->release_x;
        ui->my = pressed ? ui->press_y : ui->release_y;
        ui->mdown[CUI_MOUSE_LEFT] = true;
        if (pressed) ui->hot = key;
    }

    // CUI-131 (docs/future/cui-upstream-bugs.md#cui-131): leave keys to the popup above the track.
    const bool mute_keys = enabled && cui__keys_covered(ui);
    uint8_t keys[CUI_KEY_COUNT];
    if (mute_keys) {
        memcpy(keys, ui->kpressed, sizeof keys);
        memset(ui->kpressed, 0, sizeof ui->kpressed);
    }
    cui_push_disabled(ui, !enabled);
    cui_event_track_result native = cui_event_track_ex(ui, id, times, count, capacity,
        o->minimum, o->maximum, &options);
    cui_pop_disabled(ui);
    if (mute_keys) memcpy(ui->kpressed, keys, sizeof keys);

    if (released) {
        state = cui__state_lazy(ui, key, sizeof *state, false);
        cui__ev_paint *paint = ui->nodes[node].paint_data;
        if (pressed) fpas_event_track_release(ui, key, state, paint, times, *count, o->step, &native);
        if (native_selection) state->sel = native.selected;
        state->drag = paint->drag = -1;
        native.dragging = false;
        ui->mx = mouse_x;
        ui->my = mouse_y;
        ui->hot = hot;
        ui->mdown[CUI_MOUSE_LEFT] = false;
    }
    result[0] = native.selected;
    result[1] = native.added;
    result[2] = native.removed;
    result[3] = native.dragging;
    result[4] = enabled ? (native.changed ? 1 : 0) | (prior_selection != native.selected ? 2 : 0) |
        (dragging != native.dragging ? 4 : 0) : 0;
    ui->last_node = node;
}
