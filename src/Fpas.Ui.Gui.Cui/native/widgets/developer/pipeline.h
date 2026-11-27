// [Documentation](../../../../../docs/ui/gui/developer-tools/pipeline.md)
#ifndef FPAS_CUI_PIPELINE_H
#define FPAS_CUI_PIPELINE_H
/** Draws stages and jobs with optional selection highlighting.
 * @param ui Current frame context.
 * @param id Stable widget identity.
 * @param stages Stage names in display order.
 * @param stages_count Number of stages.
 * @param jobs Job descriptors.
 * @param jobs_count Number of jobs.
 * @param selection Selected job index, -1 to start unselected, or -2 to omit selection.
 * @param width_kind FPAS extent kind.
 * @param width Width basis for that extent.
 * @param flags Bit 0 enables input. Bit 1 requests focus for a job.
 * @param out Receives selected, clicked and hovered input indices. */
CUI_API void fpas_cui_pipeline(cui_ctx *ui, const char *id, const char *const *stages, int stages_count,
    const cui_pipeline_job *jobs, int jobs_count, int selection, int width_kind, float width,
    int flags, fpas_dev_result *out);
#endif
