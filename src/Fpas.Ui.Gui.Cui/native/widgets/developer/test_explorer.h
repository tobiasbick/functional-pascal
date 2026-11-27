// [Documentation](../../../../../docs/ui/gui/more-developer-tools/test-explorer.md)
#ifndef FPAS_CUI_TEST_EXPLORER_H
#define FPAS_CUI_TEST_EXPLORER_H
/** Draws the test explorer. Entry arrays and strings are borrowed for this call.
 * Height zero uses 320 logical units and negative height fills the parent.
 * The result contains selected, activated and run indices in index, action and
 * hover, run-all in flags, and total/passed/failed/skipped in count/x/y/z. */
CUI_API void fpas_cui_test_explorer(cui_ctx *ui, const char *id, const cui_test_item *entries,
    int entries_count, float height, int width_kind, float width, int enabled, fpas_dev_result *out);
#endif
