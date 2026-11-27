// [Documentation](../../../../docs/ui/gui/text-and-display/README.md)
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>
#include <string.h>

static bool close_to(float actual, float expected) {
    return fabsf(actual - expected) < 0.01f;
}

static bool check_canvas(bool covered, bool bridge) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 280, 180, 1);
    cui_set_reduced_motion(ui, true);
    const fpas_canvas_shape shape = {1, 15, 25, 0, 0, 6, 0xff112233u};
    int activations = 0;
    bool coordinates = false;
    for (int frame = 0; frame < 6; ++frame) {
        if (frame == 2) {
            cui_input_mouse_move(ui, 31, 41);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        }
        if (frame == 3) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        cui_begin_frame(ui, .016f);
        cui_box(ui, .dir = CUI_STACK, .pad = 16) {
            float x, y, w, h;
            float dx, dy;
            if (fpas_cui_drawing_canvas(ui, "canvas", 2, 80, 2, 60, &shape, 1, 1, &x, &y, &w, &h, 0, &dx, &dy, NULL, NULL, 0, 0, 0, 0, CUI_CURSOR_HAND) & 1) {
                ++activations;
                coordinates = close_to(x, 15) && close_to(y, 25) && close_to(w, 80) && close_to(h, 60) && cui_get_cursor(ui) == CUI_CURSOR_HAND;
            }
            if (covered) {
                if (bridge) {
                    fpas_cui_veil_begin(ui, "disabled-cover", 0, NULL, NULL, 0, 0, 0, 2, 80, 0);
                    cui_box(ui, .width = CUI_PX(80), .height = CUI_PX(60)) {}
                    fpas_cui_veil_end(ui);
                } else {
                    bool shown = false;
                    cui_push_disabled(ui, true);
                    cui_veil(ui, "disabled-cover", &shown, .width = CUI_PX(80)) {
                        cui_box(ui, .width = CUI_PX(80), .height = CUI_PX(60)) {}
                    }
                    cui_pop_disabled(ui);
                }
            }
        }
        cui_end_frame(ui);
    }
    cui_destroy(ui);
    if (activations == 1 && coordinates) return true;
    fprintf(stderr, "Expected one canvas activation at (15,25) in an 80x60 area; got %d.\n", activations);
    return false;
}

static bool check_literal_link(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 280, 180, 1);
    const char *labels[] = {"Docs", "Docs#", "#"};
    cui_id identity = 0;
    float plain_width = 0;
    bool ok = true;
    for (int frame = 0; frame < 3; ++frame) {
        cui_begin_frame(ui, .016f);
        int node;
        fpas_cui_link(ui, "###docs", labels[frame], 1, frame == 0, &node);
        cui_end_frame(ui);
        const cui_id key = fpas_cui_node_key(ui, node);
        const float width = fpas_cui_node_rect(ui, node).w;
        if (frame == 0) {
            identity = key;
            plain_width = width;
        }
        if (!key || key != identity || !fpas_cui_focused(ui, node) || width <= 0 ||
            (frame == 1 && width <= plain_width)) ok = false;
    }
    cui_destroy(ui);
    if (!ok) fprintf(stderr, "A trailing # changed the link's caption or focused identity.\n");
    return ok;
}

enum { color_width = 280, color_height = 180 };

static bool render_media_colors(uint8_t *pixels, bool bridge, bool dark) {
    cui_sw_renderer *renderer = cui_sw_create();
    if (!renderer) return false;
    cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(renderer) });
    if (!ui) {
        cui_sw_destroy(renderer);
        return false;
    }
    cui_set_viewport(ui, color_width, color_height, 1);
    cui_set_theme(ui, dark ? CUI_THEME_DARK : CUI_THEME_LIGHT);
    cui_set_reduced_motion(ui, true);
    const uint8_t white[] = { 255, 255, 255, 255 };
    const fpas_media_source source = { .width = 1, .height = 1, .pixels = white, .version = 1 };
    cui_texture texture = bridge ? 0 : cui_image_create(ui, 1, 1, white);
    cui_begin_frame(ui, .016f);
    cui_column(ui, .pad = 8, .gap = 6) {
        if (bridge) {
            fpas_cui_display_image(ui, "picture", &source, 2, 90, 2, 54, 9,
                -CUI_TOKEN_FOREGROUND - 1, 0);
            fpas_cui_veil_begin(ui, "cover", 0, "Picture", "Show", 7, -CUI_TOKEN_SURFACE - 1, 0, 2, 140, 3);
            cui_box(ui, .width = CUI_PX(140), .height = CUI_PX(60)) {}
            fpas_cui_veil_end(ui);
        } else {
            bool revealed = false;
            cui_image(ui, texture, .width = CUI_PX(90), .height = CUI_PX(54),
                .radius = 9, .tint = CUI_COLOR_FOREGROUND);
            cui_veil(ui, "cover", &revealed, .label = "Picture", .hint = "Show", .radius = 7,
                .color = CUI_COLOR_SURFACE, .width = CUI_PX(140), .toggle = true) {
                cui_box(ui, .width = CUI_PX(140), .height = CUI_PX(60)) {}
            }
        }
    }
    cui_end_frame(ui);
    cui_sw_render(renderer, cui_get_draw_data(ui), pixels, color_width, color_height, color_width * 4, CUI_SW_RGBA);
    if (texture) cui_image_destroy(ui, texture);
    cui_destroy(ui);
    cui_sw_destroy(renderer);
    return true;
}

static bool check_media_colors(void) {
    uint8_t direct[color_width * color_height * 4] = {0};
    uint8_t bridge[sizeof direct] = {0};
    for (int dark = 0; dark < 2; ++dark) {
        if (!render_media_colors(direct, false, dark) || !render_media_colors(bridge, true, dark) ||
            memcmp(direct, bridge, sizeof direct) != 0) {
            fprintf(stderr, "Native media theme color parity failed (dark=%d).\n", dark);
            return false;
        }
    }
    return true;
}

int main(int argc, char **argv) {
    (void)argv;
    if (argc > 1) return check_canvas(true, false) ? 0 : 1;
    return check_canvas(false, true) && check_canvas(true, true) && check_literal_link() && check_media_colors() ? 0 : 1;
}
