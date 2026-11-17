# Native CUI APIs

Build/run `NativeApi.fpasproj` with the local FPAS SDK. This companion to the
[widget gallery](../gallery/README.md) demonstrates optional canvas wheel
messages, deferred chart-label callbacks, trace dependency links, native hover
and the owner-thread span-geometry command. The regular gallery keeps its
upstream composition.

Move the wheel over the empty canvas to change both line counters. Hover the
chart for the custom value readout. Hover the dashed trace dependency to see
its original link index, then use **Read source span bounds** to inspect CUI's
last presented geometry. The example stores acknowledged hover in its model,
so unchanged hover does not produce repeated messages.

Label callbacks run on the owner thread while CUI paints. They must be pure,
short and return valid Unicode without NUL. CUI supplies a bounded UTF-8
buffer; excess text is cut at a complete Unicode scalar. An exception is
re-raised after native rendering returns, without crossing C frames.
