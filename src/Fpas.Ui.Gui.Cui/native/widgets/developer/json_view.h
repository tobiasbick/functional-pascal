// [Documentation](../../../../../docs/ui/gui/code/json-viewer.md)
#ifndef FPAS_CUI_JSON_VIEW_H
#define FPAS_CUI_JSON_VIEW_H

/** Draws the JSON viewer with native folding and optional scrolling.
 * @param ui Current CUI context.
 * @param id Stable widget identity.
 * @param json UTF-8 JSON text.
 * @param depth Initial expansion depth, or zero for the native default.
 * @param height Maximum viewport height, or zero for no widget limit.
 * @param enabled Whether the viewer accepts pointer and wheel input.
 * @return Whether the native viewer parsed the document. */
CUI_API int fpas_cui_json_view(cui_ctx *ui, const char *id, const char *json, int depth, float height, int enabled);

#endif
