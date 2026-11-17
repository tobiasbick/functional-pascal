#include "../fpas_cui.h"
#include <stdio.h>
#include <string.h>

static void key(cui_ctx *ui, cui_key value) {
    cui_input_key(ui, value, true, 0);
    cui_input_key(ui, value, false, 0);
}

static bool read_only_week(bool adapter) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 940, 620, 1);
    cui_set_reduced_motion(ui, true);
    const cui_date monday = {2026, 11, 9};
    const int date = cui_date_to_days(monday) + 719162;
    int result = -1;
    int part;
    cui_cal_event e = {"Review", NULL, 0, 9, 11, CUI_TONE_INFO};
    fpas_week_event model = {"Review", NULL, 0, 9, 11, CUI_TONE_INFO};
    bool ok = true;
    for (int frame = 0; frame < 10; ++frame) {
        if (frame == 1) key(ui, CUI_KEY_TAB);
        if (frame == 2) key(ui, CUI_KEY_ENTER);
        if (frame == 3) key(ui, CUI_KEY_DOWN);
        if (frame == 4) key(ui, CUI_KEY_ENTER);
        if (frame == 5) cui_input_mouse_move(ui, 100, 145);
        if (frame == 6 || frame == 8) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        if (frame == 7 || frame == 9) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        if (frame == 9) cui_input_mouse_move(ui, 125, 164);
        cui_begin_frame(ui, .016f);
        result = adapter
            ? fpas_cui_week_calendar(ui, "week", date, &model, 1, 8, 18, 0, 0, 0, 3,
                -1, &part)
            : cui_week_calendar(ui, "week", monday, &e, 1, .start_hour = 8, .end_hour = 18, .read_only = true);
        cui_end_frame(ui);
        /* A drag coalesced into the release frame must not activate the event. */
        const int expected = frame == 2 || frame == 4 || frame == 7 ? 0 : -1;
        if (result != expected) {
            fprintf(stderr, "%s: read-only week activation at frame %d was %d, expected %d\n",
                adapter ? "FPAS" : "CUI", frame, result, expected);
            ok = false;
        }
    }
    cui_destroy(ui);
    return ok && (adapter ? model.start : e.start) == 9;
}

static bool read_only_gantt(bool adapter) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 940, 620, 1);
    cui_set_reduced_motion(ui, true);
    const cui_date origin = {2026, 11, 9};
    const int date = cui_date_to_days(origin) + 719162;
    cui_gantt_task task = {"Review", NULL, 2, 5, .5f, CUI_TONE_INFO, -1, false, CUI_GANTT_AUTO};
    fpas_gantt_task model = {"Review", NULL, 2, 5, .5f, CUI_TONE_INFO, -1, 0, CUI_GANTT_AUTO};
    int part = -1;
    bool ok = true;
    for (int frame = 0; frame < (adapter ? 8 : 5); ++frame) {
        if (frame == 1) key(ui, CUI_KEY_TAB);
        if (frame == 2) key(ui, CUI_KEY_ENTER);
        if (frame == 3) key(ui, CUI_KEY_RIGHT);
        if (frame == 4) key(ui, CUI_KEY_ENTER);
        if (frame == 5) {
            if (part < 0) { ok = false; break; }
            const cui_rect rect = cui_get_rect(ui, fpas_cui_node_key(ui, part));
            cui_input_mouse_move(ui, rect.x + rect.w / 2, rect.y + rect.h / 2);
        }
        if (frame == 6 || frame == 7) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, frame == 6);
        cui_begin_frame(ui, .016f);
        const int clicked = adapter ? fpas_cui_gantt(ui, "gantt", date, 12, &model, 1, 0, 0, 0,
            0, 0, 3, -1, &part) : cui_gantt(ui, "gantt", &task, 1, origin, 12, .read_only = true);
        cui_end_frame(ui);
        const int expected = frame == 2 || frame == 4 || frame == 7 ? 0 : -1;
        if (clicked != expected) {
            fprintf(stderr, "%s: read-only Gantt activation at frame %d was %d, expected %d\n",
                adapter ? "FPAS" : "CUI", frame, clicked, expected);
            ok = false;
        }
    }
    cui_destroy(ui);
    return ok && (adapter ? model.start : task.start) == 2;
}

/* mode: 0 read-only, 1 removed target, 2 disabled input. */
static bool interrupted_drag(bool gantt, int mode) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 940, 620, 1);
    cui_set_reduced_motion(ui, true);
    const int date = cui_date_to_days((cui_date){2026, 11, 9}) + 719162;
    fpas_gantt_task tasks[] = {
        {"Plan", NULL, 1, 4, .5f, CUI_TONE_INFO, -1, 0, CUI_GANTT_AUTO},
        {"Review", NULL, 3, 6, .5f, CUI_TONE_INFO, -1, 0, CUI_GANTT_AUTO}
    };
    fpas_week_event events[] = {
        {"Plan", NULL, 0, 9, 11, CUI_TONE_INFO},
        {"Review", NULL, 1, 9, 11, CUI_TONE_INFO}
    };
    const float day_width = 30, hour_height = 32;
    const float expected_start = gantt ? 4 : 10;
    int parts[] = {-1, -1};
    float x = 0, y = 0;
    bool ok = true;
    for (int frame = 0; frame < 9; ++frame) {
        if (frame == 2) {
            if (parts[1] < 0) { ok = false; break; }
            const cui_rect rect = cui_get_rect(ui, fpas_cui_node_key(ui, parts[1]));
            if (rect.w <= 0 || rect.h <= 0) { ok = false; break; }
            x = rect.x + rect.w / 2;
            y = rect.y + rect.h / 2;
            cui_input_mouse_move(ui, x, y);
        }
        if (frame == 3) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        if (frame >= 4 && frame <= 6) {
            const int distance = frame - 3;
            cui_input_mouse_move(ui, x + (gantt ? day_width * distance : 0),
                y + (gantt ? 0 : hour_height * distance));
        }
        if (frame == 7) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        const int count = frame == 5 && mode == 1 ? 1 : 2;
        const int flags = frame == 5 ? (mode == 2 ? 0 : mode == 0 ? 3 : 1) : 1;
        cui_begin_frame(ui, .016f);
        if (gantt) fpas_cui_gantt(ui, "gantt", date, 12, tasks, count, 0, day_width, 32,
            0, 0, flags, -1, parts);
        else fpas_cui_week_calendar(ui, "week", date, events, count, 8, 18, hour_height,
            0, 0, flags, -1, parts);
        cui_end_frame(ui);
        if (frame >= 4) {
            const float start = gantt ? tasks[1].start : events[1].start;
            if (start == expected_start) continue;
            fprintf(stderr, "%s drag, %s at frame %d: start was %g, expected %g\n",
                gantt ? "Gantt" : "Week", mode == 1 ? "item removed" : mode == 2 ? "disabled" : "read-only",
                frame, start, expected_start);
            ok = false;
            break;
        }
    }
    cui_destroy(ui);
    return ok;
}

int main(int argc, char **argv) {
    const bool adapter = argc < 2 || strcmp(argv[1], "upstream") != 0;
    const bool week = read_only_week(adapter);
    const bool gantt = read_only_gantt(adapter);
    if (!adapter) return week && gantt ? 0 : 1;
    const bool gantt_removed = interrupted_drag(true, 1);
    const bool gantt_read_only = interrupted_drag(true, 0);
    const bool week_read_only = interrupted_drag(false, 0);
    const bool week_removed = interrupted_drag(false, 1);
    const bool gantt_disabled = interrupted_drag(true, 2);
    const bool week_disabled = interrupted_drag(false, 2);
    return week && gantt && gantt_removed && gantt_read_only && week_read_only
        && week_removed && gantt_disabled && week_disabled ? 0 : 1;
}
