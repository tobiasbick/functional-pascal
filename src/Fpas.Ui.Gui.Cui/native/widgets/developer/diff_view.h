// [Documentation](../../../../../docs/ui/gui/code/diff-viewer.md)
#ifndef FPAS_CUI_DIFF_VIEW_H
#define FPAS_CUI_DIFF_VIEW_H
/** Copies up to capacity rows from first. Returns the total after folding. */
CUI_API int fpas_cui_diff_view_rows(cui_ctx *ui, const char *id, cui_diff_row *rows, int first, int capacity);

/** Detached native diff summary. */
typedef struct {
    int current, count, inline_mode; /**< Selection, hunk count, and actual layout. */
    int added, removed, folds, hidden; /**< Numbers of added, removed, fold, and hidden lines. */
    int approximate, current_changed, recomputed; /**< Flags for computation and selection changes. */
} fpas_diff_observation;
CUI_API int fpas_cui_diff_view(cui_ctx *ui, const char *id, const char *before, const char *after, int current, const char *language, const char *old_title, const char *new_title, int context, int width_kind, float width, float height, float size, float split_min_width, int flags, fpas_diff_observation *observation);
#endif
