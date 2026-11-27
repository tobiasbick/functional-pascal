// [Documentation](../../../../docs/ui/gui/instruments/README.md)
/* Focused native clipboard, parsing and instrument interaction checks. */
#include "../fpas_cui.h"
#include <stdio.h>
#include <string.h>

static void key(cui_ctx *ui, cui_key value) {
    cui_input_key(ui, value, true, 0);
    cui_input_key(ui, value, false, 0);
}

static cui_ctx *context(void) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 420, 1);
    cui_set_reduced_motion(ui, true);
    return ui;
}

static void clipboard(void *user, const char *text) { snprintf(user, 128, "%s", text); }

static bool code(const char *language, const char *title) {
    cui_ctx *ui = context();
    char copied[128] = "";
    const char *source = "\t  SELECT team FROM people;\n";
    const cui_platform_iface platform = { .set_clipboard = clipboard, .user = copied };
    cui_set_platform(ui, &platform);
    int copy = -1;
    for (int frame = 0; frame < 3; ++frame) {
        if (frame == 1) key(ui, CUI_KEY_ENTER);
        cui_begin_frame(ui, .016f);
        fpas_cui_code_block(ui, "code", source, language, title, 0, 0, 0, 0,
            frame == 0 ? 12 : 4, &copy);
        cui_end_frame(ui);
    }
    const bool ok = language || title
        ? copy >= 0 && fpas_cui_focused(ui, copy) && strcmp(copied, source) == 0
        : copy < 0 && copied[0] == '\0';
    cui_destroy(ui);
    return ok;
}

enum { CODE_WIDTH = 400, CODE_HEIGHT = 180 };

static void draw_code(const char *source, const char *language, unsigned char *pixels) {
    cui_sw_renderer *renderer = cui_sw_create();
    cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(renderer) });
    cui_set_viewport(ui, CODE_WIDTH, CODE_HEIGHT, 1);
    cui_set_reduced_motion(ui, true);
    cui_begin_frame(ui, .016f);
    int copy;
    fpas_cui_code_block(ui, "code", source, language, NULL, 1, 2, 60, 14, 7, &copy);
    cui_end_frame(ui);
    cui_sw_render(renderer, cui_get_draw_data(ui), pixels,
        CODE_WIDTH, CODE_HEIGHT, CODE_WIDTH * 4, CUI_SW_RGBA);
    cui_destroy(ui);
    cui_sw_destroy(renderer);
}

static bool code_pixels(void) {
    static unsigned char first[CODE_WIDTH * CODE_HEIGHT * 4], second[CODE_WIDTH * CODE_HEIGHT * 4];
    draw_code("    Alpha\nMore\n", "text", first);
    draw_code("    Zebra\nMore\n", "plain", second);
    const bool visible = memcmp(first, second, sizeof first) != 0;
    draw_code("    Alpha\nMore\n", "plain", second);
    const bool alias = memcmp(first, second, sizeof first) == 0;
    draw_code("\t  return 1;\n", "c", first);
    draw_code("      return 1;\n", "h", second);
    const bool tabs = memcmp(first, second, sizeof first) == 0;
    if (!visible || !alias || !tabs)
        fprintf(stderr, "Code block pixels differ: plain=%d aliases=%d tabs=%d\n", visible, alias, tabs);
    return visible && alias && tabs;
}

static void code_scroll_frame(cui_ctx *ui, cui_sw_renderer *renderer, const char *source,
    bool enabled, unsigned char *pixels) {
    int copy;
    cui_begin_frame(ui, .016f);
    fpas_cui_code_block(ui, "code", source, NULL, NULL, 0, 0, 80, 0, enabled ? 5 : 1, &copy);
    cui_end_frame(ui);
    if (pixels) cui_sw_render(renderer, cui_get_draw_data(ui), pixels,
        CODE_WIDTH, CODE_HEIGHT, CODE_WIDTH * 4, CUI_SW_RGBA);
}

static bool code_scroll_behavior(void) {
    static unsigned char before[CODE_WIDTH * CODE_HEIGHT * 4], after[CODE_WIDTH * CODE_HEIGHT * 4];
    char large[4096] = "";
    for (int row = 0; row < 34; ++row)
        strcat(large, "A long example line for the horizontal and vertical source viewport.\n");
    cui_sw_renderer *renderer = cui_sw_create();
    cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(renderer) });
    cui_set_viewport(ui, CODE_WIDTH, CODE_HEIGHT, 1);
    cui_set_reduced_motion(ui, true);
    code_scroll_frame(ui, renderer, large, false, NULL);
    code_scroll_frame(ui, renderer, large, false, before);
    cui_input_mouse_move(ui, 90, 45);
    cui_input_scroll(ui, -4, -12);
    code_scroll_frame(ui, renderer, large, false, after);
    const bool blocked = memcmp(before, after, sizeof before) == 0;

    code_scroll_frame(ui, renderer, large, true, NULL);
    cui_input_scroll(ui, -4, -12);
    code_scroll_frame(ui, renderer, large, true, after);
    const bool scrolled = memcmp(before, after, sizeof before) != 0;
    code_scroll_frame(ui, renderer, large, true, NULL);
    code_scroll_frame(ui, renderer, large, true, NULL);
    code_scroll_frame(ui, renderer, "Short replacement", true, before);
    const bool refreshed = cui_needs_redraw(ui);
    code_scroll_frame(ui, renderer, "Short replacement", true, after);
    const bool clamped = memcmp(before, after, sizeof before) != 0 && !cui_needs_redraw(ui);
    cui_destroy(ui);
    cui_sw_destroy(renderer);
    if (!blocked || !scrolled || !refreshed || !clamped)
        fprintf(stderr, "Code scroll behavior: blocked=%d scrolled=%d refreshed=%d clamped=%d\n",
            blocked, scrolled, refreshed, clamped);
    return blocked && scrolled && refreshed && clamped;
}

static bool json(bool adapter) {
    cui_ctx *ui = context();
    const char *invalid[] = { "1e", "-", "01", "\"\\q\"", "\"\\u12x4\"" };
    bool ok = true;
    cui_begin_frame(ui, .016f);
    for (size_t i = 0; i < sizeof invalid / sizeof *invalid; ++i) {
        const bool valid = adapter ? fpas_cui_json_view(ui, "json", invalid[i], 2, 0, 1)
            : cui_json_view(ui, "json", invalid[i]);
        if (valid) fprintf(stderr, "%s accepted malformed JSON: %s\n", adapter ? "FPAS" : "CUI", invalid[i]);
        ok &= !valid;
    }
    ok &= adapter ? fpas_cui_json_view(ui, "valid", "[-2.5e+3,\"\\u00e9\"]", 2, 0, 1)
        : cui_json_view(ui, "valid", "[-2.5e+3,\"\\u00e9\"]");
    cui_end_frame(ui);
    cui_destroy(ui);
    return ok;
}

/** Captures the viewer bounds and the root row's relative scroll position. */
static cui_rect json_frame(cui_ctx *ui, const char *id, const char *source, int depth,
    float maximum, bool enabled, bool enclosing, float elapsed, float *row_y) {
    cui_begin_frame(ui, elapsed);
    if (enclosing) cui_scroll_begin(ui, &(cui_scroll_opts){ .id = "page", .height = CUI_PX(200) });
    fpas_cui_json_view(ui, id, source, depth, maximum, enabled);
    const int outer = fpas_cui_last_node(ui);
    if (enclosing) { cui_gap(ui, 240); cui_scroll_end(ui); }
    cui_end_frame(ui);
    const cui_rect box = fpas_cui_node_rect(ui, outer);
    *row_y = fpas_cui_node_rect(ui, outer + 2).y - box.y;
    return box;
}

/** Opens the native document at its requested depth and retains user folding while mounted. */
static bool json_folding(const char *source) {
    cui_ctx *ui = context();
    float row;
    const cui_rect shallow = json_frame(ui, "shallow", source, 1, 0, true, false, .016f, &row);
    const cui_rect regular = json_frame(ui, "json", source, 0, 0, true, false, .016f, &row);
    const cui_rect deep = json_frame(ui, "deep", source, 2, 0, true, false, .016f, &row);
    bool ok = regular.h > shallow.h && regular.h == deep.h;
    json_frame(ui, "json", source, 0, 0, true, false, .016f, &row);
    cui_input_mouse_move(ui, regular.x + 24, regular.y + 44);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    json_frame(ui, "json", source, 0, 0, true, false, .016f, &row);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    const cui_rect folded = json_frame(ui, "json", source, 0, 0, true, false, .016f, &row);
    cui_set_viewport(ui, 320, 420, 1);
    const cui_rect narrowed = json_frame(ui, "json", source, 3, 0, true, false, .016f, &row);
    ok &= folded.h == shallow.h && narrowed.h == folded.h && narrowed.w < regular.w;
    cui_destroy(ui);
    return ok;
}

/** Rejects input while disabled and refreshes an idle viewport when its document shrinks. */
static bool json_view_behavior(void) {
    const char *source = "{\"values\":[1,2,3,4,5,6,7,8,9,10,11,12],\"label\":\"example\"}";
    bool ok = json_folding(source);
    cui_ctx *ui = context();
    float row;
    const cui_rect capped = json_frame(ui, "json", source, 0, 100, true, false, .016f, &row);
    const float initial_row = row;
    cui_input_mouse_move(ui, capped.x + 30, capped.y + 50);
    cui_input_scroll(ui, 0, -5);
    json_frame(ui, "json", source, 0, 100, true, false, .016f, &row);
    json_frame(ui, "json", source, 0, 100, true, false, 1, &row);
    json_frame(ui, "json", source, 0, 100, true, false, 1, &row);
    ok &= capped.h == 100 && row < 0 && !cui_needs_redraw(ui);
    const char *shorter = "{\"label\":\"example\"}";
    json_frame(ui, "json", shorter, 0, 100, true, false, .016f, &row);
    ok &= cui_needs_redraw(ui);
    json_frame(ui, "json", shorter, 0, 100, true, false, .016f, &row);
    ok &= row == initial_row;
    cui_destroy(ui);

    ui = context();
    const cui_rect before = json_frame(ui, "json", source, 0, 100, true, true, .016f, &row);
    cui_input_mouse_move(ui, before.x + 30, before.y + 50);
    cui_input_scroll(ui, 0, -1);
    json_frame(ui, "json", source, 0, 100, false, true, .016f, &row);
    ok &= row == initial_row;
    cui_input_scroll(ui, 0, -1);
    const cui_rect after = json_frame(ui, "json", source, 0, 100, false, true, .016f, &row);
    ok &= row == initial_row && after.y < before.y;
    cui_destroy(ui);
    if (!ok) fprintf(stderr, "JSON fold state, viewport size, disabled input or scroll recovery mismatch.\n");
    return ok;
}

static cui_dot_chart_result chart_frame(cui_ctx *ui, bool adapter, bool focus) {
    const float samples[] = { 0, 20, 40, 1, 2, 3 };
    const int accent[] = { 0, 1 };
    const char *names[] = { "First", "Second" };
    const fpas_chart_annotation mark = { .kind = CUI_ANNOTATION_X, .x = 12, .label = "Marker" };
    const fpas_dot_options opts = { .chart = { .annotations = &mark, .annotation_count = 1 },
        .x_min = 6, .x_max = 30, .flags = 4, .names = names };
    const cui_dot_series series[] = { { samples, names[0], false }, { samples + 3, names[1], true } };
    const cui_chart_annotation annotation = { .kind = CUI_ANNOTATION_X, .x = 12, .label = "Marker" };
    cui_begin_frame(ui, .016f);
    if (focus) fpas_cui_request_focus(ui, "chart");
    cui_dot_chart_result result;
    if (adapter) {
        result.clicked = fpas_cui_dot_chart(ui, "chart", samples, accent, 2, 3, &opts,
            &result.hovered, &result.x, &result.value) != 0;
    } else {
        result = cui_dot_chart(ui, "chart", series, 2, 3, .x_min = 6, .x_max = 30,
            .annotations = &annotation, .annotation_count = 1);
    }
    cui_end_frame(ui);
    return result;
}

static cui_dot_chart_result chart_click(cui_ctx *ui, bool adapter, float x, float y) {
    cui_input_mouse_move(ui, x, y);
    chart_frame(ui, adapter, false);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    chart_frame(ui, adapter, false);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    return chart_frame(ui, adapter, false);
}

static bool chart(bool adapter) {
    cui_ctx *ui = context();
    cui_input_mouse_leave(ui);
    chart_frame(ui, adapter, true);
    key(ui, CUI_KEY_ENTER);
    cui_dot_chart_result result = chart_frame(ui, adapter, false);
    bool ok = result.clicked && result.hovered == 1 && result.x == 18 && result.value == 22;
    printf("%s chart Enter: index=%d x=%g value=%g\n",
        adapter ? "FPAS" : "CUI", result.hovered, result.x, result.value);

    const cui_rect rect = cui_get_rect(ui, cui_make_id(ui, "chart"));
    const float quarter = rect.x + (rect.w - 34) * .25f;
    result = chart_click(ui, adapter, quarter, rect.y + 75);
    ok &= result.clicked && result.hovered == 1 && result.x == 12 && result.value == 22;
    result = chart_click(ui, adapter, rect.x + rect.w - 10, rect.y + 75);
    ok &= result.clicked == !adapter && result.hovered == -1 && result.x == (adapter ? 0 : 30);
    result = chart_click(ui, adapter, quarter, rect.y + rect.h - 9);
    ok &= result.clicked == !adapter && result.hovered == -1 && result.x == (adapter ? 0 : 12);
    result = chart_click(ui, adapter, quarter, rect.y + 2);
    ok &= result.clicked == !adapter && result.hovered == -1 && result.x == (adapter ? 0 : 12);
    key(ui, CUI_KEY_HOME);
    result = chart_frame(ui, adapter, false);
    ok &= !result.clicked && result.hovered == 0 && result.x == 6 && result.value == 1;
    key(ui, CUI_KEY_SPACE);
    result = chart_frame(ui, adapter, false);
    ok &= result.clicked && result.hovered == 0 && result.x == 6 && result.value == 1;
    key(ui, CUI_KEY_ESCAPE);
    result = chart_frame(ui, adapter, false);
    ok &= !result.clicked && result.hovered == -1 && result.x == 0 && result.value == 0;
    if (!ok) fprintf(stderr, "Dot chart pointer activation or keyboard readout mismatch.\n");
    cui_destroy(ui);
    return ok;
}

static bool track(void) {
    cui_ctx *ui = context();
    double times[4] = { 8, 12.25 };
    int count = 2, result[5] = { 1 };
    const fpas_event_options opts = { .minimum = 6, .maximum = 30, .step = .25, .units = CUI_AXIS_HOURS, .flags = 8 };
    const cui_key keys[] = { CUI_KEY_NONE, CUI_KEY_RIGHT, CUI_KEY_ENTER, CUI_KEY_DELETE };
    bool ok = true;
    for (int frame = 0; frame < 4; ++frame) {
        if (frame) key(ui, keys[frame]);
        cui_begin_frame(ui, .016f);
        if (frame == 0) fpas_cui_request_focus(ui, "track");
        fpas_cui_event_track(ui, "track", times, &count, 4, &opts, result);
        cui_end_frame(ui);
        if (frame == 1) ok &= count == 2 && times[1] == 12.5 && result[4] == 1;
        if (frame == 2) ok &= count == 3 && times[2] == 12.75 && result[1] == 2 && result[4] == 3;
        if (frame == 3) ok &= count == 2 && result[2] == 2 && result[4] == 3;
    }
    cui_destroy(ui);
    return ok;
}

static void track_frame(cui_ctx *ui, double *times, int *count, int *result, bool popup) {
    const fpas_event_options options = { .minimum = 0, .maximum = 24, .step = 1, .flags = 8 | 32 };
    result[0] = 99;
    cui_begin_frame(ui, .016f);
    fpas_cui_event_track(ui, "track", times, count, 4, &options, result);
    if (popup) {
        cui_button(ui, "Anchor");
        cui_popover(ui, "Popup") cui_text(ui, "Popup content");
    }
    cui_end_frame(ui);
}

static bool track_drag_and_selection(void) {
    cui_ctx *ui = context();
    double times[4] = { 6, 12 };
    int count = 2, result[5];
    track_frame(ui, times, &count, result, false);
    bool ok = result[0] == -1 && result[4] == 0;
    const cui_rect track = cui_get_rect(ui, cui_make_id(ui, "track"));
    const float unit = (track.w - 34) / 24, y = track.y + 12;
    const float first = track.x + 6 * unit;
    cui_input_mouse_move(ui, first, y);
    track_frame(ui, times, &count, result, false);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    track_frame(ui, times, &count, result, false);
    ok &= result[0] == 0 && !result[3] && result[4] == 2;
    track_frame(ui, times, &count, result, false);
    ok &= result[0] == 0 && result[4] == 0;
    cui_input_mouse_move(ui, first + 3, y);
    track_frame(ui, times, &count, result, false);
    ok &= times[0] == 6 && result[3] && result[4] == 4;
    cui_input_mouse_move(ui, track.x + 16 * unit, y);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    cui_input_mouse_move(ui, first, y);
    track_frame(ui, times, &count, result, false);
    ok &= count == 2 && times[0] == 12 && times[1] == 16 && result[0] == 1 && !result[3] && result[4] == 7;
    track_frame(ui, times, &count, result, false);
    ok &= result[0] == 1 && result[4] == 0;

    cui_input_mouse_move(ui, track.x + 12 * unit, y);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    cui_input_mouse_move(ui, track.x + 20 * unit, track.y + track.h + 10);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    track_frame(ui, times, &count, result, false);
    ok &= count == 2 && times[0] == 16 && times[1] == 20 && result[0] == 1 && !result[3] && result[4] == 1;

    const float last = track.x + 20 * unit;
    cui_input_mouse_move(ui, last, y);
    track_frame(ui, times, &count, result, false);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    track_frame(ui, times, &count, result, false);
    cui_input_mouse_move(ui, last + 3, y);
    track_frame(ui, times, &count, result, false);
    ok &= times[1] == 20 && result[3] && result[4] == 4;
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    track_frame(ui, times, &count, result, false);
    ok &= times[1] == 20 && !result[3] && result[4] == 4;

    key(ui, CUI_KEY_HOME);
    track_frame(ui, times, &count, result, false);
    ok &= result[0] == 0 && result[4] == 2;
    cui_popover_open(ui, "Popup");
    track_frame(ui, times, &count, result, true);
    const cui_key blocked[] = { CUI_KEY_RIGHT, CUI_KEY_ENTER, CUI_KEY_SPACE, CUI_KEY_DELETE };
    for (size_t i = 0; i < sizeof blocked / sizeof blocked[0]; ++i) {
        key(ui, blocked[i]);
        track_frame(ui, times, &count, result, true);
        ok &= count == 2 && times[0] == 16 && times[1] == 20 && result[0] == 0 && result[4] == 0;
    }
    ok &= cui_popover_is_open(ui, "Popup");
    cui_destroy(ui);
    return ok;
}

int main(int argc, char **argv) {
    const bool adapter = argc < 2 || strcmp(argv[1], "upstream") != 0;
    const bool parsed = json(adapter);
    const bool hit = chart(adapter);
    const bool copied = !adapter || (code(NULL, NULL) && code(NULL, "") && code("", NULL)
        && code("sql", "teams.sql") && code_pixels());
    const bool scrolled = !adapter || code_scroll_behavior();
    const bool edited = !adapter || (track() && track_drag_and_selection());
    const bool viewed = !adapter || json_view_behavior();
    if (!copied || !edited) fprintf(stderr, "Code clipboard=%d, event track=%d\n", copied, edited);
    return parsed && hit && copied && edited && viewed && scrolled ? 0 : 1;
}
