// [Documentation](../../../../../docs/ui/gui/voice/composer.md)
#ifndef FPAS_CUI_COMPOSER_H
#define FPAS_CUI_COMPOSER_H
/** Draws the chat editor or recorder and returns native event bits.
 * Input arrays and NUL-terminated UTF-8 strings are borrowed for this call.
 * draft must contain a terminator within capacity; edits remain within capacity.
 * @param ui Current UI-thread frame context.
 * @param id Stable composer identity.
 * @param draft Writable application draft using LF line endings.
 * @param capacity Draft buffer size in bytes, including its terminator.
 * @param placeholder Empty-draft caption, or NULL for the native default.
 * @param attachments Array of attachment captions.
 * @param attachment_count Number of attachment captions.
 * @param models Available model records.
 * @param model_count Number of models.
 * @param selected Current model index, replaced by a selection proposal.
 * @param levels Recording signal samples, or NULL when level_count is zero.
 * @param level_count Number of recording samples.
 * @param seconds Elapsed recording time.
 * @param max_lines Editor line limit, or zero for the native default.
 * @param width_kind FPAS extent kind.
 * @param width Width basis for that extent.
 * @param flags Bit 0 enables input, bit 1 indicates busy, bit 2 shows the
 * microphone, and bit 3 indicates recording.
 * @param focus Requested part index or -1. Absent parts cannot receive focus.
 * @param removed Receives the attachment removal proposal; initialize to -1.
 * @param nodes Receives 6 + attachment_count node indices. The editor, send,
 * microphone, cancel, attach and model controls precede the attachment remove
 * controls in input order. Absent parts are -1. Indices are current-frame only.
 * @return Native CUI_COMPOSER_* event bitmask, or zero while disabled. */
CUI_API int fpas_cui_composer(cui_ctx *ui, const char *id, char *draft, size_t capacity, const char *placeholder,
    const char *const *attachments, int attachment_count, const cui_model *models, int model_count, int *selected,
    const float *levels, int level_count, float seconds, int max_lines, int width_kind, float width,
    int flags, int focus, int *removed, int *nodes);
#endif
