// [Documentation](../../../../../docs/ui/gui/code/code-editor.md)
#ifndef FPAS_CUI_CODE_EDITOR_H
#define FPAS_CUI_CODE_EDITOR_H
/** Detached native editor observations; cursor and anchor are UTF-8 byte offsets. */
typedef struct {
    int line;        /**< One-based caret line. */
    int column;      /**< One-based visual column. */
    int lines;       /**< Document line count. */
    int cursor;      /**< UTF-8 caret offset. */
    int anchor;      /**< UTF-8 selection anchor. */
    int matches;     /**< Find result count. */
    int match_index; /**< One-based current match, or zero. */
    int focused;     /**< Editor or find field owns input. */
    int length;      /**< UTF-8 source length excluding the terminator. */
    int bracket;     /**< UTF-8 offset of the bracket beside the caret, or minus one. */
    int bracket_match; /**< UTF-8 offset of the paired bracket, or minus one. */
} fpas_code_editor_observation;

/** Builds the native editor, which copies visible highlights, and reports changes.
 * @param ui Owner-thread context.
 * @param id Stable editor identity.
 * @param text Mutable UTF-8 document, terminated within capacity.
 * @param capacity Buffer size including the terminator.
 * @param language Native syntax language.
 * @param placeholder Optional empty-document hint.
 * @param width_kind Native content/fill/fixed/percent extent.
 * @param width Logical fixed/percent extent value.
 * @param height Logical height, zero for default or minus one to fill.
 * @param size Logical font size, zero for default.
 * @param tab_size Visual columns per indentation level.
 * @param flags Enabled/read-only/tabs/no-gutter/focus/autofocus bits.
 * @param marks Sorted borrowed UTF-8 highlight ranges, used during this call.
 * @param mark_count Number of ranges.
 * @param observation Detached caret, selection, focus and find result.
 * @return Changed/refused/observation bits. */
CUI_API int fpas_cui_code_editor(cui_ctx *ui, const char *id, char *text, size_t capacity,
    const char *language, const char *placeholder, int width_kind, float width, float height,
    float size, int tab_size, int flags, const cui_text_mark *marks, int mark_count,
    fpas_code_editor_observation *observation);
#endif
