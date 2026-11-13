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

static bool code(void) {
    cui_ctx *ui = context();
    char copied[128] = "";
    const char *source = "SELECT team FROM people;\n";
    const cui_platform_iface platform = { .set_clipboard = clipboard, .user = copied };
    cui_set_platform(ui, &platform);
    int copy = -1;
    for (int frame = 0; frame < 3; ++frame) {
        if (frame == 1) key(ui, CUI_KEY_ENTER);
        cui_begin_frame(ui, .016f);
        fpas_cui_code_block(ui, "code", source, "sql", "teams.sql", 0, 0, 0, 0,
            frame == 0 ? 12 : 4, &copy);
        cui_end_frame(ui);
    }
    const bool ok = copy >= 0 && fpas_cui_focused(ui, copy) && strcmp(copied, source) == 0;
    cui_destroy(ui);
    return ok;
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
    const bool copied = !adapter || code();
    const bool edited = !adapter || track();
    if (!copied || !edited) fprintf(stderr, "Code clipboard=%d, event track=%d\n", copied, edited);
    return parsed && hit && copied && edited ? 0 : 1;
}
