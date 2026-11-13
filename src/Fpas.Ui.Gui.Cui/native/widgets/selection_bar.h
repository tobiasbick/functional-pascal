#ifndef FPAS_CUI_SELECTION_BAR_H
#define FPAS_CUI_SELECTION_BAR_H
CUI_API int fpas_cui_selection_bar(cui_ctx *ui, const char *id, bool *selected, int count,
    const cui_selection_action *actions, int action_count, int enabled, int no_escape);
#endif
