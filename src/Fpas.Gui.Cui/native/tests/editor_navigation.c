#include "../fpas_cui.h"
#include <cui/cui_render.h>
#include <stdio.h>
#include <string.h>

typedef struct {
    char text[64];
    int64_t version;
    int anchor, caret;
} document;

static const char editor_id[] = "editor";

static int draw_editor(cui_ctx *ui, document *doc, cui_key key, int mods) {
    cui_begin_frame(ui, 1.0f / 60.0f);
    if (key != CUI_KEY_NONE) {
        cui_input_key(ui, key, true, mods);
        cui_input_key(ui, key, false, mods);
    }
    int events = fpas_cui_text_edit(ui, editor_id, doc->text, sizeof doc->text, 96,
                                    doc->version, &doc->anchor, &doc->caret);
    cui_end_frame(ui);
    return events;
}

static bool at_selection(const document *doc, int anchor, int caret) {
    return doc->anchor == anchor && doc->caret == caret;
}

static bool check_editor(cui_ctx *ui) {
    document doc = { .text = "aa\n\xc3\xa4" "b\ncc", .version = 1 };
    fpas_cui_request_focus(ui, editor_id);
    if (!(draw_editor(ui, &doc, CUI_KEY_NONE, 0) & CUI_FOCUSED)) return false;

    draw_editor(ui, &doc, CUI_KEY_DOWN, CUI_MOD_SHIFT);
    if (!at_selection(&doc, 0, 3)) return false;
    draw_editor(ui, &doc, CUI_KEY_END, CUI_MOD_SHIFT);
    if (!at_selection(&doc, 0, 6)) return false;
    draw_editor(ui, &doc, CUI_KEY_HOME, 0);
    if (!at_selection(&doc, 3, 3)) return false;
    draw_editor(ui, &doc, CUI_KEY_DOWN, 0);
    if (!at_selection(&doc, 7, 7)) return false;
    draw_editor(ui, &doc, CUI_KEY_UP, 0);
    if (!at_selection(&doc, 3, 3)) return false;

    cui_input_text(ui, "X");
    if (!(draw_editor(ui, &doc, CUI_KEY_NONE, 0) & CUI_CHANGED)) return false;
    if (!(draw_editor(ui, &doc, CUI_KEY_Z, CUI_MOD_PRIMARY) & CUI_CHANGED)) return false;
    if (strcmp(doc.text, "aa\n\xc3\xa4" "b\ncc") != 0) return false;

    doc = (document){ .text = "new\nrow", .version = 2, .anchor = 4, .caret = 4 };
    if (draw_editor(ui, &doc, CUI_KEY_Y, CUI_MOD_PRIMARY) & CUI_CHANGED) return false;
    if (draw_editor(ui, &doc, CUI_KEY_Z, CUI_MOD_PRIMARY) & CUI_CHANGED) return false;
    return strcmp(doc.text, "new\nrow") == 0 && at_selection(&doc, 4, 4);
}

int main(void) {
    cui_sw_renderer *renderer = cui_sw_create();
    if (!renderer) return 1;
    const cui_ctx_desc settings = { .renderer = cui_sw_iface(renderer) };
    cui_ctx *ui = cui_create(&settings);
    bool ok = ui && check_editor(ui);
    if (ui) cui_destroy(ui);
    cui_sw_destroy(renderer);
    if (!ok) fprintf(stderr, "CUI editor navigation or document history check failed.\n");
    return ok ? 0 : 1;
}
