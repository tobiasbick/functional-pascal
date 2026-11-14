#ifndef FPAS_CUI_TIME_HISTOGRAM_H
#define FPAS_CUI_TIME_HISTOGRAM_H
/** Draws the histogram and records the geometry of the plot and legend together.
 * @param ui Current UI context.
 * @param id Stable histogram identity.
 * @param start Start of the first bucket in Unix seconds.
 * @param bucket Duration of each bucket in seconds.
 * @param buckets Number of buckets in each series.
 * @param series Native histogram series and their bucket counts.
 * @param series_count Number of entries in series.
 * @param range_start Start of the current selection in Unix seconds.
 * @param range_end End of the current selection in Unix seconds.
 * @param height Plot height in logical units, or zero for the native default.
 * @param utc Nonzero to label times in UTC.
 * @param width_kind FPAS width extent kind.
 * @param width Width basis for the chosen extent.
 * @param enabled Nonzero to allow input outside disabled scopes.
 * @param out Receives the range, change flag and hovered bucket. */
CUI_API void fpas_cui_time_histogram(cui_ctx *ui, const char *id, double start, double bucket,
    int buckets, const cui_histogram_series *series, int series_count, double range_start,
    double range_end, float height, int utc, int width_kind, float width, int enabled,
    fpas_dev_result *out);
#endif
