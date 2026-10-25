# `Std.Acp.Experimental`

Typed v1 extensions from the pinned ACP SDK and schema. These declarations
cover provider configuration, fork operations, MCP routing and the supporting
experimental session, plan and subagent values. Their presence grants no
capability and enables no host effect.

Import `uses Std.Acp.Experimental as Exp;` alongside `Std.Acp.Types`.
The stable root enums include the corresponding extension variants with
payloads owned by this unit. The v1 payload codecs in `Std.Acp.Types` handle
these variants with the same validation and raw-data preservation rules.

Generated [editor declarations](../../../../../lib/api/Std/Acp.Experimental.fpas)
list every record and enum from the current pin. Protocol feature negotiation
and MCP route lifetimes belong to the connection packages.
