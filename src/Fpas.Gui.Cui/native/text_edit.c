#include "fpas_cui.h"
#include "cui_internal.h"

typedef struct {
    /* CUI owns the first member and releases its undo history with the widget state. */
    cui_edit_state editor;
    int64_t document_version;
} fpas_editor_state;

static void reset_document(cui_ctx *ui, cui_edit_state *editor) {
    for (int i = 0; i < editor->undo_count; ++i) {
        cui__free(ui, editor->undo[i]);
        editor->undo[i] = NULL;
    }
    editor->undo_count = editor->undo_pos = 0;
    editor->scroll_x = editor->scroll_y = 0;
    editor->preferred_x = -1;
    editor->selecting = false;
}

static int draw_text_editor(cui_ctx *ui, const char *id, char *buffer, size_t capacity, float height,
                            int64_t document_version, int *anchor, int *caret) {
    if (!buffer || capacity == 0) return 0;
    cui_id key = cui_make_id(ui, id);
    fpas_editor_state *state = cui_state(ui, key, sizeof *state);
    cui_edit_state *editor = &state->editor;
    if (anchor) {
        if (state->document_version != document_version) {
            reset_document(ui, editor);
            state->document_version = document_version;
        }
        if (editor->cursor != *caret || editor->anchor != *anchor) {
            editor->cursor = *caret;
            editor->anchor = *anchor;
            editor->preferred_x = -1;
            ui->caret_time = ui->time;
        }
    }

    const cui_input_opts options = { .height = height };
    int events = height > 0
        ? cui_textarea_ex(ui, id, buffer, capacity, &options)
        : cui_input_ex(ui, id, buffer, capacity, NULL);
    /* CUI otherwise leaves typed text available to another editor that gains focus this frame. */
    if (ui->edit == editor || ui->focus == key) {
        ui->text_in_len = 0;
        ui->text_in[0] = '\0';
    }
    if (anchor) {
        *anchor = editor->anchor;
        *caret = editor->cursor;
    }
    return events;
}

int fpas_cui_text_input(cui_ctx *ui, const char *id, char *buffer, size_t capacity) {
    return draw_text_editor(ui, id, buffer, capacity, 0, 0, NULL, NULL);
}

int fpas_cui_text_area(cui_ctx *ui, const char *id, char *buffer, size_t capacity, float height) {
    return draw_text_editor(ui, id, buffer, capacity, height, 0, NULL, NULL);
}

int fpas_cui_text_edit(cui_ctx *ui, const char *id, char *buffer, size_t capacity, float height,
                       int64_t document_version, int *anchor, int *caret) {
    return draw_text_editor(ui, id, buffer, capacity, height, document_version, anchor, caret);
}
