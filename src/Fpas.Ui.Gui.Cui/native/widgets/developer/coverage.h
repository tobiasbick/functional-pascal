// [Documentation](../../../../../docs/ui/gui/more-developer-tools/coverage.md)
#ifndef FPAS_CUI_COVERAGE_H
#define FPAS_CUI_COVERAGE_H
/** Draws file coverage, refreshing path data on every call.
 * The file array and NUL-terminated UTF-8 paths are borrowed for this call.
 * Cached path segments are rebuilt before use; sorting and folds remain native.
 * @param ui Current UI-thread frame context.
 * @param id Stable widget identity.
 * @param files Coverage records in application order.
 * @param files_count Number of records.
 * @param height Logical viewport height.
 * @param low Low coverage threshold used for coloring.
 * @param high High coverage threshold used for coloring.
 * @param width_kind FPAS extent kind.
 * @param width Width basis for that extent.
 * @param flags Bit zero enables input. Bit one requests keyboard focus.
 * @param out Receives action as the activated file index (or -1) and x as lines. */
CUI_API void fpas_cui_coverage(cui_ctx *ui, const char *id, const cui_coverage_file *files, int files_count,
    float height, float low, float high, int width_kind, float width, int flags, fpas_dev_result *out);
#endif
