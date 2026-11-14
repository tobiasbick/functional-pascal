#ifndef FPAS_SHORTCUT_RECORDER_H
#define FPAS_SHORTCUT_RECORDER_H
/** Draws CUI's recorder with native key/modifier values and optional clear action tracking.
 * @param ui Current frame context.
 * @param id Stable recorder identity.
 * @param key Model key ordinal, updated with the native proposal.
 * @param mods Model modifier bits, updated with the native proposal.
 * @param placeholder Optional literal empty-state caption.
 * @param width_kind Content zero, fill one, fixed two or percentage three.
 * @param width Logical width or percentage.
 * @param enabled Whether recording and clearing accept input.
 * @param focus Zero for none, one for the recorder, two for its clear action.
 * @param nodes Receives the recorder and clear nodes, minus one when absent. */
CUI_API void fpas_cui_shortcut_recorder(cui_ctx *ui, const char *id, int *key, int *mods,
    const char *placeholder, int width_kind, float width, int enabled, int focus, int *nodes);
/** Copies CUI's platform-specific shortcut label into the caller's buffer, including its terminator.
 * @param ui Current frame context.
 * @param key Native key ordinal.
 * @param mods Native modifier bits.
 * @param text Destination buffer for the copied native label.
 * @param capacity Destination size including the terminator. */
CUI_API void fpas_cui_shortcut_label(cui_ctx *ui, int key, int mods, char *text, size_t capacity);
/** Draws native keycaps and copies the formatted label for host inspection.
 * @param ui Current frame context.
 * @param key Native key ordinal.
 * @param mods Native modifier bits.
 * @param text Destination buffer for the copied label.
 * @param capacity Destination size including the terminator. */
CUI_API void fpas_cui_shortcut_display(cui_ctx *ui, int key, int mods, char *text, size_t capacity);
#endif
