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
    bool ok = true;
    for (int frame = 0; frame < 5; ++frame) {
        if (frame == 1) key(ui, CUI_KEY_TAB);
        if (frame == 2) key(ui, CUI_KEY_ENTER);
        if (frame == 3) key(ui, CUI_KEY_RIGHT);
        if (frame == 4) key(ui, CUI_KEY_ENTER);
        cui_begin_frame(ui, .016f);
        int part;
        const int clicked = adapter ? fpas_cui_gantt(ui, "gantt", date, 12, &model, 1, 0, 0, 0,
            0, 0, 3, -1, &part) : cui_gantt(ui, "gantt", &task, 1, origin, 12, .read_only = true);
        cui_end_frame(ui);
        if ((frame == 2 || frame == 4) && clicked != 0) {
            fprintf(stderr, "%s: Enter did not activate read-only Gantt task (got %d)\n",
                adapter ? "FPAS" : "CUI", clicked);
            ok = false;
        }
    }
    cui_destroy(ui);
    return ok && (adapter ? model.start : task.start) == 2;
}

int main(int argc, char **argv) {
    const bool adapter = argc < 2 || strcmp(argv[1], "upstream") != 0;
    const bool week = read_only_week(adapter);
    const bool gantt = read_only_gantt(adapter);
    return week && gantt ? 0 : 1;
}
