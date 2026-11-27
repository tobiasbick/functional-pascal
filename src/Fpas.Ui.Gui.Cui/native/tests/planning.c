// [Documentation](../../../../docs/ui/gui/planning/README.md)
// [Month calendar](../../../../docs/ui/gui/planning/month-calendar.md)
#include "../fpas_cui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void key(cui_ctx *ui, cui_key value) {
    cui_input_key(ui, value, true, 0);
    cui_input_key(ui, value, false, 0);
}

/** Checks CUI's signed day numbers against the supported CLR date domain. */
static bool calendar_day_numbers(void) {
    static const struct { cui_date date; int day; } known[] = {
        {{1, 1, 1}, -719162}, {{1969, 12, 31}, -1}, {{1970, 1, 1}, 0},
        {{2000, 2, 29}, 11016}, {{2100, 3, 1}, 47541}, {{9999, 12, 31}, 2932896}
    };
    for (size_t i = 0; i < sizeof known / sizeof known[0]; ++i) {
        const cui_date actual = cui_date_from_days(known[i].day);
        const cui_date expected = known[i].date;
        if (actual.year != expected.year || actual.month != expected.month || actual.day != expected.day ||
            cui_date_to_days(expected) != known[i].day) return false;
    }
    return true;
}

typedef struct {
    int month, selected, today, activated;
    cui_rect grid, next;
} month_fixture;

/** Draws a model-owned month and receives its proposed date and event. */
static int draw_month(cui_ctx *ui, month_fixture *state, int focus) {
    const int day = 719162 - 17; /* December 15, 1969 */
    const fpas_month_event event = {"Review", day, -1, 9.5f, 1, CUI_TONE_INFO};
    int parts[4];
    cui_begin_frame(ui, .016f);
    const int flags = fpas_cui_month_calendar(ui, "month", &state->month, &state->selected,
        &event, 1, 0, 104, NULL, NULL, state->today, 1, focus, parts, &state->activated);
    const cui_id grid = fpas_cui_node_key(ui, parts[0]);
    const cui_id next = fpas_cui_node_key(ui, parts[2]);
    cui_end_frame(ui);
    state->grid = cui_get_rect(ui, grid);
    state->next = cui_get_rect(ui, next);
    return flags;
}

/** Checks localized, customized month paint at full and compact widths. */
static bool month_pixels_match(int width) {
    enum { HEIGHT = 690 };
    static const char *const months[] = {
        "Januar", "Februar", "Maerz", "April", "Mai", "Juni",
        "Juli", "August", "September", "Oktober", "November", "Dezember"
    };
    static const char *const days[] = { "Mo", "Di", "Mi", "Do", "Fr", "Sa", "So" };
    const cui_date month = {2024, 2, 1}, date = {2024, 2, 29};
    const int first = cui_date_to_days(month) + 719162;
    const int selected = cui_date_to_days(date) + 719162;
    const cui_month_event appointment = {"Review", date, {0}, 9.5f, false, CUI_TONE_INFO};
    const fpas_month_event event = {"Review", selected, -1, 9.5f, 0, CUI_TONE_INFO};
    const size_t bytes = (size_t)width * HEIGHT * 4;
    uint8_t *pixels[2] = { calloc(1, bytes), calloc(1, bytes) };
    bool valid = pixels[0] && pixels[1];
    for (int pass = 0; pass < 2 && valid; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        cui_ctx *ui = renderer ? cui_create(&(cui_ctx_desc){.renderer = cui_sw_iface(renderer)}) : NULL;
        if (!ui) { cui_sw_destroy(renderer); valid = false; break; }
        cui_set_viewport(ui, width, HEIGHT, 1);
        cui_set_reduced_motion(ui, true);
        for (int frame = 0; frame < 3; ++frame) {
            cui_begin_frame(ui, .016f);
            if (pass) {
                int shown = first, day = selected, parts[4], activated;
                fpas_cui_month_calendar(ui, "month", &shown, &day, &event, 1, 6, 90,
                    months, days, selected, 3, 0, parts, &activated);
            } else {
                cui_date shown = month, day = date;
                cui_month_calendar(ui, "month", &shown, &appointment, 1, .selected = &day,
                    .first_weekday = 6, .cell_height = 90, .month_names = months,
                    .day_names = days, .hour12 = true, .today = date);
            }
            cui_end_frame(ui);
        }
        cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], width, HEIGHT, width * 4, CUI_SW_RGBA);
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    valid = valid && memcmp(pixels[0], pixels[1], bytes) == 0;
    free(pixels[0]);
    free(pixels[1]);
    if (!valid) fprintf(stderr, "Localized month calendar differs at %d logical units.\n", width);
    return valid;
}

/** Uses pointer and keyboard input across the Unix epoch and the CLR range edges. */
static bool month_calendar_contract(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 700, 700, 1);
    cui_set_reduced_motion(ui, true);
    month_fixture state = { .month = 719162 - 31, .selected = -1, .today = 719162 - 30 };
    draw_month(ui, &state, 0);
    draw_month(ui, &state, 0);

    /* December 1969 starts on Monday. Click its third Monday and then its appointment. */
    const float x = state.grid.x + state.grid.w / 14;
    const float y = state.grid.y + 2 * 104;
    cui_input_mouse_move(ui, x, y + 12);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true); draw_month(ui, &state, 0);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    bool valid = draw_month(ui, &state, 0) == 1 && state.selected == 719162 - 17;
    cui_input_mouse_move(ui, x, y + 36);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true); draw_month(ui, &state, 0);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false); draw_month(ui, &state, 0);
    valid = valid && state.activated == 0;

    state.selected = 719162 - 1;
    draw_month(ui, &state, 1);
    key(ui, CUI_KEY_RIGHT);
    valid = draw_month(ui, &state, 0) == 5 && state.month == 719162 && state.selected == 719162 && valid;
    key(ui, CUI_KEY_LEFT);
    valid = draw_month(ui, &state, 0) == 5 && state.month == 719162 - 31 && state.selected == 719162 - 1 && valid;

    const int display = cui_date_to_days((cui_date){2025, 5, 1}) + 719162;
    state.month = display;
    state.selected = -1;
    valid = draw_month(ui, &state, 1) == 0 && state.month == display && valid;
    state.selected = display + 4;
    key(ui, CUI_KEY_ENTER);
    valid = draw_month(ui, &state, 0) == 3 && state.selected == display + 4 && valid;
    key(ui, CUI_KEY_HOME);
    valid = draw_month(ui, &state, 0) == 5 && state.month == 719162 - 31 && state.selected == state.today && valid;

    state.month = state.selected = 0;
    draw_month(ui, &state, 1);
    key(ui, CUI_KEY_LEFT);
    valid = draw_month(ui, &state, 0) == 0 && state.month == 0 && state.selected == 0 && valid;
    key(ui, CUI_KEY_PAGE_UP);
    valid = draw_month(ui, &state, 0) == 0 && state.month == 0 && state.selected == 0 && valid;
    state.month = 3652058 - 30;
    state.selected = 3652058;
    draw_month(ui, &state, 1);
    const float december_height = state.grid.h;
    key(ui, CUI_KEY_RIGHT);
    valid = draw_month(ui, &state, 0) == 0 && state.month == 3652058 - 30 &&
        state.selected == 3652058 && state.grid.h == december_height && valid;
    cui_input_mouse_move(ui, state.next.x + state.next.w / 2, state.next.y + state.next.h / 2);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true); draw_month(ui, &state, 0);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    valid = draw_month(ui, &state, 0) == 0 && state.month == 3652058 - 30 &&
        state.grid.h == december_height && valid;

    cui_destroy(ui);
    if (!valid) fprintf(stderr, "Month calendar date domain, focus or selection mismatch.\n");
    return valid && calendar_day_numbers() && month_pixels_match(700) && month_pixels_match(420);
}

static bool draw_kanban(cui_ctx *ui, const char *title, int source, cui_rect *card, cui_kanban_move *move, int *added) {
    int node = -1;
    if (added) *added = -1;
    cui_begin_frame(ui, .016f);
    const int root = fpas_cui_kanban_begin(ui, "board", 1);
    for (int column = 0; column < 2; ++column) {
        int parts[2];
        if (fpas_cui_kanban_column_begin(ui, column ? "Ready" : title, source == column, 0, 0, 0, 220, parts) && added)
            *added = column;
        if (source == column) {
            fpas_cui_kanban_card_begin(ui, 7, 0, &node);
            cui_text(ui, "Plan");
            fpas_cui_kanban_card_end(ui);
        }
        fpas_cui_kanban_column_end(ui);
    }
    const bool changed = fpas_cui_kanban_end(ui, root, move);
    cui_end_frame(ui);
    if (card) *card = fpas_cui_node_rect(ui, node);
    return changed;
}

/** Releasing a lifted card proposes a move only while its source column holds it. */
static bool kanban_source_change(int source) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 650, 380, 1);
    cui_set_reduced_motion(ui, true);
    cui_rect card;
    cui_kanban_move move = {0};
    draw_kanban(ui, "Backlog", 0, &card, &move, NULL);
    const float x = card.x + card.w / 2, y = card.y + card.h / 2;
    cui_input_mouse_move(ui, x, y);
    draw_kanban(ui, "Backlog", 0, NULL, &move, NULL);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    draw_kanban(ui, "Backlog", 0, NULL, &move, NULL);
    cui_input_mouse_move(ui, x + 234, y);
    draw_kanban(ui, "Backlog", 0, NULL, &move, NULL);
    draw_kanban(ui, "Backlog", source, NULL, &move, NULL);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    const bool moved = draw_kanban(ui, "Backlog", source, NULL, &move, NULL);
    const bool valid = source == 0
        ? moved && move.card == 7 && move.from_column == 0 && move.to_column == 1 && move.to_index == 0
        : !moved;
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "Kanban reported the wrong move for model source %d.\n", source);
    return valid;
}

static bool kanban_keyboard_move(bool leave_card) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 650, 380, 1);
    cui_set_reduced_motion(ui, true);
    cui_kanban_move move = {0};
    draw_kanban(ui, "Backlog", 0, NULL, &move, NULL);
    const cui_key presses[] = { CUI_KEY_TAB, CUI_KEY_TAB, CUI_KEY_SPACE, CUI_KEY_RIGHT, CUI_KEY_TAB };
    for (int step = 0; step < (leave_card ? 5 : 4); ++step) {
        key(ui, presses[step]);
        draw_kanban(ui, "Backlog", 0, NULL, &move, NULL);
    }
    key(ui, CUI_KEY_ENTER);
    int added;
    const bool moved = draw_kanban(ui, "Backlog", 0, NULL, &move, &added);
    const bool valid = leave_card ? !moved && added == 1
        : moved && move.card == 7 && move.from_column == 0 && move.to_column == 1 && added == -1;
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "Kanban lift did not respect keyboard focus.\n");
    return valid;
}

static bool kanban_literal_title(void) {
    enum { WIDTH = 480, HEIGHT = 180 };
    static unsigned char pixels[2][WIDTH * HEIGHT * 4];
    const char *titles[] = { "Backlog", "Backlog##revised" };
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(renderer) });
        if (!ui) { cui_sw_destroy(renderer); return false; }
        cui_set_viewport(ui, WIDTH, HEIGHT, 1);
        cui_set_reduced_motion(ui, true);
        draw_kanban(ui, titles[pass], -1, NULL, NULL, NULL);
        cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    const bool valid = memcmp(pixels[0], pixels[1], sizeof pixels[0]) != 0;
    if (!valid) fprintf(stderr, "Kanban truncated a literal column title.\n");
    return valid;
}

static bool kanban_contract(void) {
    return kanban_source_change(0) && kanban_source_change(-1) && kanban_source_change(1) &&
        kanban_keyboard_move(false) && kanban_keyboard_move(true) && kanban_literal_title();
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
    for (int frame = 0; frame < 12; ++frame) {
        if (frame == 1) key(ui, CUI_KEY_TAB);
        if (frame == 2) key(ui, CUI_KEY_ENTER);
        if (frame == 3) key(ui, CUI_KEY_DOWN);
        if (frame == 4) key(ui, CUI_KEY_ENTER);
        if (frame == 5) cui_input_mouse_move(ui, 100, 145);
        if (frame == 10) cui_input_mouse_move(ui, 100, 145);
        if (frame == 6 || frame == 8 || frame == 10) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        if (frame == 9) cui_input_mouse_move(ui, 125, 164);
        if (frame == 7 || frame == 9 || frame == 11) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        if (frame == 11) cui_input_mouse_move(ui, 125, 164);
        cui_begin_frame(ui, .016f);
        result = adapter
            ? fpas_cui_week_calendar(ui, "week", date, &model, 1, 8, 18, 0, 0, 0, 3,
                -1, &part)
            : cui_week_calendar(ui, "week", monday, &e, 1, .start_hour = 8, .end_hour = 18, .read_only = true);
        cui_end_frame(ui);
        /* Motion before release is a drag; motion after release preserves the click. */
        const int expected = frame == 2 || frame == 4 || frame == 7 || frame == 11 ? 0 : -1;
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

/** Captures the native task bars and chart rectangle within an explicit parent width. */
static cui_rect gantt_frame(cui_ctx *ui, float width, int height_kind, float height,
    int flags, cui_rect *bars) {
    const int date = cui_date_to_days((cui_date){2026, 11, 9}) + 719162;
    fpas_gantt_task tasks[] = {
        {"Plan", "Maya", 1, 4, .5f, CUI_TONE_INFO, -1, 0, CUI_GANTT_AUTO},
        {"Review", "", 3, 6, .5f, CUI_TONE_INFO, 0, 0, CUI_GANTT_ACTIVE},
        {"Deploy", NULL, 6, 6, 0, CUI_TONE_NEUTRAL, 1, 1, CUI_GANTT_PLANNED}
    };
    int root = -1, parts[3];
    cui_begin_frame(ui, .016f);
    cui_box(ui, .width = CUI_PX(width), .height = CUI_PX(240)) {
        fpas_cui_gantt(ui, "gantt", date, 12, tasks, 3, 0, 30, 40, height_kind, height, flags, -1, parts);
        root = fpas_cui_last_node(ui);
    }
    cui_end_frame(ui);
    for (int i = 0; i < 3; ++i) bars[i] = fpas_cui_node_rect(ui, parts[i]);
    return fpas_cui_node_rect(ui, root);
}

/** Keeps capped rows aligned and refreshes native density when the parent width changes. */
static bool gantt_height_and_layout(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 940, 620, 1);
    cui_set_reduced_motion(ui, true);
    cui_rect bars[3];
    for (int i = 0; i < 3; ++i) gantt_frame(ui, 400, 0, 0, 1, bars);
    const float slim = bars[0].x;
    bool ok = !cui_needs_redraw(ui);
    gantt_frame(ui, 600, 0, 0, 1, bars);
    ok &= cui_needs_redraw(ui);
    gantt_frame(ui, 600, 0, 0, 1, bars);
    ok &= bars[0].x - slim == 102;
    gantt_frame(ui, 400, 0, 0, 1, bars);
    ok &= cui_needs_redraw(ui);
    gantt_frame(ui, 400, 0, 0, 1, bars);
    ok &= bars[0].x == slim;
    cui_destroy(ui);

    ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 940, 620, 1);
    cui_set_reduced_motion(ui, true);
    cui_rect viewport = gantt_frame(ui, 600, 2, 100, 1, bars);
    const float last_y = bars[2].y;
    ok &= viewport.h == 100 && last_y >= viewport.y + viewport.h;
    cui_input_mouse_move(ui, viewport.x + 320, viewport.y + 74);
    cui_input_scroll(ui, 0, -3);
    viewport = gantt_frame(ui, 600, 2, 100, 1, bars);
    const float scrolled = bars[2].y;
    ok &= scrolled < last_y && scrolled < viewport.y + viewport.h;
    cui_input_scroll(ui, 0, 1);
    gantt_frame(ui, 600, 2, 100, 0, bars);
    ok &= bars[2].y == scrolled;
    viewport = gantt_frame(ui, 600, 3, 50, 1, bars);
    ok &= viewport.h == 120;
    viewport = gantt_frame(ui, 600, 1, 0, 1, bars);
    ok &= viewport.h == 240;
    cui_destroy(ui);
    if (!ok) fprintf(stderr, "Gantt viewport sizing, scroll admission or responsive density mismatch.\n");
    return ok;
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

static int week_frame(cui_ctx *ui, fpas_week_event *event, int flags, int *part) {
    const int week = cui_date_to_days((cui_date){2026, 11, 9}) + 719162;
    cui_begin_frame(ui, .016f);
    const int clicked = fpas_cui_week_calendar(ui, "week", week, event, 1,
        8, 18, 52, 0, 0, flags, -1, part);
    cui_end_frame(ui);
    return clicked;
}

static bool week_release(bool resize, bool long_event) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 940, 620, 1);
    cui_set_reduced_motion(ui, true);
    fpas_week_event event = {"Review", NULL, 0, long_event ? 1 : 9, long_event ? 22 : 11, CUI_TONE_INFO};
    int part;
    week_frame(ui, &event, 1, &part);
    week_frame(ui, &event, 1, &part);
    const cui_rect rect = cui_get_rect(ui, fpas_cui_node_key(ui, part));
    const float x = rect.x + rect.w / 2;
    const float y = rect.y + (resize ? rect.h - 2 : rect.h / 2);
    cui_input_mouse_move(ui, x, y);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    week_frame(ui, &event, 1, &part);

    cui_input_mouse_move(ui, x + (resize ? 0 : 126), y + 52);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    cui_input_mouse_move(ui, 10, 10);
    const int clicked = week_frame(ui, &event, 1, &part);
    const bool valid = clicked == -1 && event.day == (resize ? 0 : 1) &&
        event.start == (long_event ? 1 : resize ? 9 : 10) && event.finish == (long_event ? 22 : 12);
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "Week release: resize=%d long=%d day=%d start=%g finish=%g click=%d\n",
        resize, long_event, event.day, event.start, event.finish, clicked);
    return valid;
}

static bool week_disabled_scroll(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 940, 620, 1);
    cui_set_reduced_motion(ui, true);
    fpas_week_event event = {"Review", NULL, 0, 10, 12, CUI_TONE_INFO};
    int part;
    week_frame(ui, &event, 1, &part);
    week_frame(ui, &event, 1, &part);
    const cui_id appointment = fpas_cui_node_key(ui, part);
    const cui_rect initial = cui_get_rect(ui, appointment);
    cui_input_mouse_move(ui, initial.x + initial.w / 2, initial.y + 12);
    cui_input_scroll(ui, 0, -2);
    week_frame(ui, &event, 0, &part);
    bool valid = cui_get_rect(ui, appointment).y == initial.y;
    cui_input_scroll(ui, 0, -2);
    week_frame(ui, &event, 0, &part);
    valid &= cui_get_rect(ui, appointment).y == initial.y;
    week_frame(ui, &event, 3, &part);
    cui_input_scroll(ui, 0, -2);
    week_frame(ui, &event, 3, &part);
    valid &= cui_get_rect(ui, appointment).y < initial.y;
    fpas_cui_forget(ui, cui_make_id(ui, "week"));
    week_frame(ui, &event, 1, &part);
    valid &= cui_get_rect(ui, appointment).y == initial.y;
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "Week scroll admission or retirement failed.\n");
    return valid;
}

static bool week_localized_headers(void) {
    enum { WIDTH = 760, HEIGHT = 420 };
    static unsigned char pixels[2][WIDTH * HEIGHT * 4];
    const cui_date date = {2026, 11, 11};
    const int number = cui_date_to_days(date) + 719162;
    const cui_strings localized = { .weekday_short = {"M1", "T2", "W3", "T4", "F5", "S6", "S7"} };
    const cui_strings rotated = { .weekday_short = {"W3", "T4", "F5", "S6", "S7", "M1", "T2"} };
    bool valid = true;
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(renderer) });
        if (!ui) { cui_sw_destroy(renderer); return false; }
        cui_set_viewport(ui, WIDTH, HEIGHT, 1);
        cui_set_reduced_motion(ui, true);
        cui_set_strings(ui, pass ? &localized : &rotated);
        for (int frame = 0; frame < 3; ++frame) {
            cui_begin_frame(ui, .016f);
            if (pass) {
                fpas_cui_week_calendar(ui, "week", number, NULL, 0, 8.5f, 19.5f,
                    36, 3, 65, 1, -1, NULL);
                valid &= cui_get_strings(ui) == &localized;
            } else {
                cui_week_calendar(ui, "week", date, NULL, 0, .start_hour = 8.5f,
                    .end_hour = 19.5f, .hour_height = 36, .height = CUI_PCT(65));
            }
            cui_end_frame(ui);
        }
        cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    valid &= memcmp(pixels[0], pixels[1], sizeof pixels[0]) == 0;
    if (!valid) fprintf(stderr, "Week weekdays did not follow the localized starting date.\n");
    return valid;
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
        && week_removed && gantt_disabled && week_disabled && kanban_contract()
        && week_release(false, false) && week_release(true, false) && week_release(false, true)
        && week_localized_headers() && week_disabled_scroll() && gantt_height_and_layout()
        && month_calendar_contract() ? 0 : 1;
}
