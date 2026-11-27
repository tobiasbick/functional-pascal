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

static bool chart(bool adapter) {
    cui_ctx *ui = context();
    const float samples[] = { 0, 20, 40 };
    const int accent = 0;
    const fpas_dot_options opts = { .x_min = 6, .x_max = 30, .flags = 4 };
    const cui_dot_series series = { samples, NULL, false };
    int index = -1, clicked = 0;
    double x = 0;
    float value = 0;
    for (int frame = 0; frame < 2; ++frame) {
        if (frame == 1) key(ui, CUI_KEY_ENTER);
        cui_begin_frame(ui, .016f);
        if (frame == 0) fpas_cui_request_focus(ui, "chart");
        if (adapter)
            clicked = fpas_cui_dot_chart(ui, "chart", samples, &accent, 1, 3, &opts, &index, &x, &value);
        else {
            const cui_dot_chart_result r = cui_dot_chart(ui, "chart", &series, 1, 3, .x_min = 6, .x_max = 30);
            clicked = r.clicked; index = r.hovered; x = r.x; value = r.value;
        }
        cui_end_frame(ui);
    }
    printf("%s chart Enter: index=%d x=%g value=%g\n", adapter ? "FPAS" : "CUI", index, x, value);
    cui_destroy(ui);
    return clicked && index == 1 && x == 18 && value == 20;
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
        if (frame == 1) ok &= count == 2 && times[1] == 12.5;
        if (frame == 2) ok &= count == 3 && times[2] == 12.75 && result[1] == 2;
        if (frame == 3) ok &= count == 2 && result[2] == 2;
    }
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
    const bool edited = !adapter || track();
    const bool viewed = !adapter || json_view_behavior();
    if (!copied || !edited) fprintf(stderr, "Code clipboard=%d, event track=%d\n", copied, edited);
    return parsed && hit && copied && edited && viewed && scrolled ? 0 : 1;
}
