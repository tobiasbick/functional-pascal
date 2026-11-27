// [Documentation](../../../../../docs/ui/gui/developer-tools/network-waterfall.md)
#ifndef FPAS_CUI_NETWORK_WATERFALL_H
#define FPAS_CUI_NETWORK_WATERFALL_H
/* Selection is -2 for native retention. Flags encode enabled and focus.
 * The result carries request indices and reports selection changes. */
CUI_API void fpas_cui_network_waterfall(cui_ctx *ui, const char *id, const cui_net_request * requests, int requests_count, int selection, float height, double dom_loaded, double loaded, int width_kind, float width, int flags, fpas_dev_result * out);
#endif
