// [Documentation](../../../../../docs/ui/gui/planning/week-calendar.md)
#include "../../fpas_cui.h"

static bool fpas_week_final_drag(cui_ctx *ui, const cui__week_state *state, int count) {
    return state && state->drag > 0 && count >= state->last_count && !state->read_only &&
        ui->mreleased[CUI_MOUSE_LEFT] && !ui->mdown[CUI_MOUSE_LEFT] &&
        (state->moved || fabsf(ui->release_x - state->press_x) + fabsf(ui->release_y - state->press_y) > 4);
}

static void fpas_week_retire(cui_ctx *ui, cui_id key) {
    fpas_forget_state(ui, key ^ 0x5C3u);
}

int fpas_cui_week_calendar(cui_ctx *ui, const char *id, int week, fpas_week_event *events, int count,
    float start, float finish, float hour_height, int height_kind, float height, int flags, int focus, int *parts) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_id horizontal = key ^ 0x7EE4u, vertical = key ^ 0x5C3u;
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
    /* CUI-078: CUI indexes weekday captions from Monday while the shown dates start at week.
       See docs/future/cui-upstream-bugs.md#cui-078. */
    const int weekday = week % 7;
    const cui_strings *original = cui_get_strings(ui);
    cui_strings rotated;
    if (weekday != 0) {
        rotated = original ? *original : (cui_strings){0};
        for (int day = 0; day < 7; ++day)
            rotated.weekday_short[day] = cui__weekday_short(ui, weekday + day);
        cui_set_strings(ui, &rotated);
    }

    /* CUI-077: apply the release position through CUI's held-drag path.
       See docs/future/cui-upstream-bugs.md#cui-077. */
    cui__week_state *state = cui__state_lazy(ui, key, sizeof *state, false);
    const bool finish_drag = !options.read_only && fpas_week_final_drag(ui, state, count);
    const float mouse_x = ui->mx, mouse_y = ui->my;
    if (finish_drag) {
        ui->mx = ui->release_x;
        ui->my = ui->release_y;
        ui->mdown[CUI_MOUSE_LEFT] = true;
    }
    /* CUI-086: disregard wheel input after the host disables the calendar.
       See docs/future/cui-upstream-bugs.md#cui-086. */
    fpas_document_scroll_input(ui, horizontal, enabled);
    fpas_document_scroll_input(ui, vertical, enabled);
    cui_push_disabled(ui, !enabled);
    int clicked = cui_week_calendar_ex(ui, id, fpas_date_from_number(week), native, count, &options);
    cui_pop_disabled(ui);
    if (finish_drag) {
        ui->mdown[CUI_MOUSE_LEFT] = false;
        ui->mx = mouse_x;
        ui->my = mouse_y;
        state->drag = 0;
        if (ui->active == key) ui->active = 0;
        cui_request_redraw(ui);
    }
    if (weekday != 0) cui_set_strings(ui, original);
    if (!enabled) {
        const int x = fpas_find(ui, horizontal), y = fpas_find(ui, vertical);
        ui->nodes[x].paint = fpas_disabled_document_scroll_layout;
        ui->nodes[y].paint = fpas_disabled_document_scroll_layout;
    }
    for (int i = 0; i < count; ++i) {
        events[i].day = native[i].day;
        /* CUI-084: a duration longer than the visible hours can invert the native clamp.
           See docs/future/cui-upstream-bugs.md#cui-084. */
        if (native[i].start >= 0 && native[i].end <= 24 && native[i].end > native[i].start) {
            events[i].start = native[i].start;
            events[i].finish = native[i].end;
        } else {
            cui_request_redraw(ui);
        }
        parts[i] = fpas_find(ui, fpas_well_part(key ^ 0xE7u, i));
    }
    fpas_own(ui, key, horizontal, key ^ 0x6E1Du, 0);
    fpas_on_forget(ui, key, fpas_week_retire);
    fpas_creative_node(ui, node, key);
    return enabled ? clicked : -1;
}
