#ifndef FPAS_CUI_SCHEDULE_PICKER_H
#define FPAS_CUI_SCHEDULE_PICKER_H
/** Draws the native schedule editor and reports its next occurrence.
 * @param ui Current frame context.
 * @param id Stable widget identity.
 * @param value Null-terminated editable schedule buffer.
 * @param capacity Buffer capacity in bytes, including the terminator.
 * @param now Reference Unix time in seconds.
 * @param utc Nonzero to interpret and display times in UTC.
 * @param next_count Number of future occurrences to display.
 * @param width_kind FPAS width extent kind.
 * @param width Width basis for the chosen extent.
 * @param enabled Nonzero to accept input outside disabled scopes.
 * @param out Receives changed (bit 0), valid (bit 1), and next Unix time in x. */
CUI_API void fpas_cui_schedule_picker(cui_ctx *ui, const char *id, char *value, size_t capacity, int64_t now, int utc, int next_count, int width_kind, float width, int enabled, fpas_dev_result *out);
#endif
