# Neovim DAP prototype

Status: **Incomplete** external-client configuration. The
[MCP agent debugger](../../docs/future/debugger/README.md) has independent scope.
[fpas_debug.lua](lua/fpas_debug.lua) configures nvim-dap to launch unmodified
NetCoreDbg directly. It provides preflight checks and a stopped-state scope view;
loading the module does not launch, build or download anything.

Install Neovim 0.11.7, nvim-dap 0.10.0 and NetCoreDbg 3.2.0-1092 separately.
nvim-dap is GPL-3.0; NetCoreDbg is MIT. Keep upstream notices with those external
tools. Build an FPAS program with matching portable symbols and make .NET
available to the backend. Adapt these placeholder paths:

```lua
local repository = '/path/to/functional-pascal'
vim.opt.rtp:append(repository .. '/integrations/neovim')
require('fpas_debug').setup({
  adapter = '/path/to/netcoredbg',
  program = repository .. '/examples/hello/bin/Debug/net10.0/Hello.dll',
  cwd = repository,
  args = {},
})
```

Open an FPAS source and use `:DapToggleBreakpoint`, `:DapContinue`,
`:DapStepOver`, `:DapStepInto` and `:DapStepOut`.
`:lua require('fpas_debug').scopes()` shows current stopped-frame values;
`:lua require('dap').repl.open()` shows output/evaluation controls.
Use `:lua require('dap').terminate()` before closing the client.

Program stdin is unavailable on this route; REPL input is debugger input.
Initialization warnings and disconnect timeouts are not general request
cancellation. Adapter crash/hang cleanup and exit with an active session remain
open. Generic scope widgets/printed REPL history can retain previous values;
the FPAS scope view invalidates frame references on resume.

IDE debugging is documented separately in the
[VS Code guide](../../docs/pascal/tools/debugging.md). No editor/backend binary,
protocol proxy or batch debugging interface is packaged with FPAS.
