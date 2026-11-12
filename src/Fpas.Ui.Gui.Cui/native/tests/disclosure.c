/* Compare adapter output to the actual CUI widgets, including omitted decorations. */
#include "../fpas_cui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static bool pixels_match(int kind, bool open, bool decorated, bool dark, int width) {
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
        cui_set_reduced_motion(ui, true);
        cui_set_theme(ui, dark ? CUI_THEME_DARK : CUI_THEME_LIGHT);
        for (int frame = 0; frame < 4; ++frame) {
            cui_begin_frame(ui, 0.04f);
            cui_column(ui, .width = CUI_FILL, .pad = 16, .gap = 8) {
                if (kind == 2) {
                    const char *text = "Native text wraps across several lines. Literal ## hashes are retained. "
                        "Native text wraps across several lines. Native text wraps across several lines. "
                        "Native text wraps across several lines. Native text wraps across several lines.";
                    const cui_expandable_text_opts options = { .lines = 2,
                        .size = decorated ? 18 : 0,
                        .color = decorated ? CUI_RGB(0x21435d) : (cui_color){0},
                        .fade_to = decorated ? CUI_RGB(0xaaddee) : (cui_color){0} };
                    if (pass) {
                        int toggle;
                        fpas_cui_expandable_text(ui, "text", text, 2, options.size, decorated ? 3 : 0,
                            decorated ? 0xff5d4321u : 0, decorated ? 0xffeeddaau : 0, 1, 0, &toggle);
                    } else cui_expandable_text_ex(ui, "text", text, &options);
                } else {
                    bool value = open;
                    const cui_expandable_card_opts options = {
                        .description = decorated ? "Supporting description" : NULL,
                        .icon = decorated ? CUI_ICON_CREDIT_CARD : NULL,
                        .badge = decorated ? "Active" : NULL, .badge_tone = CUI_TONE_SUCCESS
                    };
                    int node, header;
                    bool draw;
                    if (pass) draw = (fpas_cui_disclosure_begin(ui, "###header", "Native title", kind,
                        open, 1, 0, options.description ? options.description : "",
                        options.icon ? options.icon : "", options.badge ? options.badge : "",
                        CUI_TONE_SUCCESS, &node, &header) & 1) != 0;
                    else draw = kind ? cui_expandable_card_begin(ui, "Native title", &value, &options)
                                     : cui_accordion_begin(ui, "Native title", &value);
                    if (draw) {
                        cui_text(ui, "The same native body.");
                        cui_button(ui, "Body action");
                        if (pass) fpas_cui_disclosure_end(ui, kind, node);
                        else if (kind) cui_expandable_card_end(ui);
                        else cui_accordion_end(ui);
                    }
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
    if (!valid) fprintf(stderr, "Disclosure pixel mismatch: kind=%d open=%d decorated=%d dark=%d width=%d\n",
        kind, open, decorated, dark, width);
    return valid;
}

int main(void) {
    bool valid = true;
    for (int kind = 0; kind < 3; ++kind)
        for (int open = 0; open < 2; ++open)
            for (int decorated = 0; decorated < 2; ++decorated)
                for (int dark = 0; dark < 2; ++dark)
                    for (int narrow = 0; narrow < 2; ++narrow)
                        valid = pixels_match(kind, open != 0, decorated != 0, dark != 0,
                            narrow ? 280 : 640) && valid;
    return valid ? 0 : 1;
}
