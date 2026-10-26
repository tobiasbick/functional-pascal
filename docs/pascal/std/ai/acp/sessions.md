# Stable v1 sessions

Use `Std.Acp.Send(Connection, Types.ClientRequest...)` for requests and
`Std.Acp.Notify(Connection, Types.ClientNotification...)` for notifications.
Both admit work locally; completion arrives through the matching RequestToken
in a response or request-failure event. See [connections](connections.md) for
transport setup, exclusive reception and bounded cleanup.

## Initialization and authentication

Send `ClientRequest.InitializeRequest` once with a `Types.InitializeRequest` matching
the connection options. A locally rejected offer publishes nothing and can be
corrected. Admission changes the lifecycle to `Initializing`; successful
negotiation changes it to `Ready` before its response becomes observable.
Another initialize returns `Busy('Initialize')`. Unsupported peer versions and
failed initialization produce a request failure and close the connection.

During initialization, advertised form/URL elicitation is accepted only when
its `requestId` identifies an outstanding request. Ordinary session requests
wait until Ready. Host filesystem and terminal requests require the advertised
client capability and an admitted session; opening a connection grants no
automatic host access. Application-defined underscore extension messages
remain available independently of stable session state.

The initialize response and `Acp.State` expose agent information, negotiated
version, complete agent capabilities and authentication methods. The client
does not infer login status or automatically retry an authentication-required
peer error. Send `authenticate` with an advertised agent-handled method ID,
then retry the desired request. Terminal authentication methods require the
separate host-adapter workflow; passing one to Authenticate returns
`UnsupportedCapability('TerminalAuthentication')`. Logout requires the
advertised `auth.logout` object.

## Session lifecycle and capability checks

| Request | Local admission rule |
| --- | --- |
| `session/new` | Ready, available session slot, supported MCP/root descriptors |
| `session/load` | Advertised `loadSession`; installs replay state before sending |
| `session/resume` | Advertised `sessionCapabilities.resume`; remains distinct from load |
| `session/list` | Advertised `sessionCapabilities.list`; returns one peer page |
| `session/delete` | Advertised `sessionCapabilities.delete`; removes remote history |
| `session/close` | Advertised `sessionCapabilities.close`; releases local active resources on success |
| `session/set_mode` | An admitted session and an advertised mode ID |
| `session/set_config_option` | An advertised config ID and matching boolean/select value |
| `session/prompt` | An admitted session with no active prompt; supported content |

Optional object capabilities distinguish absence/null from `{}`. An empty
list/resume/close/logout capability object advertises support. Boolean false
does not. An empty client elicitation capability supports neither form nor URL.
Checks use the pinned SDK's defaults for invalid optional fields.

Additional workspace directories require `additionalDirectories` support.
Stdio MCP descriptors require no extra capability; HTTP and SSE descriptors
require their corresponding MCP capabilities. No provider or model names are
hardcoded. Session modes and config choices come from setup responses and
subsequent config updates. Config arrays replace the previously advertised
array, including when the replacement is empty.

New/load/resume requests reserve session capacity before publication. Failed
setup returns its reservation. Replay updates may arrive before the setup
response and are delivered once in received order. The application keeps the
session ID; the connection retains only bounded admission/config state.
List cursors and entries are returned unchanged without automatic paging.
Delete and close have separate meanings: deleting history does not release an
active local session. Closing cancels its pending permission responders and
releases its local modes/config state after a successful peer response.

## Prompts and cancellation

Independent sessions may run concurrently. A second prompt for the same
session returns `Busy('SessionPrompt')` before publication. Empty chunks, tool
completion, compaction and silence do not finish a prompt. Only its actual
response, peer error or terminal connection failure settles its token.

The `session/cancel` notification consumes unanswered permission
responders belonging to that session. They receive the protocol's cancelled
permission outcome; the application sees `RequestCancelled`. A reply already
admitted by the application wins over a later cancellation. New permission
requests for the cancelled active prompt receive cancelled automatically.
Other sessions continue. Continue receiving the cancelled prompt's final
response and permitted late updates.

`CancelRequest` instead uses a request token and `$/cancel_request`; it does
not perform session cancellation. Receive timeout and local wait cancellation
consume no events and do not cancel peer work. Closing a session and reopening
the same ID does not let an old late response release the new prompt owner.

The [native peer regressions](../../../../../crates/fpas-acp/tests/v1.rs) own
method, capability, replay and cancellation behavior. The
[public FPAS regression](../../../../../tests/stdlib/acp/client_test.fpas)
checks configured offers and negotiated-state conversion.
