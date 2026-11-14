#ifndef FPAS_CUI_CARD_H
#define FPAS_CUI_CARD_H

/* Opens a native card, returns activation and writes the node. End it with fpas_cui_card_end. */
CUI_API int fpas_cui_card_begin(cui_ctx *ui, const char *id, int width_kind, float width,
    int height_kind, float height, float minimum, float maximum, float pad, float pad_x,
    float pad_y, float gap, int row, int align, int wrap, int muted, int selected,
    int hoverable, int enabled, int focus, int *node);

/* Closes the native card and makes it the last widget. */
CUI_API void fpas_cui_card_end(cui_ctx *ui, int node);

#endif
