#include "../fpas_cui.h"

void fpas_cui_display_text(cui_ctx *ui, const char *text, int role) {
    switch (role) {
    case 0: cui_text(ui, text); break;
    case 1: case 2: case 3: case 4: cui_heading(ui, text, role); break;
    case 5: cui_label(ui, text); break;
    case 6: cui_secondary(ui, text); break;
    case 7: cui_muted(ui, text); break;
    case 8: cui_code(ui, text); break;
    }
}
