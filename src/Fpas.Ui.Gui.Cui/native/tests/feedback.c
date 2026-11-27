// [Documentation](../../../../docs/ui/gui/native-build.md#feedback-adapters)
/* Native widgets and their ABI adapters must produce identical pixels. */
#include "../fpas_cui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static bool pixels_match(int kind, int tone, bool decorated, bool dark, int width, bool dither) {
    const int height = 440;
    const size_t bytes = (size_t)width * height * 4;
    unsigned char *pixels[2] = { calloc(1, bytes), calloc(1, bytes) };
    if (!pixels[0] || !pixels[1]) { free(pixels[0]); free(pixels[1]); return false; }
    bool valid = true;
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        const cui_ctx_desc settings = { .renderer = cui_sw_iface(renderer) };
        cui_ctx *ui = cui_create(&settings);
        if (!ui) { cui_sw_destroy(renderer); valid = false; break; }
        cui_set_viewport(ui, width, height, 1);
        cui_set_theme(ui, dark ? CUI_THEME_DARK : CUI_THEME_LIGHT);
        for (int frame = 0; frame < 4; ++frame) {
            cui_begin_frame(ui, 0.04f);
            cui_column(ui, .width = CUI_FILL, .pad = 16, .gap = 8) {
                if (kind == 0) {
                    const cui_alert_opts options = { .tone = (cui_tone)tone,
                        .description = decorated ? "Supporting ## literal description wraps across narrow space." : NULL,
                        .icon = decorated ? CUI_ICON_STAR : NULL, .dismissible = decorated };
                    if (pass) {
                        int node, dismiss; unsigned int key;
                        valid = fpas_cui_alert(ui, "###alert", "Native heading", options.description,
                            options.icon, tone, decorated, 1, 0, &node, &dismiss, &key) == 1 && valid;
                    } else valid = cui_alert_ex(ui, "Native heading", &options) && valid;
                } else if (kind == 1) {
                    const cui_progress_opts options = { .tone = (cui_tone)tone,
                        .label = decorated ? "Uploading assets" : NULL, .show_value = decorated,
                        .indeterminate = !decorated, .height = decorated ? 12 : 0, .dither = dither };
                    if (pass) fpas_cui_progress(ui, "###progress", .68f, tone, !decorated,
                        options.height, options.label ? options.label : "", decorated, dither, 0, 0);
                    else cui_progress_ex(ui, .68f, &options);
                } else if (kind == 2) {
                    if (pass) fpas_cui_spinner(ui, "###spinner", decorated ? 28 : 0);
                    else cui_spinner(ui, decorated ? 28 : 0);
                } else if (kind == 3) {
                    const char *text = decorated ? "Update available#" : "";
                    const cui_banner_opts options = { .tone = (cui_tone)tone,
                        .action = decorated ? "Install" : "", .icon = decorated ? CUI_ICON_STAR : "",
                        .dismissible = true };
                    if (pass) {
                        int node, action, dismiss;
                        unsigned int key;
                        fpas_cui_banner(ui, "###banner", text, 1, options.action, options.icon, tone, 1, 1, 0,
                            &node, &action, &dismiss, &key);
                    } else {
                        bool shown = true;
                        cui_banner_ex(ui, text, &shown, &options);
                    }
                } else if (kind == 4) {
                    const cui_checklist_opts options = { .title = decorated ? "" : NULL,
                        .description = decorated ? "" : NULL };
                    cui_check_step step = { .title = "Prepare", .description = decorated ? "" : NULL,
                        .action = decorated ? "" : "Prepare#" };
                    if (pass) {
                        const char *titles[] = {step.title}, *descriptions[] = {step.description}, *actions[] = {step.action};
                        int node, parts[2], completed = 0;
                        int action = fpas_cui_checklist(ui, "###checklist", options.title, options.description,
                            titles, descriptions, actions, &completed, 1, 1, 0, &node, parts);
                        valid = action == -1 && parts[1] >= 0 && valid;
                    } else cui_checklist_ex(ui, "checklist", &step, 1, &options);
                } else {
                    const cui_countdown_opts options = { .compact = decorated, .hide_days = decorated };
                    const double remaining = frame < 2 ? 177420.75 : 177419.25;
                    if (pass) fpas_cui_countdown(ui, "###countdown", remaining, options.compact, options.hide_days);
                    else cui_countdown_ex(ui, "countdown", remaining, &options);
                }
            }
            cui_end_frame(ui);
            cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], width, height, width * 4, CUI_SW_RGBA);
        }
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    valid = valid && memcmp(pixels[0], pixels[1], bytes) == 0;
    free(pixels[0]); free(pixels[1]);
    if (!valid) fprintf(stderr, "Feedback pixel mismatch: kind=%d tone=%d decorated=%d dark=%d width=%d\n",
        kind, tone, decorated, dark, width);
    return valid;
}

static bool progress_widths_match(bool labeled) {
    enum { WIDTH = 400, HEIGHT = 260 };
    const char *ids[] = { "###fit", "###fill", "###fixed", "###percent", "###weighted" };
    const cui_size widths[] = { CUI_FIT, CUI_FILL, CUI_PX(104), CUI_PCT(50), CUI_GROW(2) };
    const float extent_values[] = { 0, 0, 104, 50, 0 };
    const char *label = labeled ? "Progress" : NULL;
    const size_t bytes = WIDTH * HEIGHT * 4;
    unsigned char *pixels[2] = { calloc(1, bytes), calloc(1, bytes) };
    bool valid = pixels[0] && pixels[1];
    for (int pass = 0; pass < 2 && valid; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        const cui_ctx_desc settings = { .renderer = cui_sw_iface(renderer) };
        cui_ctx *ui = cui_create(&settings);
        if (!ui) { cui_sw_destroy(renderer); valid = false; break; }
        cui_set_viewport(ui, WIDTH, HEIGHT, 1);
        cui_set_reduced_motion(ui, true);
        cui_begin_frame(ui, .04f);
        cui_column(ui, .width = CUI_FILL, .pad = 16, .gap = 8) {
            for (int kind = 0; kind < 5; ++kind) {
                cui_row(ui, .width = CUI_FILL, .gap = 8) {
                    if (kind == 4) cui_spacer(ui);
                    if (pass) {
                        fpas_cui_progress(ui, ids[kind], .68f, CUI_TONE_ACCENT, 0, 0,
                            label ? label : "", labeled, 0, kind == 4 ? 0 : kind, extent_values[kind]);
                        if (kind == 4)
                            fpas_cui_node_size(ui, fpas_cui_last_node(ui), 1, 1, 0, 0, -1, 2, 0);
                    } else {
                        cui_progress(ui, .68f, .tone = CUI_TONE_ACCENT, .label = label,
                            .show_value = labeled, .width = widths[kind]);
                    }
                }
            }
        }
        cui_end_frame(ui);
        cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    valid = valid && memcmp(pixels[0], pixels[1], bytes) == 0;
    free(pixels[0]); free(pixels[1]);
    if (!valid) fprintf(stderr, "Progress width options differ from CUI: labeled=%d\n", labeled);
    return valid;
}

static bool narrow_progress_clips_fill(bool meter) {
    enum { WIDTH = 128, HEIGHT = 96 };
    unsigned char pixels[2][WIDTH * HEIGHT * 4] = {0};
    cui_sw_renderer *renderer = cui_sw_create();
    const cui_ctx_desc settings = { .renderer = cui_sw_iface(renderer) };
    cui_ctx *ui = cui_create(&settings);
    if (!ui) { cui_sw_destroy(renderer); return false; }
    cui_set_viewport(ui, WIDTH, HEIGHT, 1);
    cui_set_reduced_motion(ui, true);
    cui_rect bounds;
    for (int value = 0; value <= 1; ++value) {
        int node;
        cui_begin_frame(ui, .04f);
        cui_column(ui, .pad = 20) {
            if (meter) {
                cui_column(ui, .width = CUI_PX(4)) {
                    fpas_cui_meter(ui, "###meter", "Usage", (float)value, 1, "");
                    node = fpas_cui_last_node(ui);
                }
            } else {
                fpas_cui_progress(ui, "###bar", (float)value, CUI_TONE_ACCENT, 0, 20, "", 0, 0, 2, 4);
                node = fpas_cui_last_node(ui);
            }
        }
        cui_end_frame(ui);
        bounds = fpas_cui_node_rect(ui, node);
        cui_sw_render(renderer, cui_get_draw_data(ui), pixels[value], WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
    }
    cui_destroy(ui);
    cui_sw_destroy(renderer);
    const int y = (int)(bounds.y + (meter ? bounds.h - 3 : bounds.h / 2));
    const int inside = (y * WIDTH + (int)(bounds.x + bounds.w / 2)) * 4;
    const int beyond = (y * WIDTH + (int)(bounds.x + bounds.w)) * 4;
    const bool valid = bounds.w == 4 && memcmp(&pixels[0][inside], &pixels[1][inside], 4) != 0 &&
        memcmp(&pixels[0][beyond], &pixels[1][beyond], 4) == 0;
    if (!valid) fprintf(stderr, "Narrow solid fill escaped its track: meter=%d\n", meter);
    return valid;
}

/* Native painting, not a managed timer, must request activity frames. */
static bool activity_requests_frames(int kind) {
    cui_sw_renderer *renderer = cui_sw_create();
    const cui_ctx_desc settings = { .renderer = cui_sw_iface(renderer) };
    cui_ctx *ui = cui_create(&settings);
    if (!ui) { cui_sw_destroy(renderer); return false; }
    cui_set_viewport(ui, 320, 160, 1);
    bool active = false;
    for (int frame = 0; frame < 32; ++frame) {
        cui_begin_frame(ui, .04f);
        cui_column(ui, .width = CUI_FILL, .pad = 16) {
            if (frame < 24) {
                if (kind) fpas_cui_spinner(ui, "###spin", 22);
                else fpas_cui_progress(ui, "###progress", 0, 0, 1, 0, "Indexing", 0, 0, 0, 0);
            } else cui_text(ui, "Finished");
        }
        cui_end_frame(ui);
        if (frame == 23) active = cui_needs_redraw(ui);
    }
    const bool stopped = !cui_needs_redraw(ui);
    cui_destroy(ui);
    cui_sw_destroy(renderer);
    if (!active || !stopped) fprintf(stderr, "Native activity wakeup mismatch: kind=%d\n", kind);
    return active && stopped;
}

static bool banner_preserves_dismiss_focus(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 640, 200, 1);
    cui_set_reduced_motion(ui, true);
    int node, action, dismiss;
    unsigned int key;

    cui_begin_frame(ui, .04f);
    fpas_cui_banner(ui, "###banner", "Update available", 1, "Install", NULL,
        CUI_TONE_INFO, 1, 1, 2, &node, &action, &dismiss, &key);
    cui_end_frame(ui);
    const unsigned int banner_key = key;
    const unsigned int dismiss_key = fpas_cui_node_key(ui, dismiss);
    const bool initially_focused = fpas_cui_focus_key(ui, dismiss_key);

    cui_begin_frame(ui, .04f);
    fpas_cui_banner(ui, "###banner", "Update available", 1, "Install now", NULL,
        CUI_TONE_INFO, 1, 1, 0, &node, &action, &dismiss, &key);
    cui_end_frame(ui);
    const bool still_focused = fpas_cui_node_key(ui, dismiss) == dismiss_key &&
        fpas_cui_focus_key(ui, dismiss_key);

    cui_begin_frame(ui, .04f);
    fpas_cui_banner(ui, "###banner", "Update available#", 1, "Install now", NULL,
        CUI_TONE_INFO, 1, 1, 0, &node, &action, &dismiss, &key);
    cui_end_frame(ui);
    const bool literal_focus = key == banner_key && fpas_cui_node_key(ui, dismiss) == dismiss_key &&
        fpas_cui_focus_key(ui, dismiss_key);

    cui_begin_frame(ui, .04f);
    cui_input_key(ui, CUI_KEY_ENTER, true, 0);
    cui_input_key(ui, CUI_KEY_ENTER, false, 0);
    const int result = fpas_cui_banner(ui, "###banner", "Update available#", 1, "Install now", NULL,
        CUI_TONE_INFO, 1, 1, 0, &node, &action, &dismiss, &key);
    cui_end_frame(ui);
    cui_destroy(ui);

    if (!initially_focused || !still_focused || !literal_focus || !(result & 2))
        fprintf(stderr, "Banner action and caption changes must preserve keyboard dismissal.\n");
    return initially_focused && still_focused && literal_focus && (result & 2);
}

static bool feedback_paints_focus(bool banner) {
    enum { WIDTH = 640, HEIGHT = 200 };
    const size_t bytes = WIDTH * HEIGHT * 4;
    unsigned char *before = calloc(1, bytes), *after = calloc(1, bytes);
    cui_sw_renderer *renderer = cui_sw_create();
    cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(renderer) });
    bool valid = ui && before && after;
    if (valid) {
        cui_set_viewport(ui, WIDTH, HEIGHT, 1);
        cui_set_reduced_motion(ui, true);
        int target = -1;
        for (int frame = 0; frame < 4; ++frame) {
            if (frame == 2) {
                cui_input_key(ui, CUI_KEY_TAB, true, 0);
                cui_input_key(ui, CUI_KEY_TAB, false, 0);
            }
            cui_begin_frame(ui, .04f);
            int root;
            if (banner) {
                int dismiss;
                unsigned int key;
                fpas_cui_banner(ui, "###banner", "Update available", 1, "Install", NULL,
                    0, 0, 1, 0, &root, &target, &dismiss, &key);
            } else {
                const char *titles[] = {"Prepare"}, *empty[] = {NULL};
                int parts[2], completed = 1;
                fpas_cui_checklist(ui, "###checklist", NULL, NULL, titles, empty, empty,
                    &completed, 1, 1, 0, &root, parts);
                target = parts[0];
            }
            cui_end_frame(ui);
            if (frame == 1 || frame == 3) cui_sw_render(renderer, cui_get_draw_data(ui),
                frame == 1 ? before : after, WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
        }
        valid = target >= 0 && fpas_cui_focus_key(ui, fpas_cui_node_key(ui, target)) && memcmp(before, after, bytes) != 0;
        if (valid && !banner) {
            const cui_rect circle = fpas_cui_node_rect(ui, target);
            const size_t body = ((size_t)(circle.y + circle.h / 2) * WIDTH + (size_t)(circle.x + 5)) * 4;
            valid = body + 4 <= bytes && before[body + 1] > before[body] && memcmp(before + body, after + body, 4) == 0;
        }
    }
    if (ui) cui_destroy(ui);
    cui_sw_destroy(renderer);
    free(before);
    free(after);
    if (!valid) fprintf(stderr, "Disclosure keyboard focus paint mismatch: banner=%d\n", banner);
    return valid;
}

int main(void) {
    bool valid = true;
    for (int kind = 0; kind < 6; ++kind)
        for (int tone = 0; tone < (kind >= 2 ? 1 : 9); ++tone)
            for (int decorated = 0; decorated < 2; ++decorated)
                for (int dark = 0; dark < 2; ++dark)
                    for (int narrow = 0; narrow < 2; ++narrow)
                        for (int dither = 0; dither < (kind == 1 ? 2 : 1); ++dither)
                            valid = pixels_match(kind, tone, decorated != 0, dark != 0,
                                narrow ? 280 : 640, dither != 0) && valid;
    return progress_widths_match(false) && progress_widths_match(true) &&
        narrow_progress_clips_fill(false) && narrow_progress_clips_fill(true) &&
        banner_preserves_dismiss_focus() && feedback_paints_focus(true) && feedback_paints_focus(false) &&
        activity_requests_frames(0) && activity_requests_frames(1) && valid ? 0 : 1;
}
