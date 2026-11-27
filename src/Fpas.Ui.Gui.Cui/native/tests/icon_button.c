// [Documentation](../../../../docs/ui/gui/actions/icon-button.md)
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
        dark ? "Switch to light" : "Switch to dark", NULL, 0, CUI_SIZE_MD, enabled, 0, 0, focus, 0, 0, 0);
    *identity = fpas_cui_node_key(ui, fpas_cui_last_node(ui));
    cui_end_frame(ui);
    return activated;
}

static bool verify(cui_ctx *ui) {
    uint32_t before[CUI_TOKEN_COUNT - 1], after[CUI_TOKEN_COUNT - 1];
    fpas_cui_theme_read(ui, before);
    before[0] = 0xff352719u;
    fpas_cui_theme(ui, 1, 2, before, NULL, 0);
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

/** Compares every supported width and a foreground override with CUI. */
static bool options_match(cui_ctx *ui, cui_sw_renderer *renderer) {
    enum { WIDTH = 360, HEIGHT = 64 };
    unsigned char pixels[2][WIDTH * HEIGHT * 4];
    const float widths[] = { 0, 0, 118, 56 };
    const cui_size extents[] = { CUI_FIT, CUI_FILL, CUI_PX(118), CUI_PCT(56) };
    const cui_color foreground = { 13, 127, 203, 255 };
    cui_set_viewport(ui, WIDTH, HEIGHT, 1);
    cui_input_mouse_leave(ui);
    for (int kind = 0; kind < 4; ++kind) {
        for (int bridge = 0; bridge < 2; ++bridge) {
            cui_begin_frame(ui, 0.04f);
            cui_column(ui, .width = CUI_FILL, .pad = 8) {
                if (bridge) {
                    fpas_cui_icon_button(ui, "###width", CUI_ICON_COPY, "Copy", NULL,
                        5, CUI_SIZE_MD, 1, 0, 0, 0, kind, widths[kind], 0xffcb7f0du);
                } else {
                    cui_icon_button(ui, CUI_ICON_COPY, .tooltip = "Copy", .variant = CUI_BUTTON_OUTLINE,
                        .full_width = kind == 1, .width = extents[kind], .color = foreground);
                }
            }
            cui_end_frame(ui);
            cui_sw_render(renderer, cui_get_draw_data(ui), pixels[bridge], WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
        }
        if (memcmp(pixels[0], pixels[1], sizeof pixels[0]) != 0) {
            fprintf(stderr, "Icon button differs from CUI for width kind %d.\n", kind);
            return false;
        }
    }
    return true;
}

/** Checks the Elements.Size width rule against differently weighted CUI siblings. */
static bool weighted_width_match(cui_ctx *ui) {
    cui_set_viewport(ui, 360, 64, 1);
    const char *ids[] = { "###copy", "###share" };
    const char *glyphs[] = { CUI_ICON_COPY, CUI_ICON_SHARE_2 };
    const char *titles[] = { "Copy", "Share" };
    const float widths[] = { 112, 224 };
    for (int bridge = 0; bridge < 2; ++bridge) {
        int nodes[2];
        cui_begin_frame(ui, 0.04f);
        cui_row(ui, .width = CUI_FILL, .pad = 8, .gap = 8) {
            for (int index = 0; index < 2; ++index) {
                const float weight = (float)(index + 1);
                if (bridge) {
                    fpas_cui_icon_button(ui, ids[index], glyphs[index], titles[index], NULL,
                        0, CUI_SIZE_MD, 1, 0, 0, 0, 0, 0, 0);
                    fpas_cui_node_size(ui, fpas_cui_last_node(ui), 1, 1, 0, 0, -1, weight, 0);
                } else {
                    cui_icon_button(ui, glyphs[index], .tooltip = titles[index], .width = CUI_GROW(weight));
                }
                nodes[index] = fpas_cui_last_node(ui);
            }
        }
        cui_end_frame(ui);
        for (int index = 0; index < 2; ++index) {
            if (fpas_cui_node_rect(ui, nodes[index]).w != widths[index]) {
                fprintf(stderr, "Icon button did not keep its weighted width.\n");
                return false;
            }
        }
    }
    return true;
}

int main(void) {
    cui_sw_renderer *renderer = cui_sw_create();
    if (!renderer) return 1;
    const cui_ctx_desc options = { .renderer = cui_sw_iface(renderer) };
    cui_ctx *ui = cui_create(&options);
    if (!ui) { cui_sw_destroy(renderer); return 1; }
    cui_set_viewport(ui, 640, 480, 1);
    cui_set_reduced_motion(ui, true);
    const bool identity = verify(ui);
    const bool styling = options_match(ui, renderer);
    const bool weighting = weighted_width_match(ui);
    cui_destroy(ui); cui_sw_destroy(renderer);
    if (!identity) fprintf(stderr, "Theme toggle changed the palette, lost identity or ignored disabled input.\n");
    return identity && styling && weighting ? 0 : 1;
}
