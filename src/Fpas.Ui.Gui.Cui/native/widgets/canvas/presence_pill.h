// [Documentation](../../../../../docs/ui/gui/canvas-and-bars/presence-pill.md)
#ifndef FPAS_CUI_PRESENCE_PILL_H
#define FPAS_CUI_PRESENCE_PILL_H
/** Draws the pill with optional images from UTF-8 PNG or JPEG paths.
 * @param ui Drawing context.
 * @param id Stable widget identity.
 * @param people Borrowed native collaborator data.
 * @param avatars Optional borrowed path array in collaborator order.
 * @param count Number of collaborators.
 * @param size Avatar diameter, or zero for the native default.
 * @param max_visible Visible avatar limit, or zero for the native default. */
CUI_API void fpas_cui_presence_pill(cui_ctx *ui, const char *id, const cui_live_user *people,
    const char *const *avatars, int count, float size, int max_visible);
#endif
