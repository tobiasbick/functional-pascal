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

int main(int argc, char **argv) {
    const bool adapter = argc < 2 || strcmp(argv[1], "upstream") != 0;
    return gradient_keys(adapter) ? 0 : 1;
}
