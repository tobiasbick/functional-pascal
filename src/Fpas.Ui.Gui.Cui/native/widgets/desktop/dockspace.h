// [Documentation](../../../../../docs/ui/gui/app-chrome/dockspace.md)
#ifndef FPAS_CUI_DOCKSPACE_H
#define FPAS_CUI_DOCKSPACE_H

/* All operations use ui, the current UI-thread context. id is a stable retained identity.
 * Returned node indices are valid only in the frame that produced them. */

/** Begins a workspace; submit every panel then call dockspace_end in this frame.
 * id identifies retained layout; layout is optional dock1 text. shortcuts/enabled admit input.
 * Width/height use shared extent pairs; minimum_width/minimum_height are panel minima. */
CUI_API void fpas_cui_dockspace_begin(cui_ctx *ui, const char *id, const char *layout, int shortcuts,
    int width_kind, float width, int height_kind, float height, float minimum_width, float minimum_height,
    int enabled);
/** Submits a title###stable-key panel to the current workspace; icon is optional.
 * dirty marks unsaved work. open is an input/output presence boolean when closable;
 * a nonclosable panel is always submitted. Returns nonzero when content should be built,
 * and only then requires dock_panel_end. node receives its frame-local tab node or -1. */
CUI_API int fpas_cui_dock_panel_begin(cui_ctx *ui, const char *title, const char *icon,
    int dirty, int closable, int *open, int *node);
/** Closes a visible panel's content scope opened by dock_panel_begin. */
CUI_API void fpas_cui_dock_panel_end(cui_ctx *ui);
/** Closes the workspace after every panel has been submitted. */
CUI_API void fpas_cui_dockspace_end(cui_ctx *ui);
/** Loads dock1 layout text for retained workspace id; returns the native load result. */
CUI_API int fpas_cui_dockspace_load(cui_ctx *ui, const char *id, const char *layout);
/** Serializes retained layout for id. buffer/capacity is writable UTF-8 storage;
 * NULL/0 queries the byte length. Allocate length+1 bytes for the terminating zero.
 * Returns the native serialized length, or zero when no layout exists. */
CUI_API size_t fpas_cui_dockspace_save(cui_ctx *ui, const char *id, char *buffer, size_t capacity);

#endif
