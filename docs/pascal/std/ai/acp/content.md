# Content and session updates

Stable values live in `Std.Acp.Types`. See [protocol values](types.md) for the
complete SDK-derived types, nominal IDs, raw JSON and patch fields, and
[sessions](sessions.md) for prompt ownership and cancellation.

Prompts support text and resource links without optional prompt capabilities.
Images require `promptCapabilities.image`, audio requires `.audio`, and
embedded resources require `.embeddedContext`. Unsupported content is rejected
before publication. The capability check does not fetch resource links, decode
media, read files or execute tools.

Inbound events preserve text, image, audio, embedded text/blob resources and
resource links. Tool content includes ordinary content blocks, diffs and
terminal references. Metadata and arbitrary tool inputs/outputs retain their
raw numeric spellings; `RawMessage` also preserves ignored fields.

Every stable session update is forwarded in received order: user/agent message
chunks, thought chunks, tool calls and tool patches, plans, available commands,
mode/config changes, session information, usage, notices, compaction updates
and compaction summary chunks. Repeated compaction chunks remain separate
events. Message, tool and compaction IDs retain their protocol identity.
Unknown open-enum values retain their unknown representation and raw data.

Nullable patches distinguish `Types.Field.Missing`, `NullValue` and
`Value(Replacement)`. A missing tool title/content/output does not mean clear;
an explicit null differs from both omission and an empty replacement array.
Applications decide how to reduce these patches into their own UI state.

The connection stores no transcript, tool history or compaction summary.
Only bounded modes/config choices and request/session ownership are retained
for admission checks. Applications should also bound any event history they
retain. See [connection limits](connections.md#bounds-and-cleanup).
