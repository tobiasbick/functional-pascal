// [Documentation](../../../../docs/ui/gui/developer-tools/README.md)
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>
#include <string.h>

static int failures;

static void check(bool condition, const char *message) {
    if (!condition) {
        fprintf(stderr, "%s\n", message);
        ++failures;
    }
}

static cui_ctx *create(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return NULL;
    cui_set_viewport(ui, 700, 550, 1);
    cui_set_reduced_motion(ui, true);
    return ui;
}

static void trace(bool adapter) {
    cui_ctx *ui = create();
    if (!ui) { check(false, "Could not open trace context."); return; }
    cui_trace_track track = { "Main", false };
    const cui_trace_span spans[] = { { "Frame", 0, 1, 0, 0 }, { "Paint", 0.25, 0.5, 0, 1 } };
    fpas_dev_result result;
    cui_trace_view view = { 0, 1, 0, 0, 0 };
    for (int frame = 0; frame < 8; ++frame) {
        if (frame == 2) {
            cui_input_mouse_move(ui, 15, 38);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        }
        if (frame == 3) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        if (frame == 4) {
            cui_input_key(ui, CUI_KEY_A, true, CUI_MOD_SHIFT);
            cui_input_mouse_move(ui, 235, 38);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        }
        if (frame == 5) cui_input_mouse_move(ui, 395, 38);
        if (frame == 6) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        if (frame == 7) cui_input_key(ui, CUI_KEY_A, false, 0);
        cui_begin_frame(ui, 0.016f);
        if (adapter) {
            int hovered_link = -1;
            fpas_cui_trace_viewer(ui, "trace", &track, 1, spans, 2, NULL, 0,
                view.start, view.end, view.scroll, view.mark_start, view.mark_end,
                -1, 180, 0, 1, 0, 1, &result, NULL, 0, &hovered_link);
        } else {
            int selected = -1;
            const cui_trace_result native = cui_trace_view(ui, "trace", &track, 1, spans, 2,
                &view, &selected, .height = 180);
            result = (fpas_dev_result){ .flags = native.view_changed ? 2 : 0,
                .x = view.start, .y = view.end, .z = view.scroll, .a = view.mark_start, .b = view.mark_end };
        }
        cui_end_frame(ui);
        if (adapter && frame == 2) {
            check(result.action == 0 && result.count == 1, "Clicking the trace lane must propose collapsing it.");
            track.collapsed = result.count != 0;
        }
        if (frame == 5) {
            check((result.flags & 2) != 0 && result.b > result.a,
                "Shift-drag must propose a measurement without panning.");
        }
        view = (cui_trace_view){ result.x, result.y, (float)result.z, result.a, result.b };
    }
    cui_destroy(ui);
}

static void trace_disables_pending_input(void) {
    cui_ctx *ui = create();
    if (!ui) { check(false, "Could not open a passive trace context."); return; }
    const cui_trace_track track = { "Main", false };
    const cui_trace_span span = { "Frame", 0, 1, 0, 0, 0 };
    fpas_dev_result result;
    double mark_start = 0, mark_end = 0;
    for (int frame = 0; frame < 5; ++frame) {
        if (frame == 1) {
            cui_input_key(ui, CUI_KEY_A, true, CUI_MOD_SHIFT);
            cui_input_mouse_move(ui, 310, 48);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        }
        if (frame == 2) {
            cui_input_mouse_move(ui, 460, 48);
            cui_input_scroll(ui, 0, 3);
        }
        if (frame == 3) {
            cui_input_key(ui, CUI_KEY_A, false, 0);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        }
        if (frame == 4) cui_input_scroll(ui, 0, 3);
        cui_begin_frame(ui, .016f);
        int hovered_link = -1;
        const int flags = frame == 0 ? 3 : frame == 2 || frame == 4 ? 0 : 1;
        fpas_cui_trace_viewer(ui, "trace", &track, 1, &span, 1, NULL, 0, .2, .8,
            0, mark_start, mark_end, -1, 180, 0, 1, 0, flags, &result, NULL, 0, &hovered_link);
        cui_end_frame(ui);
        if (frame == 0) check(cui_focused_id(ui) == cui_make_id(ui, "trace"), "Trace must accept requested focus.");
        if (frame > 1)
            check(result.x == .2 && fabs(result.y - .8) < 1e-9 &&
                result.a == mark_start && result.b == mark_end,
                "Disabled trace wheel and a retired measurement must leave its view unchanged.");
        mark_start = result.a;
        mark_end = result.b;
    }
    cui_destroy(ui);
}

static void waterfall_details_and_focus(bool retained) {
    cui_ctx *ui = create();
    if (!ui) { check(false, "Could not open waterfall context."); return; }
    const cui_net_request request = { "https://example.test/data", NULL, 200, "fetch", 42, 0,
        { .02f, .08f, .05f, .09f, .1f, .05f } };
    fpas_dev_result result;
    int selected = -1;
    for (int frame = 0; frame < 23; ++frame) {
        if (frame == 1) cui_input_mouse_move(ui, 520, 45);
        if (frame == 18 || frame == 19) {
            const cui_key key = frame == 18 ? CUI_KEY_HOME : CUI_KEY_ENTER;
            cui_input_key(ui, key, true, 0);
            cui_input_key(ui, key, false, 0);
        }
        cui_begin_frame(ui, .06f);
        fpas_cui_network_waterfall(ui, "net", &request, frame == 21 ? 0 : 1, retained ? -2 : selected, 200, .05, .2,
            1, 0, frame == 0 ? 3 : 1, &result);
        cui_end_frame(ui);
        if (frame == 0) check(fpas_cui_focused(ui, fpas_cui_last_node(ui)), "Waterfall must report focus at its root.");
        if (frame == 17) {
            int tip = -1;
            const char *text = fpas_cui_tooltip_inspect(ui, &tip);
            check(result.hover == 0 && text && strstr(text, request.name) && strstr(text, "DNS") && strstr(text, "Total"),
                "Waterfall must show the request timing when hovering its bar.");
        }
        if (frame == 18) check(result.index == 0 && result.flags, "Home must report a changed waterfall selection.");
        if (frame == 19) check(result.action == 0 && !result.flags, "Enter must activate the retained waterfall selection.");
        if (frame == 20) check(result.index == 0 && !result.flags, "An idle waterfall must keep its current selection.");
        if (frame == 21) check(result.index == -1 && result.flags, "Removing the selected request must clear selection.");
        if (frame == 22) check(result.index == -1 && !result.flags, "A new request must not restore a removed selection.");
        selected = result.index;
    }
    cui_destroy(ui);
}

static void regex(void) {
    cui_ctx *ui = create();
    if (!ui) { check(false, "Could not open regex context."); return; }
    char pattern[32] = "a+";
    char sample[32] = "aa b aaa";
    fpas_dev_result result;
    cui_begin_frame(ui, 0.016f);
    fpas_cui_regex_tester(ui, "regex", pattern, sizeof pattern, sample, sizeof sample, 0, 110, 1, 0, 1, &result);
    cui_end_frame(ui);
    check(result.flags && result.count == 2, "Regex tester must return the native match count.");
    pattern[0] = '(';
    pattern[1] = 0;
    cui_begin_frame(ui, 0.016f);
    fpas_cui_regex_tester(ui, "regex", pattern, sizeof pattern, sample, sizeof sample, 0, 110, 1, 0, 1, &result);
    cui_end_frame(ui);
    check(!result.flags, "An invalid regex must be reported after an edit.");
    cui_destroy(ui);
}

typedef enum { REQUEST_URL, REGEX_PATTERN, REGEX_SAMPLE } text_field;

static void developer_text_replacement(text_field field, int tabs) {
    cui_ctx *ui = create();
    if (!ui) { check(false, "Could not open developer editor context."); return; }
    char pattern[32] = "old", sample[32] = "old";
    char *value = field == REGEX_SAMPLE ? sample : pattern;
    for (int frame = 0; frame <= tabs + 5; ++frame) {
        if (frame > 0 && frame <= tabs) {
            cui_input_key(ui, CUI_KEY_TAB, true, 0);
            cui_input_key(ui, CUI_KEY_TAB, false, 0);
        }
        if (frame == tabs + 1 || frame == tabs + 4) {
            cui_input_key(ui, CUI_KEY_NONE, false, 0);
            cui_input_text(ui, "x");
        }
        if (frame == tabs + 2) strcpy(value, "replacement");
        if (frame == tabs + 3 || frame == tabs + 5) {
            cui_input_key(ui, CUI_KEY_Z, true, CUI_MOD_CTRL);
            cui_input_key(ui, CUI_KEY_Z, false, CUI_MOD_CTRL);
        }
        cui_begin_frame(ui, 0.016f);
        fpas_dev_result result;
        if (field == REQUEST_URL)
            fpas_cui_request_bar(ui, "editor", 0, pattern, sizeof pattern, 0, "URL", 1, 0, 1, &result);
        else
            fpas_cui_regex_tester(ui, "editor", pattern, sizeof pattern, sample, sizeof sample, 0, 110, 1, 0, 1, &result);
        cui_end_frame(ui);
        if (frame == tabs + 1)
            check(strcmp(value, "xold") == 0, "The developer field must accept the initial edit.");
        if (frame == tabs + 3)
            check(strcmp(value, "replacement") == 0, "Undo must preserve replacement developer field text.");
        if (frame == tabs + 4)
            check(strcmp(value, "replacementx") == 0, "Editing must resume at the end of replacement text.");
        if (frame == tabs + 5)
            check(strcmp(value, "replacement") == 0, "Undo must still work for edits to replacement text.");
    }
    cui_destroy(ui);
}

static void developer_placeholders(void) {
    enum { WIDTH = 640, HEIGHT = 240 };
    static unsigned char pixels[2][WIDTH * HEIGHT * 4];
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(renderer) });
        if (!ui) {
            check(false, "Could not open developer placeholder renderer.");
            cui_sw_destroy(renderer);
            return;
        }
        cui_set_viewport(ui, WIDTH, HEIGHT, 1);
        for (int frame = 0; frame < 2; ++frame) {
            char url[32] = "", placeholder[] = "Original target";
            char key[] = "Original key", value[] = "Original value";
            cui_kv_pair pairs[1] = {0};
            int count = 0;
            fpas_dev_result result;
            cui_begin_frame(ui, 0.016f);
            fpas_cui_request_bar(ui, "request", 0, url, sizeof url, 0, placeholder, 1, 0, 1, &result);
            fpas_cui_key_value_editor(ui, "pairs", pairs, &count, 1, key, value, 0, 0, 0, 1, 0, 1, &result);
            if (pass) placeholder[0] = key[0] = value[0] = '\0';
            cui_end_frame(ui);
        }
        cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    check(memcmp(pixels[0], pixels[1], sizeof pixels[0]) == 0,
        "Developer field placeholders must survive caller buffer changes before painting.");
}

/** Draws a two-file report and returns the adapter result. */
static fpas_dev_result draw_coverage(cui_ctx *ui, int flags) {
    const cui_coverage_file files[] = {
        { "a.c", 100, 20, 10, 2, 4, 1 }, { "b.c", 100, 90, 10, 9, 4, 4 }
    };
    fpas_dev_result result;
    cui_begin_frame(ui, .016f);
    fpas_cui_coverage(ui, "coverage", files, 2, -1, 30, 85, 1, 0, flags, &result);
    cui_end_frame(ui);
    return result;
}

/** Keeps the selection on the same file after a coverage column reorders the rows. */
static void coverage_sort_preserves_selection(void) {
    cui_ctx *ui = create();
    if (!ui) { check(false, "Could not open coverage context."); return; }
    draw_coverage(ui, 1);
    const fpas_dev_result initial = draw_coverage(ui, 1);
    check(initial.x == 55 && initial.action == -1, "Coverage must return the aggregate before activation.");
    const cui_rect report = cui_get_rect(ui, cui_make_id(ui, "coverage"));
    check(report.h > 500, "A negative coverage height must fill the available height.");

    cui_input_mouse_move(ui, report.x + 60, report.y + 76);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    draw_coverage(ui, 1);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    draw_coverage(ui, 1);
    cui_input_key(ui, CUI_KEY_ENTER, true, 0);
    check(draw_coverage(ui, 1).action == 0, "Enter must activate the selected first file.");
    cui_input_key(ui, CUI_KEY_ENTER, false, 0);

    cui_push_id(ui, "coverage");
    const cui_rect lines = cui_get_rect(ui, cui_make_id(ui, "Lines"));
    cui_pop_id(ui);
    check(lines.w > 0 && lines.h > 0, "The wide coverage panel must display the Lines header.");
    cui_input_mouse_move(ui, lines.x + lines.w * .5f, lines.y + lines.h * .5f);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    draw_coverage(ui, 1);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    draw_coverage(ui, 1);
    draw_coverage(ui, 3);
    cui_input_key(ui, CUI_KEY_ENTER, true, 0);
    check(draw_coverage(ui, 1).action == 0, "Enter after sorting must activate the same file.");
    cui_input_key(ui, CUI_KEY_ENTER, false, 0);
    cui_input_key(ui, CUI_KEY_UP, true, 0);
    draw_coverage(ui, 1);
    cui_input_key(ui, CUI_KEY_UP, false, 0);
    cui_input_key(ui, CUI_KEY_ENTER, true, 0);
    check(draw_coverage(ui, 1).action == 1, "Sorting by lines must place the highest coverage first.");
    cui_input_key(ui, CUI_KEY_ENTER, false, 0);
    draw_coverage(ui, 1);
    cui_input_key(ui, CUI_KEY_ENTER, true, 0);
    check(draw_coverage(ui, 0).action == -1, "Disabled coverage must suppress keyboard activation.");
    cui_destroy(ui);
}

/** Draws two status histories and returns one hover result. */
static fpas_dev_result draw_uptime(cui_ctx *ui, const cui_uptime_service *services, bool enabled) {
    fpas_dev_result result;
    cui_begin_frame(ui, .016f);
    fpas_cui_uptime(ui, "status", services, 2, 0, 0, 0, 1, 0, enabled, &result);
    cui_end_frame(ui);
    return result;
}

/** Finds the bars for a service in the status widget. */
static cui_rect uptime_bars(cui_ctx *ui, int service) {
    cui_push_id(ui, "status");
    cui_push_id_int(ui, service);
    const cui_rect bars = cui_get_rect(ui, cui_make_id(ui, "bars"));
    cui_pop_id(ui);
    cui_pop_id(ui);
    return bars;
}

/** Reports the service and full-history day indices across width changes. */
static void uptime_hover(void) {
    cui_ctx *ui = create();
    if (!ui) { check(false, "Could not open uptime context."); return; }
    float days[90];
    for (int i = 0; i < 90; ++i) days[i] = i < 2 ? -1 : 1;
    const cui_uptime_service services[] = {{ "API", days, 90 }, { "Website", days, 90 }};
    cui_set_viewport(ui, 340, 550, 1);
    draw_uptime(ui, services, true);
    draw_uptime(ui, services, true);
    const cui_rect narrow = uptime_bars(ui, 1);
    cui_input_mouse_move(ui, narrow.x + narrow.w - 1, narrow.y + 10);
    const fpas_dev_result latest = draw_uptime(ui, services, true);
    check(latest.index == 1 && latest.action == 89, "Narrow uptime must report the latest original day.");
    const fpas_dev_result disabled = draw_uptime(ui, services, false);
    check(disabled.index == -1 && disabled.action == -1, "Disabled uptime must suppress hover.");

    cui_set_viewport(ui, 700, 550, 1);
    draw_uptime(ui, services, true);
    draw_uptime(ui, services, true);
    const cui_rect wide = uptime_bars(ui, 0);
    cui_input_mouse_move(ui, wide.x + 1, wide.y + 10);
    const fpas_dev_result first = draw_uptime(ui, services, true);
    check(first.index == 0 && first.action == 0, "Wide uptime must report the first original day.");
    cui_destroy(ui);
}

/** Checks bucket hover and controlled selection. */
static void histogram(void) {
    cui_ctx *ui = create();
    if (!ui) { check(false, "Could not open histogram context."); return; }
    const float counts[] = { 1, 3, 2, 4 };
    const cui_histogram_series series = { "Events", counts, CUI_TONE_INFO };
    double start = 0, stop = 0;
    for (int frame = 0; frame < 8; ++frame) {
        if (frame == 1) cui_input_mouse_move(ui, 160, 38);
        if (frame == 2 || frame == 5) {
            cui_input_mouse_move(ui, 160, 38);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        }
        if (frame == 3) cui_input_mouse_move(ui, 600, 38);
        if (frame == 4 || frame == 6) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        cui_begin_frame(ui, 0.016f);
        fpas_dev_result result;
        fpas_cui_time_histogram(ui, "histogram", 100, 60, 4, &series, 1,
            start, stop, 150, 1, 1, 0, 1, &result);
        cui_end_frame(ui);
        if (frame == 1) check(result.hover == 0 && !result.flags,
            "Histogram hover must report its bucket without proposing a range.");
        if (frame == 3) check(result.flags && result.x == 100 && result.y == 340,
            "Histogram drag must select a range snapped to whole buckets.");
        if (frame == 6) check(result.flags && result.x == 0 && result.y == 0,
            "Clicking the histogram must clear its range.");
        start = result.x;
        stop = result.y;
    }
    cui_destroy(ui);
}

/** Cancels a histogram selection under a disabled ancestor. */
static void histogram_disable_drag(void) {
    cui_ctx *ui = create();
    if (!ui) { check(false, "Could not open histogram context."); return; }
    const float counts[] = { 1, 3, 2, 4 };
    const cui_histogram_series series[] = {
        { "Events", counts, CUI_TONE_INFO }, { "Warnings", NULL, CUI_TONE_WARNING }
    };
    for (int frame = 0; frame < 7; ++frame) {
        if (frame == 2) {
            cui_input_mouse_move(ui, 160, 38);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        }
        if (frame == 3) cui_input_mouse_move(ui, 350, 38);
        if (frame == 4 || frame == 5) cui_input_mouse_move(ui, 600, 38);
        if (frame == 6) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        cui_begin_frame(ui, 0.016f);
        fpas_dev_result result;
        cui_push_disabled(ui, frame == 4);
        fpas_cui_time_histogram(ui, "histogram", 100, 60, 4, series, 2,
            0, 0, 150, 1, 1, 0, 1, &result);
        cui_pop_disabled(ui);
        const int node = fpas_cui_last_node(ui);
        cui_end_frame(ui);
        if (!frame) check(fpas_cui_node_rect(ui, node).h > 150,
            "The histogram rectangle must include the series legend below its 150-point plot.");
        if (frame == 3) {
            check(result.flags, "The histogram drag must begin while enabled.");
            check(fpas_cui_hovered(ui, node), "The tracked histogram must report hover over its plot.");
        }
        if (frame >= 4) check(!result.flags && result.x == 0 && result.y == 0,
            "Disabling the histogram must cancel its drag, including after re-enabling it.");
    }
    cui_destroy(ui);
}

static void query_builder_disabled(void) {
    cui_ctx *ui = create();
    if (!ui) { check(false, "Could not open query builder context."); return; }
    const char *choices[] = { "Open", "Closed" };
    const cui_query_field field = { .name = "Status", .type = CUI_QUERY_CHOICE,
        .choices = choices, .choice_count = 2 };
    cui_query_node empty[2] = {0}, rules[2] = {{ .group = true }, { .depth = 1 }};
    int empty_count = 0, count = 2;
    fpas_dev_result result;
    cui_begin_frame(ui, 0.016f);
    cui_push_disabled(ui, true);
    fpas_cui_query_builder(ui, "empty", &field, 1, empty, &empty_count, 2, 3, 0, 1, 0, 1, &result);
    check(!result.flags && empty_count == 0,
        "An empty query must stay empty while its enclosing group is disabled.");
    cui_pop_disabled(ui);
    fpas_cui_query_builder(ui, "read-only", &field, 1, rules, &count, 2, 3, 1, 1, 0, 1, &result);
    check(!result.flags && count == 2 && rules[1].value[0] == '\0',
        "A read-only query must preserve an empty choice value.");
    cui_end_frame(ui);

    cui_begin_frame(ui, 0.016f);
    fpas_cui_query_builder(ui, "empty", &field, 1, empty, &empty_count, 2, 3, 0, 1, 0, 1, &result);
    check(result.flags && empty_count == 1 && empty[0].group,
        "An editable query may initialize its root group.");
    cui_end_frame(ui);
    cui_destroy(ui);
}

/** Draws a key-value row and blank placeholders in a disabled scope. */
static void draw_disabled_key_value(cui_ctx *ui, bool adapter) {
    cui_kv_pair pairs[2] = { { .key = "Name", .value = "Example", .enabled = true } };
    int count = 1;
    cui_begin_frame(ui, .05f);
    if (adapter) {
        fpas_dev_result result;
        fpas_cui_key_value_editor(ui, "kv", pairs, &count, 2, "", NULL, 0, 0, 0, 2, 430, 0, &result);
        check(!result.flags, "A disabled key-value editor must not send edits.");
    } else {
        cui_push_disabled(ui, true);
        cui_kv_editor(ui, "kv", pairs, &count, 2, .key_placeholder = "", .width = CUI_PX(430));
        cui_pop_disabled(ui);
    }
    cui_end_frame(ui);
    check(count == 1, "The disabled key-value editor must preserve the number of rows.");
}

/** Checks that disabled retains the native editable shape and supports an empty caption. */
static void disabled_key_value_parity(void) {
    cui_ctx *raw = create(), *bridge = create();
    for (int i = 0; i < 3; ++i) {
        draw_disabled_key_value(raw, false);
        draw_disabled_key_value(bridge, true);
    }
    const cui_draw_data *direct = cui_get_draw_data(raw), *adapted = cui_get_draw_data(bridge);
    check(direct->vertex_count > 0 && direct->vertex_count == adapted->vertex_count &&
        memcmp(direct->vertices, adapted->vertices, (size_t)direct->vertex_count * sizeof *direct->vertices) == 0,
        "A disabled key-value editor must match the native disabled editable layout.");
    cui_destroy(bridge);
    cui_destroy(raw);
}

/** Switches from a legal short expression to a weekly preset using the keyboard. */
static void schedule_capacity(bool short_buffer) {
    cui_ctx *ui = create();
    char value[24] = "59 23 * * *";
    const size_t capacity = short_buffer ? 12 : sizeof value;
    const cui_key sequence[] = { CUI_KEY_NONE, CUI_KEY_TAB, CUI_KEY_RIGHT, CUI_KEY_RIGHT, CUI_KEY_RIGHT };
    fpas_dev_result result;
    double daily_next = 0;
    for (int i = 0; i < 5; ++i) {
        if (sequence[i] != CUI_KEY_NONE) {
            cui_input_key(ui, sequence[i], true, 0);
            cui_input_key(ui, sequence[i], false, 0);
        }
        cui_begin_frame(ui, .05f);
        fpas_cui_schedule_picker(ui, "schedule", value, capacity, 1794211200, 1, 0, 1, 0, 1, &result);
        cui_end_frame(ui);
        if (i == 3) daily_next = result.x;
    }
    if (short_buffer) {
        check(strcmp(value, "59 23 * * *") == 0 && result.flags == 2 && result.x == daily_next,
            "A preset that exceeds capacity must preserve the current model expression and next run.");
        check(cui_needs_redraw(ui), "A rejected schedule change must repaint the model expression.");
    } else {
        check(strcmp(value, "59 23 * * 1,2,3,4,5") == 0 && result.flags == 3 && result.x > 0,
            "A fitting schedule preset must preserve every weekday and report one valid edit.");
    }
    cui_destroy(ui);
}

static bool spinner_stops(bool adapter) {
    cui_ctx *ui = create();
    if (!ui) return false;
    const char *stages[] = { "Build" };
    const cui_pipeline_job job = { "Windows", 0, CUI_JOB_RUNNING, 1.0f, false };
    const cui_test_item tests[] = { { "Suite", 0 }, { "Layout", 1, CUI_JOB_RUNNING, -1 } };
    for (int frame = 0; frame < 14; ++frame) {
        cui_begin_frame(ui, 0.05f);
        if (adapter) {
            fpas_dev_result result;
            fpas_cui_pipeline(ui, "pipeline", stages, 1, &job, 1, -1, 1, 0, 1, &result);
            fpas_cui_test_explorer(ui, "tests", tests, 2, 220, 1, 0, 1, &result);
        } else {
            cui_pipeline(ui, "pipeline", stages, 1, &job, 1);
            cui_test_explorer(ui, "tests", tests, 2, .height = 220);
        }
        cui_end_frame(ui);
    }
    const bool stopped = !cui_needs_redraw(ui);
    cui_destroy(ui);
    return stopped;
}

/** Runs the developer tool checks or the selected focused fixtures. */
int main(int argc, char **argv) {
    if (argc > 1 && strcmp(argv[1], "reports") == 0) {
        coverage_sort_preserves_selection();
        uptime_hover();
        histogram();
        histogram_disable_drag();
        return failures != 0;
    }
    if (argc > 1 && argv[1][0] == 'm') {
        trace(false);
        return failures != 0;
    }
    if (argc > 1) return spinner_stops(false) ? 0 : 1;
    trace(true);
    trace_disables_pending_input();
    waterfall_details_and_focus(false);
    waterfall_details_and_focus(true);
    regex();
    developer_text_replacement(REQUEST_URL, 2);
    developer_text_replacement(REGEX_PATTERN, 1);
    developer_text_replacement(REGEX_SAMPLE, 5);
    developer_placeholders();
    coverage_sort_preserves_selection();
    uptime_hover();
    histogram();
    histogram_disable_drag();
    query_builder_disabled();
    disabled_key_value_parity();
    schedule_capacity(true);
    schedule_capacity(false);
    check(spinner_stops(true), "Running jobs and tests must stop requesting frames under reduced motion.");
    return failures != 0;
}
