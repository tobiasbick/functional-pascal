#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>
#include <string.h>

static void key(cui_ctx *ui, cui_key input) {
    cui_input_key(ui, input, true, 0);
    cui_input_key(ui, input, false, 0);
}

static int gradient(cui_ctx *ui, bool adapter, float *value, bool enabled, int focus) {
    int parts[2];
    uint32_t color;
    if (adapter)
        return fpas_cui_gradient_slider(ui, "gradient", "Gradient", value, 0, 1,
            CUI_GRADIENT_OPACITY, NULL, NULL, 0, 0, 0, .3f,
            NULL, NULL, NULL, NULL, enabled, focus, &color, parts);
    return cui_gradient_slider(ui, "Gradient", value, 0, 1, .step = .3f,
        .track = CUI_GRADIENT_OPACITY, .disabled = !enabled);
}

static bool gradient_keys(bool adapter) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 420, 1);
    cui_set_reduced_motion(ui, true);
    float value = 0;
    bool ok = true;
    for (int i = 0; i < 6; ++i) {
        if (!adapter && i == 1) key(ui, CUI_KEY_TAB);
        if (i == 2) key(ui, CUI_KEY_END);
        if (i == 4) key(ui, CUI_KEY_LEFT);
        if (i == 5) value = .85f;
        cui_begin_frame(ui, .016f);
        const int flags = gradient(ui, adapter, &value, i < 5, i == 1 ? 0 : -1);
        cui_end_frame(ui);
        const float expected[] = { 0, 0, 1, 1, .6f, .85f };
        const bool changed = i == 2 || i == 4;
        if (fabsf(value - expected[i]) > .00001f || ((flags & CUI_WELL_CHANGED) != 0) != changed) {
            fprintf(stderr, "%s gradient frame %d: value=%g, flags=%d\n",
                adapter ? "FPAS" : "CUI", i, value, flags);
            ok = false;
        }
    }
    cui_destroy(ui);
    return ok;
}

static int option_caption_vertices(const char *caption) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return -1;
    cui_set_viewport(ui, 640, 420, 1);
    const char *items[] = {caption, "Other"};
    int parts[3], selected = 0;
    cui_begin_frame(ui, .016f);
    fpas_cui_option_action(ui, "options", "Options", items, 2, &selected, NULL, NULL, NULL, 1, -1, parts);
    cui_end_frame(ui);
    const int vertices = cui_get_draw_data(ui)->vertex_count;
    cui_destroy(ui);
    return vertices;
}

static bool literal_option_caption(void) {
    const int expected = option_caption_vertices("ABliteral");
    const int actual = option_caption_vertices("##literal");
    if (actual > 0 && actual == expected) return true;
    fprintf(stderr, "Option action literal caption drew %d vertices, expected %d.\n", actual, expected);
    return false;
}

static bool well_caption_keeps_focus(void) {
    const char *labels[] = {"Note", "C#", "#", "##hidden"};
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 320, 180, 1);
    float value = 0;
    unsigned int track = 0;
    bool valid = true;
    for (int i = 0; i < 4; ++i) {
        uint32_t sample;
        int parts[2];
        cui_begin_frame(ui, .016f);
        fpas_cui_gradient_slider(ui, "###level", labels[i], &value, 0, 1,
            CUI_GRADIENT_OPACITY, NULL, NULL, 0, 0, 0, .1f,
            NULL, NULL, NULL, NULL, 1, i == 0 ? 0 : -1, &sample, parts);
        cui_end_frame(ui);
        if (!i) track = fpas_cui_node_key(ui, parts[0]);
        valid &= track && fpas_cui_node_key(ui, parts[0]) == track && fpas_cui_focused(ui, parts[0]);
    }
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "A well caption changed the slider identity or keyboard focus.\n");
    return valid;
}

int main(int argc, char **argv) {
    const bool adapter = argc < 2 || strcmp(argv[1], "upstream") != 0;
    const bool gradient = gradient_keys(adapter);
    const bool captions = !adapter || literal_option_caption();
    return gradient && captions && (!adapter || well_caption_keeps_focus()) ? 0 : 1;
}
