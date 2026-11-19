// [Documentation](../../../../docs/ui/gui/layout/README.md)
/* Insets, RTL, stacks and spacing must paint exactly like direct CUI calls. */
#include "../fpas_cui.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

static bool matches(int axis, bool rtl, bool dark, int width) {
    const int height = 220;
    const size_t bytes = (size_t)width * height * 4;
    unsigned char *pixels[2] = { calloc(1, bytes), calloc(1, bytes) };
    if (!pixels[0] || !pixels[1]) { free(pixels[0]); free(pixels[1]); return false; }
    bool valid = true;
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        const cui_ctx_desc options = { .renderer = cui_sw_iface(renderer) };
        cui_ctx *ui = cui_create(&options);
        if (!ui) { cui_sw_destroy(renderer); valid = false; break; }
        cui_set_viewport(ui, width, height, 1);
        cui_set_reduced_motion(ui, true);
        cui_set_theme(ui, dark ? CUI_THEME_DARK : CUI_THEME_LIGHT);
        cui_begin_frame(ui, .04f);
        int node = 0;
        if (pass) {
            node = fpas_cui_group_begin_ex(ui, axis, 0, 7, CUI_ALIGN_START, 0, 0, 0, CUI_JUSTIFY_START);
            fpas_cui_box_details(ui, node, 9, 21, 15, 33, rtl ? CUI_DIRECTION_RTL : CUI_DIRECTION_LTR);
        } else {
            const cui_box box = { .dir = (cui_dir)axis, .width = CUI_FILL, .height = CUI_FILL,
                .pad_top = 9, .pad_right = 21, .pad_bottom = 15, .pad_left = 33, .gap = 7,
                .align = CUI_ALIGN_START, .direction = rtl ? CUI_DIRECTION_RTL : CUI_DIRECTION_LTR };
            cui_box_begin(ui, &box);
        }
        cui_skeleton(ui, CUI_PX(60), 20);
        cui_gap(ui, 10); cui_spacer(ui);
        cui_skeleton(ui, CUI_PX(40), 12);
        if (pass) {
            fpas_cui_group_end(ui, node);
            fpas_cui_node_size(ui, node, 1, 1, 0, 0, -1, 0, 0);
            fpas_cui_node_size(ui, node, 0, 1, 0, 0, -1, 0, 0);
        } else cui_box_end(ui);
        cui_end_frame(ui);
        cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], width, height, width * 4, CUI_SW_RGBA);
        cui_destroy(ui); cui_sw_destroy(renderer);
    }
    valid = valid && memcmp(pixels[0], pixels[1], bytes) == 0;
    free(pixels[0]); free(pixels[1]);
    if (!valid) fprintf(stderr, "Layout mismatch: axis=%d rtl=%d dark=%d width=%d\n", axis, rtl, dark, width);
    return valid;
}

static bool check_hover(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 280, 220, 1);

    cui_begin_frame(ui, .016f);
    const int group = fpas_cui_group_begin(ui, 0, 8, 0, CUI_ALIGN_START, 0, 0, 0);
    (void)cui_button(ui, "Continue");
    const int button = fpas_cui_last_node(ui);
    fpas_cui_group_end(ui, group);
    cui_end_frame(ui);
    const cui_rect bounds = fpas_cui_node_rect(ui, button);

    cui_begin_frame(ui, .016f);
    fpas_cui_move_pointer(ui, 270, 210);
    const int current_group = fpas_cui_group_begin(ui, 0, 8, 0, CUI_ALIGN_START, 0, 0, 0);
    (void)cui_button(ui, "Continue");
    const int current_button = fpas_cui_last_node(ui);
    const bool idle = !fpas_cui_hovered(ui, current_group) && !fpas_cui_hovered(ui, current_button);
    fpas_cui_move_pointer(ui, bounds.x + bounds.w / 2, bounds.y + bounds.h / 2);
    const bool hovered = !fpas_cui_hovered(ui, current_group) && fpas_cui_hovered(ui, current_button);
    fpas_cui_group_end(ui, current_group);
    cui_end_frame(ui);

    cui_destroy(ui);
    if (!idle || !hovered) fprintf(stderr, "Hover must identify the interactive button only (idle=%d hovered=%d).\n", idle, hovered);
    return idle && hovered;
}

int main(void) {
    bool valid = check_hover();
    for (int axis = 0; axis < 3; ++axis)
        for (int rtl = 0; rtl < 2; ++rtl)
            for (int dark = 0; dark < 2; ++dark)
                for (int narrow = 0; narrow < 2; ++narrow)
                    valid = matches(axis, rtl != 0, dark != 0, narrow ? 280 : 640) && valid;
    return valid ? 0 : 1;
}
