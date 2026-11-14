#ifndef FPAS_CUI_EMPTY_STATE_H
#define FPAS_CUI_EMPTY_STATE_H

/* Draws the native empty state and returns activation of its optional action. */
CUI_API int fpas_cui_empty_state(cui_ctx *ui, const char *id, const char *title, const char *icon,
    const char *description, const char *action, int enabled, int focus, int *action_node);

#endif
