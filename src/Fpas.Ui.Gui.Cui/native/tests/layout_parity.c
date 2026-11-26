// [Documentation](../../../../docs/ui/gui/layout/README.md)
/* Insets, RTL, stacks and spacing must paint exactly like direct CUI calls. */
#include "../fpas_cui.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

static bool matches(int axis, bool rtl, bool dark, int width, bool styled) {
    const int height = 220;
    const size_t bytes = (size_t)width * height * 4;
    unsigned char *pixels[2] = { calloc(1, bytes), calloc(1, bytes) };
    if (!pixels[0] || !pixels[1]) { free(pixels[0]); free(pixels[1]); return false; }
    bool valid = true;
    const bool grid = axis == CUI_GRID;
    const bool wrap = styled && axis == CUI_ROW;
    const cui_justify justify = wrap ? CUI_JUSTIFY_BETWEEN : CUI_JUSTIFY_START;
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
            if (grid) {
                node = fpas_cui_grid_begin(ui, 0, 100, 0, 7, styled,
                                           styled ? "styled" : NULL, styled ? 2 : 0);
            } else {
                node = fpas_cui_group_open(ui, axis, 0, 7, CUI_ALIGN_START, 0, wrap, styled,
                                           justify, styled ? "styled" : NULL, styled ? 2 : 0);
            }
            fpas_cui_box_details(ui, node, 9, 21, 15, 33, rtl ? CUI_DIRECTION_RTL : CUI_DIRECTION_LTR);
            if (styled)
                fpas_cui_group_style(ui, node, 0, CUI_TOKEN_SURFACE_RAISED, 3, 13, 19, 23,
                                     2, 0, CUI_TOKEN_ACCENT, CUI_ELEVATION_RAISED, .65f);
        } else {
            cui_box box = { .dir = (cui_dir)axis, .width = CUI_FILL, .height = CUI_FILL,
                .pad_top = 9, .pad_right = 21, .pad_bottom = 15, .pad_left = 33, .gap = 7,
                .align = CUI_ALIGN_START, .justify = justify, .wrap = wrap,
                .direction = rtl ? CUI_DIRECTION_RTL : CUI_DIRECTION_LTR };
            if (grid) {
                box.align = CUI_ALIGN_DEFAULT;
                box.min_column_width = 100;
            }
            if (styled) {
                box.id = "styled"; box.scroll_y = true; box.scroll_x = true; box.clip = true;
                box.bg = CUI_COLOR_SURFACE_RAISED; box.border = 2; box.border_color = CUI_COLOR_ACCENT;
                box.radius_tl = 3; box.radius_tr = 13; box.radius_br = 19; box.radius_bl = 23;
                box.elevation = CUI_ELEVATION_RAISED; box.opacity = .65f;
            }
            if (axis == CUI_ROW) cui_row_begin(ui, &box);
            else if (axis == CUI_COLUMN) cui_column_begin(ui, &box);
            else cui_box_begin(ui, &box);
        }
        cui_skeleton(ui, CUI_PX(60), 20);
        const int first = fpas_cui_last_node(ui);
        if (!grid) {
            cui_gap(ui, 10); cui_spacer(ui);
            cui_skeleton(ui, CUI_PX(40), 12);
        }
        if (wrap || grid) {
            for (int child = 0; child < 3; ++child) cui_skeleton(ui, CUI_PX(96), 20);
        }
        const int last = fpas_cui_last_node(ui);
        if (pass) {
            valid = fpas_cui_group_end(ui, node) && valid;
            fpas_cui_node_size(ui, node, 1, 1, 0, 0, -1, 0, 0);
            fpas_cui_node_size(ui, node, 0, 1, 0, 0, -1, 0, 0);
        } else cui_box_end(ui);
        cui_end_frame(ui);
        if (wrap || grid) {
            const bool wrapped = fpas_cui_node_rect(ui, last).y > fpas_cui_node_rect(ui, first).y;
            valid = valid && wrapped == (width == 280);
        }
        cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], width, height, width * 4, CUI_SW_RGBA);
        cui_destroy(ui); cui_sw_destroy(renderer);
    }
    valid = valid && memcmp(pixels[0], pixels[1], bytes) == 0;
    free(pixels[0]); free(pixels[1]);
    if (!valid) fprintf(stderr, "Layout mismatch: axis=%d rtl=%d dark=%d width=%d styled=%d\n",
                        axis, rtl, dark, width, styled);
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

/** A mismatched native close leaves the actual inner box open. */
static bool check_box_close(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 280, 220, 1);
    cui_begin_frame(ui, .016f);
    const int outer = fpas_cui_column_begin(ui, 0, 0);
    const int inner = fpas_cui_row_begin(ui, 0, 0);
    cui_gap(ui, 10);
    const int leaf = fpas_cui_last_node(ui);
    const bool unchanged = !fpas_cui_group_end(ui, outer) && fpas_cui_last_node(ui) == leaf;
    const bool inner_closed = fpas_cui_group_end(ui, inner) && fpas_cui_last_node(ui) == inner;
    const bool outer_closed = fpas_cui_group_end(ui, outer) && fpas_cui_last_node(ui) == outer;
    cui_end_frame(ui);
    cui_destroy(ui);
    return unchanged && inner_closed && outer_closed;
}

/** Checks all physical and logical text alignments against direct CUI in an RTL viewport. */
static bool check_text_alignment(void) {
    enum { WIDTH = 300, HEIGHT = 180 };
    static unsigned char pixels[2][WIDTH * HEIGHT * 4];
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        const cui_ctx_desc settings = { .renderer = cui_sw_iface(renderer), .direction = CUI_DIRECTION_RTL };
        cui_ctx *ui = cui_create(&settings);
        if (!ui) { cui_sw_destroy(renderer); return false; }
        cui_set_viewport(ui, WIDTH, HEIGHT, 1);
        cui_begin_frame(ui, .016f);
        for (int align = CUI_TEXT_START; align <= CUI_TEXT_END; ++align) {
            const cui_direction direction = align == CUI_TEXT_CENTER ? CUI_DIRECTION_AUTO : CUI_DIRECTION_INHERIT;
            if (pass) fpas_cui_display_text(ui, "Layout", 0, direction, align);
            else {
                const cui_text_opts options = { .direction = direction, .align = (cui_text_align)align };
                cui_text_ex(ui, "Layout", &options);
            }
        }
        if (pass) {
            fpas_cui_formatted_text(ui, "Formatted", CUI_FONT_MEDIUM, 15, 0xffd02010u,
                                    CUI_DIRECTION_LTR, CUI_TEXT_END, 0);
        } else {
            const cui_text_opts options = { .font = CUI_FONT_MEDIUM, .size = 15, .color = CUI_RGBA(0x1020d0ff),
                .direction = CUI_DIRECTION_LTR, .align = CUI_TEXT_END };
            cui_text_ex(ui, "Formatted", &options);
        }
        cui_end_frame(ui);
        cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    const bool matched = memcmp(pixels[0], pixels[1], sizeof pixels[0]) == 0;
    if (!matched) fprintf(stderr, "FPAS text direction and alignment differ from direct CUI.\n");
    return matched;
}

/** Compares the ellipsis and passive tooltip against a direct CUI text node. */
static bool check_formatted_text_clipping(void) {
    enum { WIDTH = 300, HEIGHT = 140, PIXELS = WIDTH * HEIGHT * 4 };
    static unsigned char pixels[2][2][PIXELS];
    const char *text = "A long team member name that must remain on a single line";
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        const cui_ctx_desc settings = { .renderer = cui_sw_iface(renderer) };
        cui_ctx *ui = cui_create(&settings);
        if (!ui) { cui_sw_destroy(renderer); return false; }
        cui_set_viewport(ui, WIDTH, HEIGHT, 1);
        cui_set_reduced_motion(ui, true);
        cui_input_mouse_move(ui, WIDTH - 1, HEIGHT - 1);
        for (int frame = 0; frame <= 52; ++frame) {
            if (frame == 1) cui_input_mouse_move(ui, 40, 48);
            cui_begin_frame(ui, .016f);
            cui_column(ui, .width = CUI_PX(100), .pad_top = 40) {
                if (pass) {
                    fpas_cui_formatted_text(ui, text, CUI_FONT_MEDIUM, 14, 0xffd02010u,
                                            CUI_DIRECTION_LTR, CUI_TEXT_START, 1);
                } else {
                    cui_text(ui, text, .font = CUI_FONT_MEDIUM, .size = 14, .color = CUI_RGBA(0x1020d0ff),
                             .direction = CUI_DIRECTION_LTR, .nowrap = true);
                }
            }
            cui_end_frame(ui);
            if (frame == 0 || frame == 52)
                cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass][frame != 0],
                              WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
        }
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    const bool matched = memcmp(pixels[0], pixels[1], sizeof pixels[0]) == 0;
    const bool tooltip_visible = memcmp(pixels[0][0], pixels[0][1], PIXELS) != 0;
    if (!matched || !tooltip_visible)
        fprintf(stderr, "Formatted text ellipsis or hover tooltip differs from direct CUI.\n");
    return matched && tooltip_visible;
}

int main(void) {
    bool valid = check_hover() && check_box_close() && check_text_alignment() && check_formatted_text_clipping();
    for (int axis = 0; axis < 3; ++axis)
        for (int rtl = 0; rtl < 2; ++rtl)
            for (int dark = 0; dark < 2; ++dark)
                for (int narrow = 0; narrow < 2; ++narrow) {
                    valid = matches(axis, rtl != 0, dark != 0, narrow ? 280 : 640, false) && valid;
                    if (axis == CUI_ROW || axis == CUI_STACK)
                        valid = matches(axis, rtl != 0, dark != 0, narrow ? 280 : 640, true) && valid;
                }
    for (int dark = 0; dark < 2; ++dark)
        valid = matches(CUI_COLUMN, true, dark != 0, 280, true) && valid;
    for (int rtl = 0; rtl < 2; ++rtl)
        for (int dark = 0; dark < 2; ++dark)
            for (int narrow = 0; narrow < 2; ++narrow)
                valid = matches(CUI_GRID, rtl != 0, dark != 0, narrow ? 280 : 640, true) && valid;
    return valid ? 0 : 1;
}
