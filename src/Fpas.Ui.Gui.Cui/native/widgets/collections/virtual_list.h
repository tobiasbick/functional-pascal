// [Documentation](../../../../../docs/ui/gui/data-display/virtual-list.md)
#ifndef FPAS_CUI_VIRTUAL_LIST_H
#define FPAS_CUI_VIRTUAL_LIST_H

/** Opens a virtual list and reports the half-open row range.
 * @param ui Owner context.
 * @param id List identity.
 * @param count Total row count.
 * @param row_height Fixed row height in logical points.
 * @param kind Viewport height extent kind.
 * @param height Height extent value.
 * @param enabled Whether the list accepts input.
 * @param restore_offset Initial offset, or -1 to retain it.
 * @param first Receives the first realized row.
 * @param last Receives the exclusive end of the realized range.
 * @return The viewport node, or -1 at a native nesting limit. */
CUI_API int fpas_cui_virtual_list_begin(cui_ctx *ui, const char *id, int count, float row_height, int kind, float height, int enabled, float restore_offset, int *first, int *last);
CUI_API void fpas_cui_virtual_list_end(cui_ctx *ui, int node);
CUI_API void fpas_cui_virtual_row_begin(cui_ctx *ui, int index);
CUI_API void fpas_cui_virtual_row_end(cui_ctx *ui);

#endif
