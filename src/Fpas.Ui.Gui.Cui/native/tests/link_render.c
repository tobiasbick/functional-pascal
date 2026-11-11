/* Checks real consumer linkage, font initialization and software rendering
 * against either the shared library or static archive, without a display. */
#include "../fpas_cui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    int writes;
    bool exact;
} copy_probe;

static const char copy_text[] = "\xce\xbb\ntext";

static void record_copy(void *user, const char *text) {
    copy_probe *probe = user;
    probe->writes++;
    probe->exact = probe->exact && strcmp(text, copy_text) == 0;
}

/* Drives the real clipboard callback without opening the user's clipboard. */
static bool check_copy_button(cui_ctx *ui) {
    copy_probe probe = { .exact = true };
    const cui_platform_iface platform = { .user = &probe, .set_clipboard = record_copy };
    const int64_t times[] = { 0, 20, 21, 1500, 1621, 3100, 3101 };
    const int expected[] = { 4, 5, 6, 7, 6, 4, 4 };
    unsigned int identity = 0;
    bool passed = true;
    cui_set_platform(ui, &platform);
    for (int frame = 0; frame < 7; frame++) {
        if (frame > 0) {
            int64_t remaining = times[frame] - times[frame - 1];
            while (remaining > 0) {
                const int64_t step = remaining > 100 ? 100 : remaining;
                cui_begin_frame(ui, (float)step / 1000.0f);
                fpas_cui_copy_button(ui, "###copy-fixture", copy_text, 1, 0);
                cui_end_frame(ui);
                remaining -= step;
            }
        }
        cui_begin_frame(ui, 1.0f / 60.0f);
        if (frame == 1 || frame == 3 || frame == 6) {
            const cui_key key = frame == 3 ? CUI_KEY_SPACE : CUI_KEY_ENTER;
            cui_input_key(ui, key, true, 0);
            cui_input_key(ui, key, false, 0);
        }
        const int events = fpas_cui_copy_button(ui, "###copy-fixture", copy_text, frame != 6, frame == 0);
        const int node = fpas_cui_last_node(ui);
        const unsigned int key = fpas_cui_node_key(ui, node);
        if (frame == 0) identity = key;
        if (events != expected[frame] || !key || key != identity || fpas_cui_focused(ui, node) != (frame != 6))
            passed = false;
        cui_end_frame(ui);
    }
    cui_set_platform(ui, NULL);
    return passed && probe.writes == 2 && probe.exact;
}

static int hold_frames(cui_ctx *ui, int frames, bool enabled, bool focus) {
    int confirmations = 0;
    for (int frame = 0; frame < frames; ++frame) {
        cui_begin_frame(ui, 0.04f);
        confirmations += fpas_cui_hold_button(ui, "###hold-fixture", "Hold ## literally", NULL,
                                              120, 4, enabled, focus && frame == 0);
        cui_end_frame(ui);
    }
    return confirmations;
}

static bool check_hold_button(cui_ctx *ui) {
    if (hold_frames(ui, 1, true, true) != 0) return false;
    cui_input_key(ui, CUI_KEY_ENTER, true, 0);
    if (hold_frames(ui, 1, true, false) != 0) return false;
    cui_input_key(ui, CUI_KEY_ENTER, false, 0);
    if (hold_frames(ui, 4, true, false) != 0) return false;

    cui_input_key(ui, CUI_KEY_SPACE, true, 0);
    if (hold_frames(ui, 8, true, false) != 1) return false;
    if (hold_frames(ui, 8, true, false) != 0) return false;
    fpas_cui_forget(ui, fpas_cui_node_key(ui, fpas_cui_last_node(ui)));
    if (hold_frames(ui, 8, true, true) != 1) return false;
    cui_input_key(ui, CUI_KEY_SPACE, false, 0);
    if (hold_frames(ui, 4, true, false) != 0) return false;

    const cui_rect bounds = cui_last_rect(ui);
    cui_input_mouse_move(ui, bounds.x + bounds.w / 2, bounds.y + bounds.h / 2);
    if (hold_frames(ui, 1, true, false) != 0) return false;
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    if (hold_frames(ui, 8, true, false) != 1) return false;
    if (hold_frames(ui, 8, false, false) != 0) return false;
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    return hold_frames(ui, 1, true, false) == 0;
}

static bool check_copy_retirement(cui_ctx *ui) {
    copy_probe probe = { .exact = true };
    const cui_platform_iface platform = { .user = &probe, .set_clipboard = record_copy };
    cui_set_platform(ui, &platform);
    cui_begin_frame(ui, 0.04f);
    fpas_cui_copy_button(ui, "###copy-retirement", copy_text, 1, 1);
    cui_end_frame(ui);
    cui_begin_frame(ui, 0.04f);
    cui_input_key(ui, CUI_KEY_ENTER, true, 0);
    cui_input_key(ui, CUI_KEY_ENTER, false, 0);
    const int copied = fpas_cui_copy_button(ui, "###copy-retirement", copy_text, 1, 0);
    const cui_id key = fpas_cui_node_key(ui, fpas_cui_last_node(ui));
    cui_end_frame(ui);
    fpas_cui_forget(ui, key);
    cui_begin_frame(ui, 0.04f);
    const int fresh = fpas_cui_copy_button(ui, "###copy-retirement", copy_text, 1, 0);
    cui_end_frame(ui);
    cui_set_platform(ui, NULL);
    return (copied & 1) && fresh == 4 && probe.writes == 1 && probe.exact;
}

static int split_frame(cui_ctx *ui, int focus, int key, bool enabled, int *open, int *highlighted) {
    const char *labels[] = { "Save ## draft", "Save and publish", "Schedule..." };
    const char *icons[] = { "", "", "" };
    int nodes[5];
    if (key >= 0) cui_input_key(ui, (cui_key)key, true, 0);
    cui_begin_frame(ui, 1.0f / 60.0f);
    const int choice = fpas_cui_split_button(ui, "###split-fixture", "Save ## literal", "",
        labels, icons, 3, 1, 0, enabled, focus, nodes, open, highlighted);
    const bool tracked = nodes[0] >= 0 && nodes[1] >= 0 && (!*open || nodes[2] >= 0);
    cui_end_frame(ui);
    if (key >= 0) cui_input_key(ui, (cui_key)key, false, 0);
    return tracked ? choice : -99;
}

static bool check_split_button(cui_ctx *ui) {
    int open, highlighted;
    if (split_frame(ui, 1, -1, true, &open, &highlighted) != -1 || open) return false;
    if (split_frame(ui, 0, CUI_KEY_ENTER, true, &open, &highlighted) != 0) return false;
    if (split_frame(ui, 2, -1, true, &open, &highlighted) != -1) return false;
    if (split_frame(ui, 0, CUI_KEY_DOWN, true, &open, &highlighted) != -1 || !open) return false;
    if (split_frame(ui, 0, CUI_KEY_DOWN, true, &open, &highlighted) != -1 || highlighted != 0) return false;
    if (split_frame(ui, 0, CUI_KEY_DOWN, true, &open, &highlighted) != -1 || highlighted != 1) return false;
    if (split_frame(ui, 0, CUI_KEY_ENTER, true, &open, &highlighted) != 2 || open) return false;
    if (split_frame(ui, 0, CUI_KEY_DOWN, true, &open, &highlighted) != -1 || !open) return false;
    if (split_frame(ui, 0, CUI_KEY_ESCAPE, true, &open, &highlighted) != -1 || open) return false;
    if (split_frame(ui, 0, CUI_KEY_ENTER, false, &open, &highlighted) != -1 || open) return false;
    return split_frame(ui, 0, CUI_KEY_DOWN, false, &open, &highlighted) == -1 && !open;
}

/* Returns success only when the linked CUI implementation produces a drawn view. */
int main(void) {
    const int width = 320, height = 160;
    const size_t bytes = (size_t)width * (size_t)height * 4;
    cui_sw_renderer *renderer = cui_sw_create();
    if (!renderer) {
        fprintf(stderr, "Could not create CUI software renderer.\n");
        return 1;
    }
    const cui_ctx_desc settings = { .renderer = cui_sw_iface(renderer) };
    cui_ctx *ui = cui_create(&settings);
    unsigned char *pixels = calloc(1, bytes);
    int result = 1;
    if (!ui || !pixels) {
        fprintf(stderr, "Could not allocate CUI context or pixels.\n");
        goto cleanup;
    }
    cui_set_viewport(ui, (float)width, (float)height, 1);
    cui_set_reduced_motion(ui, true);
    if (!check_copy_button(ui)) {
        fprintf(stderr, "Copy button did not preserve clipboard content, focus or confirmation timing.\n");
        goto cleanup;
    }
    if (!check_hold_button(ui)) {
        fprintf(stderr, "Hold button did not require a continuous press or fired more than once.\n");
        goto cleanup;
    }
    if (!check_copy_retirement(ui)) {
        fprintf(stderr, "Removing a copy button retained its confirmation state.\n");
        goto cleanup;
    }
    if (!check_split_button(ui)) {
        fprintf(stderr, "Split button did not route its main action and keyboard menu independently.\n");
        goto cleanup;
    }
    cui_begin_frame(ui, 1.0f / 60.0f);
    fpas_cui_scroll_begin(ui, "smoke-scroll", 160, 0);
    fpas_cui_column_begin(ui, 12, 8);
    cui_heading(ui, "FPAS native CUI", 2);
    (void)cui_button(ui, "Continue");
    cui_box_end(ui);
    cui_scroll_end(ui);
    cui_end_frame(ui);
    const cui_draw_data *draw = cui_get_draw_data(ui);
    if (!draw || draw->width_px != width || draw->height_px != height ||
        draw->vertex_count <= 0 || draw->index_count <= 0 || draw->command_count <= 0) {
        fprintf(stderr, "CUI did not produce the expected draw data.\n");
        goto cleanup;
    }
    cui_sw_render(renderer, draw, pixels, width, height, width * 4, CUI_SW_RGBA);
    for (size_t offset = 4; offset < bytes; offset += 4) {
        if (memcmp(pixels, pixels + offset, 4) != 0) {
            result = 0;
            break;
        }
    }
    if (result == 0) puts("CUI consumer link and render passed.");
    else fprintf(stderr, "CUI rendered only a uniform background.\n");

cleanup:
    free(pixels);
    if (ui) cui_destroy(ui);
    cui_sw_destroy(renderer);
    return result;
}
