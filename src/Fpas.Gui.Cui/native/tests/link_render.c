/* Checks real consumer linkage, font initialization and software rendering
 * against either the shared library or static archive, without a display. */
#include "cui_single.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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
    for (int frame = 0; frame < 3; frame++) {
        cui_begin_frame(ui, 1.0f / 60.0f);
        cui_column(ui, .pad = 12, .gap = 8) {
            cui_heading(ui, "FPAS native CUI", 2);
            cui_button(ui, "Linked successfully");
        }
        cui_end_frame(ui);
    }
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
