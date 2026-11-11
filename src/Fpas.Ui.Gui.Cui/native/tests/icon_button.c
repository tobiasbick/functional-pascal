/* Checks stable native icon actions without palette side effects. */
#include "../fpas_cui.h"
#include <stdio.h>
#include <string.h>

static int frame(cui_ctx *ui, int dark, int key, int enabled, int focus, unsigned int *identity) {
    cui_begin_frame(ui, 0.04f);
    if (key) {
        cui_input_key(ui, (cui_key)key, true, 0);
        cui_input_key(ui, (cui_key)key, false, 0);
    }
    const int activated = fpas_cui_icon_button(ui, "fixture", dark ? CUI_ICON_SUN : CUI_ICON_MOON,
        dark ? "Switch to light" : "Switch to dark", NULL, 0, CUI_SIZE_MD, enabled, 0, 0, focus);
    *identity = fpas_cui_node_key(ui, fpas_cui_last_node(ui));
    cui_end_frame(ui);
    return activated;
}

static bool verify(cui_ctx *ui) {
    uint32_t before[CUI_TOKEN_COUNT - 1], after[CUI_TOKEN_COUNT - 1];
    fpas_cui_theme_read(ui, before);
    before[0] = 0xff352719u;
    fpas_cui_theme(ui, 1, 2, before);
    unsigned int initial, current;
    if (frame(ui, 0, 0, 1, 1, &initial)) return false;
    if (!fpas_cui_focus_key(ui, initial)) return false;
    if (!frame(ui, 0, CUI_KEY_ENTER, 1, 0, &current)) return false;
    if (frame(ui, 1, 0, 1, 0, &current) || current != initial) return false;
    if (!fpas_cui_focus_key(ui, initial)) return false;
    if (!frame(ui, 1, CUI_KEY_SPACE, 1, 0, &current)) return false;
    if (frame(ui, 1, CUI_KEY_ENTER, 0, 1, &current)) return false;
    if (fpas_cui_focus_key(ui, initial)) return false;
    fpas_cui_theme_read(ui, after);
    return memcmp(before, after, sizeof before) == 0 && cui_get_theme_mode(ui) == CUI_THEME_DARK;
}

int main(void) {
    cui_sw_renderer *renderer = cui_sw_create();
    if (!renderer) return 1;
    const cui_ctx_desc options = { .renderer = cui_sw_iface(renderer) };
    cui_ctx *ui = cui_create(&options);
    if (!ui) { cui_sw_destroy(renderer); return 1; }
    cui_set_viewport(ui, 640, 480, 1);
    cui_set_reduced_motion(ui, true);
    const bool passed = verify(ui);
    cui_destroy(ui); cui_sw_destroy(renderer);
    if (!passed) fprintf(stderr, "Theme toggle changed the palette, lost identity or ignored disabled input.\n");
    return passed ? 0 : 1;
}
