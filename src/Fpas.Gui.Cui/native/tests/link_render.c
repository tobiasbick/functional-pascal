/* Checks real consumer linkage, font initialization and software rendering
 * against either the shared library or static archive, without a display. */
#include "../fpas_cui.h"
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
    char text[64] = "";
    int anchor = 0, caret = 0;
    bool focused = false;
    int enter_events = 0;
    for (int frame = 0; frame < 3; frame++) {
        cui_begin_frame(ui, 1.0f / 60.0f);
        fpas_cui_scroll_begin(ui, "smoke-scroll", 160, 0);
        fpas_cui_column_begin(ui, 12, 8);
        fpas_cui_row_begin(ui, 0, 4);
        cui_heading(ui, "FPAS native CUI", 2);
        (void)fpas_cui_list_item(ui, "Selected", 1);
        (void)fpas_cui_tab_item(ui, "Tab", 1);
        cui_box_end(ui);
        if (frame == 0) {
            fpas_cui_request_focus(ui, "editor");
            cui_input_text(ui, "FPAS");
        }
        if (frame == 1) {
            cui_input_key(ui, CUI_KEY_ENTER, true, 0);
            cui_input_key(ui, CUI_KEY_ENTER, false, 0);
            cui_input_text(ui, "CUI");
        }
        int editor_events = fpas_cui_text_edit(ui, "editor", text, sizeof text, 96, 1, &anchor, &caret, "");
        if (editor_events & CUI_FOCUSED) focused = true;
        if (frame == 1) enter_events = editor_events;
        if (fpas_cui_table_begin(ui, "smoke-table", (const char *const[]){ "Status" }, 1)) {
            (void)fpas_cui_table_row(ui, 1);
            cui_table_cell_text(ui, "Linked successfully");
            cui_table_end(ui);
        }
        int open = 1;
        if (frame == 2 && fpas_cui_dialog_begin(ui, "CUI dialog", &open)) {
            cui_text(ui, "Modal contents");
            cui_dialog_end(ui);
        }
        cui_box_end(ui);
        cui_scroll_end(ui);
        cui_end_frame(ui);
    }
    if (!focused) {
        fprintf(stderr, "The requested control did not gain focus.\n");
        goto cleanup;
    }
    if (!(enter_events & CUI_CHANGED) || (enter_events & CUI_SUBMITTED) || strcmp(text, "FPAS\nCUI") != 0) {
        fprintf(stderr, "Enter did not insert a line break in the text area.\n");
        goto cleanup;
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
