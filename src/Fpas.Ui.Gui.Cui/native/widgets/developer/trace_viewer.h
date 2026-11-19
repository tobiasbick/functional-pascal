// [Documentation](../../../../../docs/ui/gui/developer-tools/trace-viewer.md)
#ifndef FPAS_CUI_TRACE_VIEWER_H
#define FPAS_CUI_TRACE_VIEWER_H
/* Link and span indices use the caller's sorted span array. */
CUI_API void fpas_cui_trace_viewer(cui_ctx *ui, const char *id, const cui_trace_track * tracks, int tracks_count, const cui_trace_span * spans, int spans_count, const cui_trace_marker * markers, int markers_count, double start, double stop, float scroll, double mark_start, double mark_end, int selection, float height, float row_height, int width_kind, float width, int enabled, fpas_dev_result * out, const cui_trace_link *links, int link_count, int *hovered_link);
/* Read the native geometry of a span after presentation; invisible spans return an empty rectangle. */
CUI_API cui_rect fpas_cui_trace_span_rect(cui_ctx *ui, const char *id, double start, double end, int track, int depth);
#endif
