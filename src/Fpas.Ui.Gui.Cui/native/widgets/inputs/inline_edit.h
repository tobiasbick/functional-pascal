#ifndef FPAS_INLINE_EDIT_H
#define FPAS_INLINE_EDIT_H
/** Draws CUI inline edit, retaining an uncommitted UTF-8 draft until native commit/cancel.
 * value receives the current draft; flags report CUI_CHANGED only for a commit.
 * capacity includes the terminator and must not exceed CUI's 512-byte backup.
 * node/key identify the native outer control and its retained state.
 * @param ui Current frame context.
 * @param id Stable application identity.
 * @param value Confirmed model text on entry, native draft on return.
 * @param capacity Buffer capacity including terminator, at most 512.
 * @param placeholder Optional literal empty-state text.
 * @param font Native built-in font ordinal.
 * @param size Logical font size, zero for the native default.
 * @param width_kind Content zero, fill one, fixed two or percentage three.
 * @param width Logical width or percentage.
 * @param enabled Whether editing accepts input.
 * @param focus Whether to request the current interaction mode's focus.
 * @param editing Receives whether native editing remains active.
 * @param node Receives the visible outer node.
 * @param key Receives the stable retained-state owner.
 * @return Native commit flags, zero for an uncommitted or disabled draw. */
CUI_API int fpas_cui_inline_edit(cui_ctx *ui, const char *id, char *value, size_t capacity,
    const char *placeholder, int font, float size, int width_kind, float width,
    int enabled, int focus, int *editing, int *node, unsigned int *key);
#endif
