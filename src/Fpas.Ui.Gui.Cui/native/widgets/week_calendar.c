#include "../fpas_cui.h"

int fpas_cui_week_calendar(cui_ctx *ui, const char *id, int week, fpas_week_event *events, int count,
    float start, float finish, float hour_height, int height_kind, float height, int flags, int focus, int *parts) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (enabled && focus >= 0 && focus < count)
        ui->focus_next_request = fpas_well_part(key ^ 0xE7u, focus);
    cui_cal_event *native = cui__frame_alloc(ui, sizeof *native * (size_t)CUI_MAX(count, 1));
    for (int i = 0; i < count; ++i) {
        const fpas_week_event *e = &events[i];
        native[i] = (cui_cal_event){ e->title, e->detail, e->day, e->start, e->finish, e->tone };
    }
    const cui_week_opts options = { .start_hour = start, .end_hour = finish, .hour_height = hour_height,
        .height = fpas_extent(height_kind, height), .read_only = (flags & 2) != 0 || !enabled };
    cui__week_state *state = cui__state_lazy(ui, key, sizeof *state, false);
    int activated = -1;
    /* CUI-032: native read-only weeks also skip the appointment's release. */
    if (state && state->drag > 0) {
        if (!enabled || state->drag > count) {
            state->drag = 0;
        } else if (options.read_only) {
            if (fabsf(ui->mx - state->press_x) + fabsf(ui->my - state->press_y) > 4) state->moved = true;
            if (!ui->mdown[CUI_MOUSE_LEFT]) {
                if (!state->moved && ui->mreleased[CUI_MOUSE_LEFT]) activated = state->drag - 1;
                state->drag = 0;
            }
        }
    }
    cui_push_disabled(ui, !enabled);
    int clicked = cui_week_calendar_ex(ui, id, fpas_date_from_number(week), native, count, &options);
    cui_pop_disabled(ui);
    if (clicked < 0) clicked = activated;
    for (int i = 0; i < count; ++i) {
        events[i].day = native[i].day;
        events[i].start = native[i].start;
        events[i].finish = native[i].end;
        parts[i] = fpas_find(ui, fpas_well_part(key ^ 0xE7u, i));
        /* CUI-032 (docs/future/cui-upstream-bugs.md#cui-032): native read-only
           mode also suppresses keyboard activation of the appointment. */
        if (enabled && options.read_only && clicked < 0 && parts[i] >= node && ui->focus_visible &&
            fpas_well_keys(ui, ui->nodes[parts[i]].id) &&
            (cui_key_pressed(ui, CUI_KEY_ENTER) || cui_key_pressed(ui, CUI_KEY_SPACE))) clicked = i;
    }
    fpas_own(ui, key, key ^ 0x7EE4u, key ^ 0x6E1Du, 0);
    fpas_creative_node(ui, node, key);
    return enabled ? clicked : -1;
}
