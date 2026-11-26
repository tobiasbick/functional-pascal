// [Documentation](../../../../../docs/ui/gui/text-and-display/text.md)
#include "../../fpas_cui.h"

/** Draws the native text role and applies the text node's reading direction and alignment. */
void fpas_cui_display_text(cui_ctx *ui, const char *text, int role, int direction, int alignment) {
    switch (role) {
    case 0: {
        const cui_text_opts options = { .direction = (cui_direction)direction, .align = (cui_text_align)alignment };
        cui_text_ex(ui, text, &options);
        return;
    }
    case 1: case 2: case 3: case 4: cui_heading(ui, text, role); break;
    case 5: cui_label(ui, text); break;
    case 6: cui_secondary(ui, text); break;
    case 7: cui_muted(ui, text); break;
    case 8: cui_code(ui, text); break;
    default: return;
    }
    cui_node *node = &ui->nodes[ui->last_node];
    node->direction = (uint8_t)direction;
    node->text_align = (uint8_t)alignment;
    if (alignment != CUI_TEXT_LEFT && alignment != CUI_TEXT_START) node->self_align = CUI_ALIGN_STRETCH;
}
