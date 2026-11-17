/* Public API extensions must keep the pixels and input ownership of direct CUI. */
#include "../fpas_cui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void label(void *user, double value, char *out, size_t capacity) {
    snprintf(out, capacity, "%s%.0f", (const char *)user, value);
}

static void wheel_region(cui_ctx *ui, cui_rect rect, void *user) {
    cui_wheel_region(ui, *(const cui_id *)user, rect);
}

static bool pixels(bool trace, bool dark) {
    enum { WIDTH = 700, HEIGHT = 420 };
    const size_t bytes = WIDTH * HEIGHT * 4;
    unsigned char *images[2] = { calloc(1, bytes), calloc(1, bytes) };
    bool ok = images[0] && images[1];
    for (int pass = 0; ok && pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        const cui_ctx_desc desc = { .renderer = cui_sw_iface(renderer) };
        cui_ctx *ui = cui_create(&desc);
        cui_set_viewport(ui, WIDTH, HEIGHT, 1);
        cui_set_reduced_motion(ui, true);
        cui_set_theme(ui, dark ? CUI_THEME_DARK : CUI_THEME_LIGHT);
        for (int frame = 0; frame < 3; ++frame) {
            cui_input_mouse_move(ui, 300, 60);
            cui_begin_frame(ui, .016f);
            cui_column(ui, .pad = 8, .gap = 8) {
                if (trace) {
                    cui_trace_track tracks[] = { { "Send", false }, { "Receive", false } };
                    const cui_trace_span spans[] = { { "A", 0, 1, 0, 0 }, { "B", 3, 4, 1, 0 } };
                    const cui_trace_link links[] = { {0, 1, CUI_TONE_ACCENT, 1.5f, true} };
                    if (pass) {
                        fpas_dev_result result;
                        int hover;
                        fpas_cui_trace_viewer(ui, "trace", tracks, 2, spans, 2, NULL, 0,
                            0, 5, 0, 0, 0, -1, 210, 26, 1, 0, 1, &result, links, 1, &hover);
                    } else {
                        cui_trace_view view = {0, 5};
                        int selected = -1;
                        cui_trace_view(ui, "trace", tracks, 2, spans, 2, &view, &selected,
                            .height = 210, .row_height = 26, .links = links, .link_count = 1);
                    }
                } else {
                    const float samples[] = {1, 2, 3};
                    const int accent = 1;
                    double events[] = {1, 3, 0, 0};
                    int count = 2;
                    if (pass) {
                        const fpas_dot_options dot = { .chart = {.height = 130}, .x_min = 0, .x_max = 6,
                            .x_step = 2, .flags = 4, .x_label = label, .x_label_user = "X",
                            .value_label = label, .value_label_user = "V" };
                        const fpas_event_options track = { .minimum = 0, .maximum = 6, .label_step = 2,
                            .flags = 8, .label = label, .label_user = "T" };
                        int index, result[5] = {-1}; double x; float value;
                        fpas_cui_dot_chart(ui, "chart", samples, &accent, 1, 3, &dot, &index, &x, &value);
                        fpas_cui_event_track(ui, "events", events, &count, 4, &track, result);
                    } else {
                        const cui_dot_series band = {samples, NULL, true};
                        cui_dot_chart(ui, "chart", &band, 1, 3, .height = 130, .x_min = 0, .x_max = 6,
                            .x_step = 2, .x_label = label, .x_label_user = "X",
                            .value_label = label, .value_label_user = "V");
                        int selected = -1;
                        cui_event_track(ui, "events", events, &count, 4, 0, 6,
                            .label_step = 2, .selected = &selected, .label = label, .label_user = "T");
                    }
                }
            }
            cui_end_frame(ui);
        }
        cui_sw_render(renderer, cui_get_draw_data(ui), images[pass], WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
        cui_destroy(ui); cui_sw_destroy(renderer);
    }
    ok = ok && memcmp(images[0], images[1], bytes) == 0;
    free(images[0]); free(images[1]);
    if (!ok) fprintf(stderr, "Direct CUI pixel mismatch: trace=%d dark=%d\n", trace, dark);
    return ok;
}

static bool wheel(bool adapter) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 400, 200, 1);
    float dx = 0, dy = 0;
    cui_push_id(ui, "canvas");
    const cui_id wheel_key = cui_make_id(ui, "wheel");
    cui_pop_id(ui);
    for (int frame = 0; frame < 3; ++frame) {
        if (frame == 2) cui_input_scroll(ui, 2, -3);
        cui_input_mouse_move(ui, 50, 50);
        cui_begin_frame(ui, .016f);
        if (adapter) {
            float x, y, w, h;
            fpas_cui_drawing_canvas(ui, "canvas", 1, 0, 2, 140, NULL, 0, 1,
                &x, &y, &w, &h, 1, &dx, &dy, NULL, NULL, 0);
        } else {
            cui_vec2 delta = {0};
            cui_wheel_take(ui, wheel_key, &delta);
            dx = delta.x; dy = delta.y;
            cui_canvas(ui, "canvas", CUI_FILL, CUI_PX(140), wheel_region, &wheel_key, sizeof wheel_key);
        }
        cui_end_frame(ui);
    }
    const bool ok = dx == 2 && dy == -3;
    if (!ok) fprintf(stderr, "Native canvas wheel: adapter=%d dx=%g dy=%g\n", adapter, dx, dy);
    cui_destroy(ui);
    return ok;
}

int main(void) {
    return pixels(false, false) && pixels(false, true) && pixels(true, false) && pixels(true, true)
        && wheel(false) && wheel(true) ? 0 : 1;
}
