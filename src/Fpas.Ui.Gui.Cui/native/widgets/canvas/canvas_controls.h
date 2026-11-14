#ifndef FPAS_CUI_CANVAS_CONTROLS_H
#define FPAS_CUI_CANVAS_CONTROLS_H
typedef struct { const char *icon, *label; } fpas_canvas_toggle;
CUI_API int fpas_cui_canvas_controls(cui_ctx *ui, const char *id, cui_node_view *view, cui_rect content,
    const float *steps, int step_count, const fpas_canvas_toggle *toggles, bool *values, int count, int flags);
#endif
