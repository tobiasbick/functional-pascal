#ifndef FPAS_CUI_TIME_HISTOGRAM_H
#define FPAS_CUI_TIME_HISTOGRAM_H
CUI_API void fpas_cui_time_histogram(cui_ctx *ui, const char *id, double start, double bucket, int buckets, const cui_histogram_series * series, int series_count, double range_start, double range_end, float height, int utc, int width_kind, float width, int enabled, fpas_dev_result * out);
#endif
