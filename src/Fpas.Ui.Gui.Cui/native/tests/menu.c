/* Compare the adapter with the native gallery's dropdown primitives. */
#include "../fpas_cui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static bool parity(bool icon_only, int placement, float width, bool contextual) {
    const int w = 640, h = 520;
    const size_t bytes = (size_t)w * h * 4;
    unsigned char *expected = malloc(bytes), *actual = malloc(bytes);
    if (!expected || !actual) { free(expected); free(actual); return false; }
    bool valid = true;
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        if (!renderer) { valid = false; break; }
        const cui_ctx_desc settings = { .renderer = cui_sw_iface(renderer) };
        cui_ctx *ui = cui_create(&settings);
        if (!ui) { cui_sw_destroy(renderer); valid = false; break; }
        cui_set_viewport(ui, w, h, 1);
        cui_set_reduced_motion(ui, true);
        const char *keys[] = { "", "rename", "", "delete", "disabled" };
        const char *labels[] = { "Project", "Rename", "", "Delete", "Disabled" };
        const char *icons[] = { "", CUI_ICON_PENCIL, "", CUI_ICON_TRASH_2, "" };
        const char *shortcuts[] = { "", "F2", "", "", "" };
        const int flags[] = { 1, 16, 2, 8, 4 };
        bool opened = false;
        for (int frame = 0; frame < 6; ++frame) {
            cui_begin_frame(ui, 0.04f);
            cui_input_mouse_move(ui, frame < 4 ? 40 : -1, frame < 4 ? 40 : -1);
            if (frame == 2) cui_input_mouse_button(ui, contextual ? CUI_MOUSE_RIGHT : CUI_MOUSE_LEFT, true);
            if (frame == 3) cui_input_mouse_button(ui, contextual ? CUI_MOUSE_RIGHT : CUI_MOUSE_LEFT, false);
            cui_column(ui, .width = CUI_FILL, .pad = 24) {
                if (contextual) {
                    cui_card(ui, .id = "target", .muted = true, .width = CUI_PX(220), .height = CUI_PX(100), .hoverable = true) { }
                }
                if (pass) {
                    int nodes[6], open, highlighted;
                    unsigned int owner;
                    fpas_cui_menu(ui, "fixture", "Options", CUI_ICON_SETTINGS, keys, labels,
                        icons, shortcuts, flags, 5, 0, placement, width, icon_only, 1, 0,
                        contextual, nodes, &open, &highlighted, &owner);
                    opened = open != 0;
                } else {
                    const cui_menu_opts options = { .icon = CUI_ICON_SETTINGS, .icon_only = icon_only,
                        .placement = (cui_placement)placement, .width = width };
                    opened = contextual ? cui_context_menu_begin(ui, "fixture") : cui_menu_begin(ui, "Options", &options);
                    if (opened) {
                        cui_menu_label(ui, "Project");
                        cui_menu_item(ui, "Rename", .icon = CUI_ICON_PENCIL, .shortcut = "F2", .checked = true);
                        cui_menu_separator(ui);
                        cui_menu_item(ui, "Delete", .icon = CUI_ICON_TRASH_2, .danger = true);
                        cui_menu_item(ui, "Disabled", .disabled = true);
                        if (contextual) cui_context_menu_end(ui); else cui_menu_end(ui);
                    }
                }
            }
            cui_end_frame(ui);
            cui_sw_render(renderer, cui_get_draw_data(ui), pass ? actual : expected, w, h, w * 4, CUI_SW_RGBA);
        }
        valid = valid && opened;
        cui_destroy(ui); cui_sw_destroy(renderer);
    }
    valid = valid && memcmp(expected, actual, bytes) == 0;
    free(expected); free(actual);
    return valid;
}

int main(void) {
    bool valid = true;
    for (int icon = 0; icon < 2; ++icon)
        for (int place = 0; place < 2; ++place)
            for (int sized = 0; sized < 2; ++sized)
                valid = parity(icon != 0, place ? CUI_PLACE_TOP_END : CUI_PLACE_BOTTOM_START, sized ? 260 : 0, false) && valid;
    valid = parity(false, CUI_PLACE_BOTTOM_START, 0, true) && valid;
    if (!valid) fprintf(stderr, "Menu adapter pixels differ from native CUI.\n");
    return valid ? 0 : 1;
}
