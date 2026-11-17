#ifndef FPAS_CUI_SELECTION_BAR_H
#define FPAS_CUI_SELECTION_BAR_H
/** Builds CUI's selection bar from item flags or a count, using a validated integer count format. */
CUI_API int fpas_cui_selection_bar(cui_ctx *ui, const char *id, bool *selected, int item_count, int count,
    const cui_selection_action *actions, int action_count, int enabled, int no_escape, const char *count_format);
#endif
