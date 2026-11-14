#include "../fpas_cui.h"
#include <stdio.h>

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
            fpas_cui_trace_viewer(ui, "trace", &track, 1, spans, 2, NULL, 0,
                view.start, view.end, view.scroll, view.mark_start, view.mark_end,
                -1, 180, 0, 1, 0, 1, &result);
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

static void histogram(void) {
    cui_ctx *ui = create();
    if (!ui) { check(false, "Could not open histogram context."); return; }
    const float counts[] = { 1, 3, 2, 4 };
    const cui_histogram_series series = { "Events", counts, CUI_TONE_INFO };
    double start = 0, stop = 0;
    for (int frame = 0; frame < 8; ++frame) {
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
        if (frame == 3) check(result.flags && result.x == 100 && result.y == 340,
            "Histogram drag must select a range snapped to whole buckets.");
        if (frame == 6) check(result.flags && result.x == 0 && result.y == 0,
            "Clicking the histogram must clear its range.");
        start = result.x;
        stop = result.y;
    }
    cui_destroy(ui);
}

static void histogram_disable_drag(void) {
    cui_ctx *ui = create();
    if (!ui) { check(false, "Could not open histogram context."); return; }
    const float counts[] = { 1, 3, 2, 4 };
    const cui_histogram_series series[] = {
        { "Events", counts, CUI_TONE_INFO }, { "Warnings", counts, CUI_TONE_WARNING }
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

int main(int argc, char **argv) {
    if (argc > 1 && argv[1][0] == 'm') {
        trace(false);
        return failures != 0;
    }
    if (argc > 1) return spinner_stops(false) ? 0 : 1;
    trace(true);
    regex();
    histogram();
    histogram_disable_drag();
    query_builder_disabled();
    check(spinner_stops(true), "Running jobs and tests must stop requesting frames under reduced motion.");
    return failures != 0;
}
