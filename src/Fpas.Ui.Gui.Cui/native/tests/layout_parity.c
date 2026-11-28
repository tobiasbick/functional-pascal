// [Documentation](../../../../docs/ui/gui/layout/README.md)
/* Insets, RTL, stacks and spacing must paint exactly like direct CUI calls. */
#include "../fpas_cui.h"
#include <math.h>
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
            fpas_cui_formatted_text(ui, "0123\n8765", CUI_FONT_MEDIUM, 15, 0xffd02010u,
                                    CUI_DIRECTION_LTR, CUI_TEXT_END, 0, 2, 98, 1.8f, 1);
            fpas_cui_formatted_text(ui, CUI_ICON_COPY, CUI_FONT_ICONS, 16, 0,
                                    CUI_DIRECTION_INHERIT, CUI_TEXT_START, 0, 0, 0, 0, 0);
        } else {
            cui_text(ui, "0123\n8765", .font = CUI_FONT_MEDIUM, .size = 15, .color = CUI_RGBA(0x1020d0ff),
                     .direction = CUI_DIRECTION_LTR, .align = CUI_TEXT_END,
                     .width = CUI_PX(98), .line_height = 1.8f, .tabular = true);
            cui_text(ui, CUI_ICON_COPY, .font = CUI_FONT_ICONS, .size = 16);
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
    const char *text = "111: a long team member name that must remain on a single line";
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
                                            CUI_DIRECTION_LTR, CUI_TEXT_CENTER, 1, 2, 84, 0, 1);
                } else {
                    cui_text(ui, text, .font = CUI_FONT_MEDIUM, .size = 14, .color = CUI_RGBA(0x1020d0ff),
                             .direction = CUI_DIRECTION_LTR, .nowrap = true, .width = CUI_PX(84),
                             .align = CUI_TEXT_CENTER, .tabular = true);
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

/** Checks that an overflowing label scrolls on hover and resets when the pointer leaves. */
static bool check_hover_scroll_text(void) {
    enum { WIDTH = 300, HEIGHT = 100, PIXELS = WIDTH * HEIGHT * 4 };
    static unsigned char pixels[6][PIXELS];
    cui_sw_renderer *renderer = cui_sw_create();
    if (!renderer) return false;
    const cui_ctx_desc settings = { .renderer = cui_sw_iface(renderer) };
    cui_ctx *ui = cui_create(&settings);
    if (!ui) { cui_sw_destroy(renderer); return false; }
    cui_set_viewport(ui, WIDTH, HEIGHT, 1);
    cui_set_reduced_motion(ui, false);
    cui_rect bounds = {0};
    for (int sample = 0; sample < 6; ++sample) {
        const bool hovered = sample == 1 || sample == 2 || sample == 5;
        const char *text = sample < 4 ? "A long file path with a distinctive filename.mp3" : "Short.mp3";
        cui_input_mouse_move(ui, hovered ? bounds.x + bounds.w / 2 : WIDTH - 1,
                             hovered ? bounds.y + bounds.h / 2 : HEIGHT - 1);
        const int frames = sample == 2 || sample == 5 ? 20 : 1;
        for (int frame = 0; frame < frames; ++frame) {
            int label = 0;
            cui_begin_frame(ui, .05f);
            cui_row(ui, .width = CUI_FILL, .pad = 8, .gap = 8) {
                cui_text(ui, text, .width = CUI_PX(100));
                label = fpas_cui_last_node(ui);
                fpas_cui_node_role(ui, label, 30);
                (void)cui_button(ui, "Copy");
            }
            cui_end_frame(ui);
            bounds = fpas_cui_node_rect(ui, label);
        }
        cui_sw_render(renderer, cui_get_draw_data(ui), pixels[sample],
                      WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
    }
    cui_destroy(ui);
    cui_sw_destroy(renderer);
    const bool moved = memcmp(pixels[1], pixels[2], PIXELS) != 0;
    const bool reset = memcmp(pixels[0], pixels[3], PIXELS) == 0;
    const bool short_still = memcmp(pixels[4], pixels[5], PIXELS) == 0;
    if (!moved || !reset || !short_still)
        fprintf(stderr, "Hover text: moved=%d reset=%d short_still=%d\n", moved, reset, short_still);
    return moved && reset && short_still;
}

/** Compares every card child layout and the full native card options against direct CUI. */
static bool check_card_arrangements(void) {
    enum { WIDTH = 300, HEIGHT = 180 };
    static unsigned char pixels[2][WIDTH * HEIGHT * 4];
    for (int direction = CUI_COLUMN; direction <= CUI_GRID; ++direction) {
        for (int pass = 0; pass < 2; ++pass) {
            cui_sw_renderer *renderer = cui_sw_create();
            const cui_ctx_desc context = { .renderer = cui_sw_iface(renderer) };
            cui_ctx *ui = cui_create(&context);
            if (!ui) { cui_sw_destroy(renderer); return false; }
            const cui_card_opts card = {
                .id = "arrangement", .width = CUI_PCT(80), .height = CUI_PX(145),
                .min_width = 170, .max_width = 195, .pad = 12, .pad_x = 17, .pad_y = 8,
                .gap = 7, .dir = (cui_dir)direction, .align = CUI_ALIGN_CENTER,
                .wrap = direction == CUI_ROW, .hoverable = true,
                .muted = direction == CUI_STACK, .selected = true
            };
            int node = -1;
            cui_set_viewport(ui, WIDTH, HEIGHT, 1);
            cui_set_reduced_motion(ui, true);
            cui_begin_frame(ui, .016f);
            if (pass) {
                fpas_cui_card_begin(ui, card.id, 3, card.width.value, 2, card.height.value,
                    card.min_width, card.max_width, card.pad, card.pad_x, card.pad_y, card.gap,
                    direction, card.align, card.wrap, card.muted, card.selected, card.hoverable, 1, 0, &node);
            } else cui_card_begin(ui, &card);
            cui_button(ui, "First", .width = CUI_PX(95));
            cui_button(ui, "Second", .width = CUI_PX(90));
            if (pass) fpas_cui_card_end(ui, node);
            else cui_card_end(ui);
            cui_end_frame(ui);
            cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
            cui_destroy(ui);
            cui_sw_destroy(renderer);
        }
        if (memcmp(pixels[0], pixels[1], sizeof pixels[0]) != 0) {
            fprintf(stderr, "Card arrangement %d differs from direct CUI.\n", direction);
            return false;
        }
    }
    return true;
}

typedef struct { float inset; bool geometry_retained; } reader_geometry;

static int reader_frame(cui_ctx *ui, reader_geometry *geometry) {
    const char *titles[] = {"Introduction", "Installation", "First window"};
    const char *anchors[] = {"intro", "install", "window"};
    const int levels[] = {0, 1, 0};
    int active = -1;
    cui_id install = 0;
    cui_begin_frame(ui, .04f);
    cui_row(ui, .width = CUI_FILL, .gap = 16) {
        cui_scroll(ui, .id = "reader", .width = CUI_FILL, .height = CUI_PX(240)) {
            for (int index = 0; index < 3; ++index) {
                fpas_cui_scroll_anchor(ui, anchors[index]);
                if (index == 1) install = fpas_cui_node_key(ui, fpas_cui_last_node(ui));
                cui_heading(ui, titles[index], 3);
                cui_gap(ui, 400);
            }
        }
        cui_column(ui, .width = CUI_PX(180)) {
            const float previous = cui_get_rect(ui, install).y;
            int outline;
            active = fpas_cui_table_of_contents(ui, "reader", titles, anchors, levels, 3, 1, 0, &outline, NULL);
            geometry->geometry_retained &= cui_get_rect(ui, install).y == previous;
        }
    }
    cui_end_frame(ui);
    geometry->inset = cui_get_rect(ui, install).y - cui_get_rect(ui, cui_make_id(ui, "reader")).y;
    return active;
}

static void reader_key(cui_ctx *ui, cui_key key) {
    cui_input_key(ui, key, true, 0);
    cui_input_key(ui, key, false, 0);
}

/** Jump while the article receives another scroll request in the same frame. */
static bool check_reader_jump(bool reduced_motion) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 640, 320, 1);
    cui_set_reduced_motion(ui, reduced_motion);
    reader_geometry geometry = {.geometry_retained = true};
    bool valid = reader_frame(ui, &geometry) == 0;
    reader_key(ui, CUI_KEY_TAB);
    valid = reader_frame(ui, &geometry) == 0 && valid;
    reader_key(ui, CUI_KEY_TAB);
    valid = reader_frame(ui, &geometry) == 0 && valid;

    cui_scroll_to(ui, "reader", 280);
    reader_key(ui, CUI_KEY_ENTER);
    int active = reader_frame(ui, &geometry);
    valid = active == 1 && valid;
    for (int frame = 0; frame < 80 && cui_needs_redraw(ui); ++frame) {
        active = reader_frame(ui, &geometry);
        if (reduced_motion) valid = active == 1 && valid;
    }
    valid = active == 1 && !cui_needs_redraw(ui) && geometry.geometry_retained &&
        fabsf(geometry.inset - 12) < 1 && valid;
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "Reader jump differs from the anchor inset: reduced=%d active=%d inset=%g\n",
        reduced_motion, active, geometry.inset);
    return valid;
}

int main(void) {
    bool valid = check_hover() && check_box_close() && check_text_alignment() &&
                 check_formatted_text_clipping() && check_hover_scroll_text() && check_card_arrangements() &&
                 check_reader_jump(false) && check_reader_jump(true);
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
