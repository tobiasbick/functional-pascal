// [Documentation](../../../../docs/ui/gui/code/code-editor.md)
#include "../fpas_cui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Compare native marks, programmatic selection/find and deferred paint with direct CUI. */
static bool matches(bool dark, int command) {
    const int width = 600, height = 400;
    const size_t bytes = (size_t)width * height * 4;
    unsigned char *pixels[2] = { calloc(1, bytes), calloc(1, bytes) };
    bool valid = pixels[0] && pixels[1];
    for (int pass = 0; pass < 2 && valid; pass++) {
        cui_sw_renderer *renderer = cui_sw_create();
        const cui_ctx_desc desc = { .renderer = cui_sw_iface(renderer) };
        cui_ctx *ui = cui_create(&desc);
        cui_set_viewport(ui, width, height, 1);
        cui_set_theme(ui, dark ? CUI_THEME_DARK : CUI_THEME_LIGHT);
        cui_set_reduced_motion(ui, true);
        char text[128] = "a\xf0\x9f\x98\x80\xc3\xa9\tend\nsecond\nthird";
        const cui_text_mark marks[2] = { { 1, 7, 0 }, { 11, 17, 3 } };
        for (int frame = 0; frame < 3; frame++) {
            cui_begin_frame(ui, .04f);
            if (frame == 1) {
                if (command == 0) cui_code_editor_select(ui, "editor", 1, 7);
                if (command == 1) cui_code_editor_goto(ui, "editor", 2, 2);
                if (command == 2) cui_code_editor_find(ui, "editor", "second");
            }
            if (pass) {
                fpas_code_editor_observation result;
                fpas_cui_code_editor(ui, "editor", text, sizeof text, "text", NULL,
                    1, 0, 340, 0, 4, 1, marks, 2, &result);
                if (frame == 1) valid = valid && (command == 0 ? result.cursor == 7 && result.anchor == 1 :
                    command == 1 ? result.line == 2 && result.column == 2 : result.matches == 1);
            } else {
                const cui_code_editor_opts options = { .lang = "text", .width = CUI_FILL, .height = 340,
                    .tab_size = 4, .marks = marks, .mark_count = 2 };
                cui_code_editor_ex(ui, "editor", text, sizeof text, &options);
            }
            cui_end_frame(ui);
            cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], width, height, width * 4, CUI_SW_RGBA);
        }
        cui_destroy(ui); cui_sw_destroy(renderer);
    }
    valid = valid && memcmp(pixels[0], pixels[1], bytes) == 0;
    if (!valid) fprintf(stderr, "Editor native parity failed: dark=%d command=%d\n", dark, command);
    free(pixels[0]); free(pixels[1]);
    return valid;
}

static bool retire_focused_inspector(bool hex) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 640, 300, 1);
    cui_begin_frame(ui, .04f);
    const cui_id key = cui_make_id(ui, "inspector");
    const int focus_enabled = 16 | 1;
    uint8_t data[] = {0, 1, 2, 3, 4, 5};
    if (hex) {
        int start = 0, length = 0, cursor = 0, edit_at, edit_length;
        fpas_cui_hex_view(ui, "inspector", data, sizeof data, &start, &length, &cursor,
            NULL, 0, 0, -1, 16, 1, 0, 220, 0, focus_enabled, &edit_at, &edit_length);
    } else {
        int selected, count, inline_mode;
        fpas_cui_diff_view(ui, "inspector", "one\ntwo\n", "one\nthree\n", -1,
            "text", NULL, NULL, 3, 1, 0, 220, 0, focus_enabled, &selected, &count, &inline_mode);
    }
    cui_end_frame(ui);
    const bool focused = fpas_cui_focus_key(ui, key) != 0;
    fpas_cui_forget(ui, key);
    const bool released = (fpas_cui_shortcut_owners(ui) & 4) == 0;
    cui_destroy(ui);
    if (!focused || !released) fprintf(stderr, "%s must release native focus when retired.\n", hex ? "Hex view" : "Diff view");
    return focused && released;
}

int main(void) {
    bool valid = retire_focused_inspector(true);
    valid = retire_focused_inspector(false) && valid;
    for (int dark = 0; dark < 2; dark++)
        for (int command = 0; command < 3; command++)
            valid = matches(dark != 0, command) && valid;
    return valid ? 0 : 1;
}
