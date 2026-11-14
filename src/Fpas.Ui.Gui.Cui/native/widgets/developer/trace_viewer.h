#ifndef FPAS_CUI_TRACE_VIEWER_H
#define FPAS_CUI_TRACE_VIEWER_H
CUI_API void fpas_cui_trace_viewer(cui_ctx *ui, const char *id, const cui_trace_track * tracks, int tracks_count, const cui_trace_span * spans, int spans_count, const cui_trace_marker * markers, int markers_count, double start, double stop, float scroll, double mark_start, double mark_end, int selection, float height, float row_height, int width_kind, float width, int enabled, fpas_dev_result * out);
#endif
