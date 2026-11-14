#ifndef FPAS_CUI_HEX_VIEW_H
#define FPAS_CUI_HEX_VIEW_H
typedef struct { int start, length, tone; const char *label; } fpas_hex_mark;
CUI_API int fpas_cui_hex_view(cui_ctx *ui, const char *id, uint8_t *bytes, int count, int *start, int *length, int *cursor, const fpas_hex_mark *marks, int mark_count, uint64_t address, int go_to, int bytes_per_row, int width_kind, float width, float height, float size, int flags, int *edit_offset, int *edit_length);
#endif
